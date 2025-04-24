//! # ZecWallet Lite Parser
//!
//! This module parses ZecWallet Lite wallet files, extracting key data such as
//! wallet version, keys, and other account-related information.
//!
//! ## Overview
//! The ZecWallet Lite parser reads data from the `zecwallet-lite.dat` file. The data
//! is written and read linearly using a `BufReader`/`BufWriter`.
//!
//! ### Data Read (in order):
//! - **Wallet Version**: The version of the wallet file.
//! - **Wallet Keys**: Keys associated with the wallet.
//! - **Other Data**: Currently not parsed.
//!
//! ## Caveats
//! - **Wallet Birthday**: Due to the linear and variable nature of the data storage,
//!   it is not possible to directly access certain pieces of data using file offsets.
//!   The wallet birthday is located after some data that this parser does not read,
//!   owing to complexity and incompatibility with newer `librustzcash` versions.
//! - **Encrypted Wallets**: Encrypted wallet files are not supported by this parser.
//!
//! ## Implementation Details
//! - ZecWallet Lite keeps an internal count for derived accounts, adhering to ZIP 32.
//!   It will always derive the first child (`ChildIndex 0`) for different accounts.
//! - Since the `ChildIndex` is fixed and only the account changes, this parser groups
//!   addresses derived from the same account. For instance:
//!   - If the wallet contains 1 Orchard address, 2 Sapling addresses, and 2 Transparent addresses,
//!     the exported wallet will have 2 accounts:
//!     1. The first account containing all keys.
//!     2. The second account containing only Sapling and Transparent keys.
//!
//```

pub(crate) mod block;
pub(crate) mod data;
pub(crate) mod keys;
pub(crate) mod orchard_data;
pub(crate) mod sapling_data;
pub(crate) mod transactions;
pub(crate) mod walletokey;
pub(crate) mod wallettkey;
pub(crate) mod walletzkey;

use crate::{WalletKeyType, WalletParser};

use bip0039::{English, Mnemonic};

use block::CompactBlockData;
use bridgetree::{Address, BridgeTree, Checkpoint, Hashable, Level, MerkleBridge, Position};
use data::{WalletOptions, WalletZecPriceInfo};
use keys::Keys;
// use data::BlockData;
// use wallet_txns::WalletTxns;

use orchard::{keys::SpendingKey, zip32::ChildIndex};
use sapling::zip32::ExtendedSpendingKey;
use transactions::WalletTxns;
use zcash_client_backend::{encoding::encode_transparent_address, proto::service::TreeState};
use zcash_encoding::{Optional, Vector};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::encode_payment_address,
    keys::{UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{
    consensus::{BlockHeight, MainNetwork},
    constants::mainnet::{
        B58_PUBKEY_ADDRESS_PREFIX, B58_SCRIPT_ADDRESS_PREFIX, HRP_SAPLING_PAYMENT_ADDRESS,
    },
    legacy::keys::{AccountPrivKey, IncomingViewingKey, NonHardenedChildIndex},
    merkle_tree::{
        HashSer, read_address, read_leu64_usize, read_nonempty_frontier_v1, read_position,
    },
    zip32::AccountId,
};

use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs::File,
    io::{self, BufReader, ErrorKind},
};
// use zcash_encoding::Vector;
#[derive(Debug, Clone)]
pub struct ZecWalletLite {
    pub version: u64,
    pub keys: Keys,
    pub blocks: Vec<CompactBlockData>,
    pub transactions: WalletTxns,
    pub chain_name: String,
    pub wallet_options: WalletOptions,
    pub birthday: u64,
    pub verified_tree: Option<TreeState>,
    pub price_info: WalletZecPriceInfo,
}

impl ZecWalletLite {
    fn serialized_version() -> u64 {
        25
    }

    fn get_wallet_keys(&self, idx: usize) -> io::Result<crate::WalletKeys> {
        // construct a WalletTKey assosiated with hd index `idx`
        let tkeys: Vec<crate::WalletTKey> = self
            .keys
            .tkeys
            .clone()
            .iter()
            .enumerate()
            .filter(|&(_, k)| idx as u32 == k.hdkey_num.unwrap_or(u32::MAX))
            .map(|(_, t)| {
                let key_type = match t.keytype {
                    wallettkey::WalletTKeyType::HdKey => crate::WalletKeyType::HdDerived,
                    wallettkey::WalletTKeyType::ImportedKey => crate::WalletKeyType::Imported,
                };

                crate::WalletTKey {
                    pk: t.key.unwrap(),
                    key_type,
                    index: t.hdkey_num.unwrap_or(0),
                    address: t.address.clone(),
                }
            })
            .collect::<Vec<_>>();

        // construct a WalletZKey assosiated with hd index `idx`
        let zkeys: Vec<crate::WalletZKey> = self
            .keys
            .zkeys
            .iter()
            .enumerate()
            .filter(|&(_, k)| idx as u32 == k.hdkey_num.unwrap_or(u32::MAX))
            .map(|(_, z)| {
                let key_type = match z.keytype {
                    walletzkey::WalletZKeyType::HdKey => crate::WalletKeyType::HdDerived,
                    walletzkey::WalletZKeyType::ImportedSpendingKey
                    | walletzkey::WalletZKeyType::ImportedViewKey => crate::WalletKeyType::Imported,
                };

                let extsk = z.extsk.clone().unwrap();
                let fvk = z.clone().extfvk;
                let index = z.hdkey_num.unwrap_or(0);
                let address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &z.zaddress);

                crate::WalletZKey {
                    extsk: Some(extsk),
                    fvk,
                    key_type,
                    index,
                    address,
                }
            })
            .collect::<Vec<_>>();

        // construct a WalletOKey assosiated with hd index `idx`
        let okeys: Vec<crate::WalletOKey> = self
            .keys
            .okeys
            .iter()
            .enumerate()
            .filter(|&(_, k)| idx as u32 == k.hdkey_num.unwrap_or(u32::MAX))
            .map(|(_, o)| {
                let key_type = match o.keytype {
                    walletokey::WalletOKeyType::HdKey => crate::WalletKeyType::HdDerived,
                    walletokey::WalletOKeyType::ImportedSpendingKey
                    | walletokey::WalletOKeyType::ImportedFullViewKey => {
                        crate::WalletKeyType::Imported
                    }
                };

                let sk = o.sk.unwrap();
                let fvk = o.clone().fvk;
                let address = o.unified_address.encode(&MainNetwork);

                let index = o.hdkey_num.unwrap_or(0);

                crate::WalletOKey {
                    sk: Some(sk),
                    fvk: Some(fvk),
                    key_type,
                    index,
                    address,
                }
            })
            .collect::<Vec<_>>();
        Ok(crate::WalletKeys {
            tkeys: tkeys.first().cloned(),
            zkeys: zkeys.first().cloned(),
            okeys: okeys.first().cloned(),
        })
    }

    pub fn get_ufvk_for_account(&self, id: u32) -> io::Result<UnifiedFullViewingKey> {
        let seed_entropy = self.keys.seed;
        let mnemonic = <Mnemonic<English>>::from_entropy(seed_entropy).unwrap();
        let seed_bytes = mnemonic.to_seed("");
        let usk = UnifiedSpendingKey::from_seed(
            &MainNetwork,
            &seed_bytes,
            AccountId::try_from(id).expect("Invalid AccountId"),
        )
        .map_err(|_| "Unable to create UnifiedSpendingKey from seed.")
        .unwrap();

        let ufvk = usk.to_unified_full_viewing_key();
        Ok(ufvk)
    }

    #[allow(deprecated)]
    pub fn from_seed_phrase(phrase: &str, num_addr: u32) -> io::Result<crate::Wallet> {
        let mnemonic = <Mnemonic<English>>::from_phrase(phrase).expect("Invalid mnemonic phrase");
        let seed = mnemonic.to_seed("");

        let mut accounts = vec![];

        // derive sapling addresses
        for hdkey_num in 0..num_addr {
            // derive extsk
            let extsk = ExtendedSpendingKey::master(&seed);

            let (_, addr) = extsk
                .clone()
                .derive_child(ChildIndex::hardened(32))
                .derive_child(ChildIndex::hardened(133))
                .derive_child(ChildIndex::hardened(hdkey_num))
                .default_address();

            let fvk = extsk.to_extended_full_viewing_key();
            let z_address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &addr);

            let zkeys = crate::WalletZKey {
                extsk: Some(extsk),
                fvk,
                key_type: crate::WalletKeyType::HdDerived,
                index: hdkey_num,
                address: z_address,
            };

            // derive orchard addresses
            let sk = SpendingKey::from_zip32_seed(
                &seed,
                133,
                AccountId::try_from(hdkey_num).expect("invalid account id"),
            )
            .expect("invalid zip32 seed");
            let fvk = orchard::keys::FullViewingKey::from(&sk);
            let oaddr = fvk.address_at(0u64, orchard::keys::Scope::External);
            let o_address = UnifiedAddress::from_receivers(Some(oaddr), None, None)
                .expect("Invalud unified address");

            let okeys = crate::WalletOKey {
                sk: Some(sk),
                fvk: Some(fvk),
                key_type: crate::WalletKeyType::HdDerived,
                index: hdkey_num,
                address: o_address.encode(&MainNetwork),
            };

            // Derive transparent addresses
            let priv_key = AccountPrivKey::from_seed(
                &MainNetwork,
                &seed,
                AccountId::try_from(0).expect("invalid account id"),
            )
            .expect("Invalid zip32 seed");

            let pk = priv_key
                .derive_external_secret_key(
                    NonHardenedChildIndex::from_index(hdkey_num).expect("Invalid index"),
                )
                .expect("Invalid secret key");

            let taddy = priv_key
                .to_account_pubkey()
                .derive_external_ivk()
                .expect("Invalid pubkey")
                .derive_address(
                    NonHardenedChildIndex::from_index(hdkey_num).expect("Invalid index"),
                )
                .expect("Invalid transparent address.");

            let t_address = encode_transparent_address(
                &B58_PUBKEY_ADDRESS_PREFIX,
                &B58_SCRIPT_ADDRESS_PREFIX,
                &taddy,
            );

            let tkeys = crate::WalletTKey {
                pk,
                key_type: WalletKeyType::HdDerived,
                index: hdkey_num,
                address: t_address,
            };

            accounts.push(crate::WalletAccount {
                name: format!("Account {}", hdkey_num + 1),
                seed: Some(seed.to_vec()),
                birthday: BlockHeight::from_u32(0),
                keys: crate::WalletKeys {
                    tkeys: Some(tkeys),
                    zkeys: Some(zkeys),
                    okeys: Some(okeys),
                },
            })
        }

        Ok(crate::Wallet {
            wallet_name: "ZecWalletLite".to_string(),
            version: 25,
            accounts,
        })
    }

    pub fn read_string<R: ReadBytesExt>(mut reader: R) -> io::Result<String> {
        // Strings are written as <littleendian> len + bytes
        let str_len = reader.read_u64::<LittleEndian>()?;
        let mut str_bytes = vec![0; str_len as usize];
        reader.read_exact(&mut str_bytes)?;

        let str = String::from_utf8(str_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(str)
    }

    /// Reads a [`BridgeTree`] value from its serialized form.
    ///
    /// [`BridgeTree`] values are expected to have been serialized with a leading version byte. Parsing
    /// behavior varies slightly based upon the serialization version.
    ///
    /// SER_V1 checkpoint serialization encoded checkpoint data from the `Checkpoint` type as defined
    /// in `incrementalmerkletree` version `0.3.0-beta-2`. This version was only used in testnet
    /// wallets prior to NU5 launch. Reading `SER_V1` checkpoint data is not supported.
    ///
    /// Checkpoint identifiers are `u32` values which for `SER_V3` serialization correspond to block
    /// heights; checkpoint identifiers were not present in `SER_V2` serialization, so when reading
    /// such data the returned identifiers will *not* correspond to block heights. As such, checkpoint
    /// ids should always be treated as opaque, totally ordered identifiers without additional
    /// semantics.
    #[allow(clippy::needless_borrows_for_generic_args)]
    #[allow(clippy::redundant_closure)]
    pub fn read_tree<H: Hashable + HashSer + Ord + Clone, R: ReadBytesExt>(
        mut reader: R,
    ) -> io::Result<BridgeTree<H, u32, 32>> {
        let _version = reader.read_u64::<LittleEndian>()?;

        // TODO: Add tree version check
        let prior_bridges = Vector::read(&mut reader, |r| ZecWalletLite::read_bridge_v1(r))?;
        let current_bridge = Optional::read(&mut reader, |r| ZecWalletLite::read_bridge_v1(r))?;
        let saved = Vector::read_collected(&mut reader, |mut r| {
            Ok((read_position(&mut r)?, read_leu64_usize(&mut r)?))
        })?;

        let fake_checkpoint_id = 0u32;
        let checkpoints = Vector::read_collected(&mut reader, |r| {
            ZecWalletLite::read_checkpoint_v2(r, fake_checkpoint_id)
        })?;
        let max_checkpoints = read_leu64_usize(&mut reader)?;

        BridgeTree::from_parts(
            prior_bridges,
            current_bridge,
            saved,
            checkpoints,
            max_checkpoints,
        )
        .map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Consistency violation found when attempting to deserialize Merkle tree: {:?}",
                    err
                ),
            )
        })
    }

    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn read_bridge_v1<H: HashSer + Ord + Clone, R: ReadBytesExt>(
        mut reader: R,
    ) -> io::Result<MerkleBridge<H>> {
        fn levels_required(pos: Position) -> impl Iterator<Item = Level> {
            (0u8..64).filter_map(move |i| {
                if u64::from(pos) == 0 || u64::from(pos) & (1 << i) == 0 {
                    Some(Level::from(i))
                } else {
                    None
                }
            })
        }

        let prior_position = Optional::read(&mut reader, read_position)?;

        let fragments = Vector::read(&mut reader, |mut r| {
            let fragment_position = read_position(&mut r)?;
            let (pos, levels_observed, values) = ZecWalletLite::read_auth_fragment_v1(r)?;

            if fragment_position == pos {
                Ok((pos, levels_observed, values))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Auth fragment position mismatch: {:?} != {:?}",
                        fragment_position, pos
                    ),
                ))
            }
        })?;

        let frontier = read_nonempty_frontier_v1(&mut reader)?;
        let mut tracking = BTreeSet::new();
        let mut ommers = BTreeMap::new();
        for (pos, levels_observed, values) in fragments.into_iter() {
            // get the list of levels at which we expect to find future ommers for the position being
            // tracked
            let levels = levels_required(pos)
                .take(levels_observed + 1)
                .collect::<Vec<_>>();

            // track the currently-incomplete parent of the tracked position at max height (the one
            // we're currently building)
            tracking.insert(Address::above_position(*levels.last().unwrap(), pos));

            for (level, ommer_value) in levels
                .into_iter()
                .rev()
                .skip(1)
                .zip(values.into_iter().rev())
            {
                let ommer_address = Address::above_position(level, pos).sibling();
                ommers.insert(ommer_address, ommer_value);
            }
        }

        Ok(MerkleBridge::from_parts(
            prior_position,
            tracking,
            ommers,
            frontier,
        ))
    }

    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn read_bridge_v2<H: HashSer + Ord + Clone, R: ReadBytesExt>(
        mut reader: R,
    ) -> io::Result<MerkleBridge<H>> {
        let prior_position = Optional::read(&mut reader, read_position)?;
        let tracking = Vector::read_collected(&mut reader, |r| read_address(r))?;
        let ommers = Vector::read_collected(&mut reader, |mut r| {
            let addr = read_address(&mut r)?;
            let value = H::read(&mut r)?;
            Ok((addr, value))
        })?;
        let frontier = read_nonempty_frontier_v1(&mut reader)?;

        Ok(MerkleBridge::from_parts(
            prior_position,
            tracking,
            ommers,
            frontier,
        ))
    }

    /// Reads part of the information required to part of a construct a `bridgetree` version `0.3.0`
    /// [`MerkleBridge`] as encoded from the `incrementalmerkletree` version `0.3.0` version of the
    /// `AuthFragment` data structure.
    #[allow(clippy::redundant_closure)]
    pub fn read_auth_fragment_v1<H: HashSer, R: ReadBytesExt>(
        mut reader: R,
    ) -> io::Result<(bridgetree::Position, usize, Vec<H>)> {
        let position = read_position(&mut reader)?;
        let alts_observed = read_leu64_usize(&mut reader)?;
        let values = Vector::read(&mut reader, |r| H::read(r))?;

        Ok((position, alts_observed, values))
    }

    /// Reads a [`bridgetree::Checkpoint`] as encoded from the `incrementalmerkletree` version `0.3.0`
    /// version of the data structure.
    ///
    /// The v2 checkpoint serialization does not include any sort of checkpoint identifier. Under
    /// ordinary circumstances, the checkpoint ID will be the block height at which the checkpoint was
    /// created, but since we don't have any source for this information, we require the caller to
    /// provide it; any unique identifier will do so long as the identifiers are ordered correctly.
    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn read_checkpoint_v2<R: ReadBytesExt>(
        mut reader: R,
        checkpoint_id: u32,
    ) -> io::Result<Checkpoint<u32>> {
        let bridges_len = read_leu64_usize(&mut reader)?;
        let _ = reader.read_u8()? == 1; // legacy is_marked flag
        let marked = Vector::read_collected(&mut reader, |r| read_position(r))?;
        let forgotten = Vector::read_collected(&mut reader, |mut r| {
            let pos = read_position(&mut r)?;
            let _ = read_leu64_usize(&mut r)?;
            Ok(pos)
        })?;

        Ok(Checkpoint::from_parts(
            checkpoint_id,
            bridges_len,
            marked,
            forgotten,
        ))
    }
}

impl WalletParser for ZecWalletLite {
    fn read(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)
            .map_err(|e| format!("Can't open file {}", e))
            .unwrap();

        let mut reader = BufReader::new(file);

        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            let e = format!(
                "Don't know how to read wallet version {}. Do you have the latest version?",
                version
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }

        // TODO: read old versions of wallet file
        let keys = Keys::read(&mut reader)?;

        let blocks = Vector::read(&mut reader, |r| CompactBlockData::read(r))?;
        // TODO: read old versions of wallet file

        let txns = WalletTxns::read(&mut reader)?;

        let chain_name = ZecWalletLite::read_string(&mut reader)?;

        let wallet_options = WalletOptions::read(&mut reader)?;

        let birthday = reader.read_u64::<LittleEndian>()?;

        let verified_tree = Optional::read(&mut reader, |r| {
            use prost::Message;

            let buf = Vector::read(r, |r| r.read_u8())?;
            TreeState::decode(&buf[..])
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("Read Error: {}", e)))
        })
        .unwrap(); // TODO: Add proper error handling

        let price_info = WalletZecPriceInfo::read(&mut reader)?;

        // TODO: orchard witnesses

        Ok(Self {
            version,
            keys,
            blocks,
            transactions: txns,
            chain_name,
            wallet_options,
            birthday,
            verified_tree,
            price_info,
        })
    }

    fn get_wallet_name(&self) -> String {
        "ZecWalletLite".to_string()
    }

    fn get_wallet_version(&self) -> u64 {
        self.version
    }

    // fn get_wallet_seed(&self) -> String {
    //     let seed_entropy = self.keys.seed;
    //     let seed = <Mnemonic<English>>::from_entropy(seed_entropy).expect("Invalid seed entropy");
    //     let phrase = seed.phrase();
    //     phrase.to_string()
    // }

    fn get_wallet_accounts(&self) -> io::Result<Vec<crate::WalletAccount>> {
        let tkeys_last_index = self.keys.tkeys.len();
        let zkeys_last_index = self.keys.zkeys.len();
        let okeys_last_index = self.keys.okeys.len();

        let mut accounts: Vec<crate::WalletAccount> = vec![];

        let last_index = std::cmp::max(
            tkeys_last_index,
            std::cmp::max(zkeys_last_index, okeys_last_index),
        );
        for i in 0..last_index {
            let keys = self.get_wallet_keys(i)?;

            // let ufvk = self.get_ufvk_for_account(0u32)?;
            // let t = ufvk
            //     .transparent().unwrap()
            //     .derive_external_ivk().unwrap()
            //     .derive_address(NonHardenedChildIndex::from_index(i as u32).expect("Invalid NonHardenedChildIndex"))
            //     .map_err(|_|"Invalid address")
            //     .unwrap();

            // let taddy = zcash_keys::encoding::encode_transparent_address_p(&MainNetwork, &t);
            // println!("{}", taddy);

            accounts.push(crate::WalletAccount {
                name: format!("Account {}", i + 1),
                seed: Some(self.keys.seed.to_vec()),
                // ufvk: Some(ufvk),
                birthday: BlockHeight::from_u32(0),
                keys,
            })
        }

        Ok(accounts)
    }

    fn print_internal(&self) {
        println!("ZecWalletLite");
        println!("{}", self);
    }

    fn get_wallet_seed(&self) -> [u8; 32] {
        return self.keys.seed;
    }

    fn get_birthday(&self) -> u64 {
        return self.birthday;
    }
}

impl Display for ZecWalletLite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Wallet Version: {}", self.version).unwrap();
        writeln!(f, "{}", self.keys).unwrap();

        // Blocks

        writeln!(f, "Blocks found: {}", self.blocks.len()).unwrap();

        // TODO: This should be moved into a wrapper struct
        // for block in &self.blocks {
        //     writeln!(f, "{}", block).unwrap();
        // }

        writeln!(f, "{}", self.transactions).unwrap();

        writeln!(f, "Chain name: {}", self.chain_name).unwrap();

        writeln!(f, "Wallet Options: {}", self.wallet_options).unwrap();

        writeln!(f, "Birthday: {}", self.birthday).unwrap();

        match &self.verified_tree {
            Some(tree) => {
                writeln!(f, ">> Verified Tree <<").unwrap();
                writeln!(f, "> Hash: {}", tree.hash).unwrap();
                writeln!(f, "> Height: {}", tree.height).unwrap();
                writeln!(f, "> Time: {}", tree.time).unwrap();

                // We may need to hide these under the `-v` flag
                writeln!(f, "> Sapling Tree: {}", tree.sapling_tree).unwrap();
                writeln!(f, "> Orchard Tree: {}", tree.orchard_tree).unwrap();
            }
            None => {
                writeln!(f, "Verified Tree: None").unwrap();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip0039::{English, Mnemonic};

    fn get_wallet() -> ZecWalletLite {
        ZecWalletLite::read("../zecwallet-light-wallet.dat")
            .map_err(|e| format!("Error parsing wallet {}", e))
            .unwrap()
    }

    #[test]
    fn test_zwl_version() {
        let wallet = get_wallet();
        assert!(wallet.version > 0);
    }

    #[test]
    fn test_zwl_seed() {
        let wallet = get_wallet();
        let seed_entropy = wallet.keys.seed;
        let seed = <Mnemonic<English>>::from_entropy(seed_entropy).expect("Invalid seed entropy");
        let phrase = seed.phrase();
        assert_eq!(
            phrase,
            "clerk family rack dragon cannon wait vendor penalty absent country better coast expand true middle stable assist clerk tent phone toilet knee female kitchen"
        );
    }

    #[test]
    fn test_zwl_transactions() {
        let wallet = get_wallet();
        assert_eq!(wallet.transactions.current.len(), 0);
    }
}

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

pub(crate) mod keys;
pub(crate) mod walletokey;
pub(crate) mod wallettkey;
pub(crate) mod walletzkey;
// pub (crate)mod data;
// pub (crate)mod wallet_txns;

use crate::{WalletKeyType, WalletParser};

use bip0039::{English, Mnemonic};

use keys::Keys;
// use data::BlockData;
// use wallet_txns::WalletTxns;

use orchard::{keys::SpendingKey, zip32::ChildIndex};
use sapling::zip32::ExtendedSpendingKey;
use zcash_client_backend::encoding::encode_transparent_address;
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
    // legacy::keys::{IncomingViewingKey, NonHardenedChildIndex}
    zip32::AccountId,
};
// use zcash_primitives::legacy::keys::ExternalIvk;

use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{self, BufReader},
};
// use zcash_encoding::Vector;

pub struct ZecWalletLite {
    pub version: u64,
    pub keys: Keys,
    // pub blocks: Vec<BlockData>
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
                // let key_type = match t.keytype {
                //     wallettkey::WalletTKeyType::HdKey => crate::WalletKeyType::HdKey,
                //     wallettkey::WalletTKeyType::ImportedKey => crate::WalletKeyType::ImportedPrivateKey,
                // };

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
                // let key_type = match z.keytype {
                //     walletzkey::WalletZKeyType::HdKey => crate::WalletKeyType::HdKey,
                //     walletzkey::WalletZKeyType::ImportedSpendingKey => crate::WalletKeyType::ImportedExtsk,
                //     walletzkey::WalletZKeyType::ImportedViewKey => crate::WalletKeyType::ImportedViewKey
                // };

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
                // let key_type = match o.keytype {
                //     walletokey::WalletOKeyType::HdKey => crate::WalletKeyType::HdKey,
                //     walletokey::WalletOKeyType::ImportedSpendingKey => crate::WalletKeyType::ImportedSpendingKey,
                //     walletokey::WalletOKeyType::ImportedFullViewKey => crate::WalletKeyType::ImportedFvk,
                // };

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

        // let blocks = Vector::read(&mut reader, |r| BlockData::read(r))?;
        // TODO: read old versions of wallet file

        // let txns = WalletTxns::read(&mut reader)?;

        Ok(Self {
            version,
            keys,
            // blocks
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
        assert_eq!(phrase, "clerk family rack dragon cannon wait vendor penalty absent country better coast expand true middle stable assist clerk tent phone toilet knee female kitchen");
    }
}

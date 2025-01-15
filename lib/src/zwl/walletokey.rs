use byteorder::{LittleEndian, ReadBytesExt};
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use std::io::{self, Read};
use zcash_encoding::{Optional, Vector};
use zcash_keys::address::UnifiedAddress;

#[derive(PartialEq, Debug, Clone)]
pub enum WalletOKeyType {
    HdKey = 0,
    ImportedSpendingKey = 1,
    ImportedFullViewKey = 2,
}

#[derive(Clone, Debug)]
pub struct WalletOKey {
    pub locked: bool,

    pub keytype: WalletOKeyType,
    pub sk: Option<SpendingKey>,
    pub fvk: FullViewingKey,
    pub unified_address: UnifiedAddress,

    // If this is a HD key, what is the key number
    pub hdkey_num: Option<u32>,

    // If locked, the encrypted private key is stored here
    pub enc_key: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
}

impl WalletOKey {
    pub fn serialized_version() -> u8 {
        1
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u8()?;
        assert!(version <= Self::serialized_version());

        // Read orchard key type
        let keytype = match reader.read_u32::<LittleEndian>()? {
            0 => Ok(WalletOKeyType::HdKey),
            1 => Ok(WalletOKeyType::ImportedSpendingKey),
            2 => Ok(WalletOKeyType::ImportedFullViewKey),
            n => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid okey type {}", n),
            )),
        }?;

        // read if key is locked
        let locked = reader.read_u8()? > 0;

        // If HD derived, read the key index
        let hdkey_num = Optional::read(&mut reader, |r| r.read_u32::<LittleEndian>())?;

        // read address fvk
        let fvk = FullViewingKey::read(&mut reader)?;

        // read sk if available (read as 32 bytes)
        let sk = Optional::read(&mut reader, |r| {
            let mut bytes = [0u8; 32];
            r.read_exact(&mut bytes)?;
            Ok(SpendingKey::from_bytes(bytes).unwrap())
        })?;

        // Derive unified address (orchard only) from fvk
        let address = fvk.address_at(0u64, Scope::External);
        let unified_address = UnifiedAddress::from_receivers(Some(address), None, None)
            .expect("Failed to construct unified address");

        // read "possible" encrypted key
        let enc_key = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        // read "possible" nouce used in key encryption
        let nonce = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        Ok(Self {
            locked,
            keytype,
            sk,
            fvk,
            unified_address,
            hdkey_num,
            enc_key,
            nonce,
        })
    }
}

use std::fmt;
use std::io::{self, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use sapling::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
use sapling::PaymentAddress;
use zcash_encoding::{Optional, Vector};

#[derive(PartialEq, Debug, Clone)]
pub enum WalletZKeyType {
    HdKey = 0,
    ImportedSpendingKey = 1,
    ImportedViewKey = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletZKey {
    pub keytype: WalletZKeyType,
    locked: bool,
    pub extsk: Option<ExtendedSpendingKey>,
    pub extfvk: ExtendedFullViewingKey,
    pub zaddress: PaymentAddress,

    // If this is a HD key, what is the key number
    pub hdkey_num: Option<u32>,

    // If locked, the encrypted private key is stored here
    pub enc_key: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
}

impl WalletZKey {
    pub fn serialized_version() -> u8 {
        1
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u8()?;
        assert!(version <= Self::serialized_version());

        // read type of the key
        let keytype: WalletZKeyType = match reader.read_u32::<LittleEndian>()? {
            0 => Ok(WalletZKeyType::HdKey),
            1 => Ok(WalletZKeyType::ImportedSpendingKey),
            2 => Ok(WalletZKeyType::ImportedViewKey),
            n => Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("Unknown zkey type {}", n),
            )),
        }?;

        // read if address is locked
        let locked = reader.read_u8()? > 0;

        // read address extsk
        let extsk = Optional::read(&mut reader, ExtendedSpendingKey::read)?;

        // read address extfvk
        let extfvk = ExtendedFullViewingKey::read(&mut reader)?;

        // derive zaddress from extfvk
        let (_, zaddress) = extfvk.default_address();

        // If HD derived, read the key index
        let hdkey_num = Optional::read(&mut reader, |r| r.read_u32::<LittleEndian>())?;

        // read "possible" encrypted key
        let enc_key = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        // read ""possible" nounce used for encryption
        let nonce = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        Ok(Self {
            keytype,
            locked,
            extsk,
            extfvk,
            zaddress,
            hdkey_num,
            enc_key,
            nonce,
        })
    }
}

impl fmt::Display for WalletZKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.keytype {
            WalletZKeyType::HdKey => {
                writeln!(f, "Type: HD key").unwrap();
            }
            WalletZKeyType::ImportedSpendingKey => {
                writeln!(f, "Type: Imported spending key").unwrap();
            }
            WalletZKeyType::ImportedViewKey => {
                writeln!(f, "Type: Imported view key").unwrap();
            }
            _ => {
                writeln!(f, "Type: Unknown").unwrap();
            }
        }

        match self.locked {
            true => {
                writeln!(f, "Status: Encrypted").unwrap();
            }
            false => {
                writeln!(f, "Status: Decrypted").unwrap();
            }
        }

        if let Some(extsk) = &self.extsk {
            writeln!(f, "{:?}", extsk).unwrap();
        }

        writeln!(f, "{:?}", self.extfvk).unwrap();

        writeln!(f, "{:?}", self.zaddress).unwrap();

        Ok(())
    }
}

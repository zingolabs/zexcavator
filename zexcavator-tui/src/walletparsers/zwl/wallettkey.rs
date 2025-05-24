use byteorder::{LittleEndian, ReadBytesExt};
use secp256k1::SecretKey;
use std::{
    fmt,
    io::{self, Read},
};
use zcash_encoding::{Optional, Vector};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WalletTKeyType {
    HdKey = 0,
    ImportedKey = 1,
}

#[derive(Debug, Clone)]
pub struct WalletTKey {
    pub keytype: WalletTKeyType,
    pub locked: bool,
    pub key: Option<secp256k1::SecretKey>,
    pub address: String,

    // If this is a HD key, what is the key number
    pub hdkey_num: Option<u32>,

    // If locked, the encrypted private key is stored here
    pub enc_key: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
}

impl WalletTKey {
    fn serialized_version() -> u8 {
        1
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u8()?;
        assert!(version <= Self::serialized_version());

        // read type of the key
        let keytype: WalletTKeyType = match reader.read_u32::<LittleEndian>()? {
            0 => Ok(WalletTKeyType::HdKey),
            1 => Ok(WalletTKeyType::ImportedKey),
            n => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown tkey type: {}", n),
            )),
        }?;

        // read if address is locked
        let locked = reader.read_u8()? > 0;

        let key = Optional::read(&mut reader, |r| {
            let mut tpk_bytes = [0u8; 32];
            r.read_exact(&mut tpk_bytes)?;
            SecretKey::from_slice(&tpk_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })?;

        // read encoded t address as String
        // Strings are written as <littleendian> len + bytes
        let str_len = reader.read_u64::<LittleEndian>()?;
        let mut str_bytes = vec![0; str_len as usize];
        reader.read_exact(&mut str_bytes)?;

        // The actual encoded address
        let address = String::from_utf8(str_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // If HD derived, read the key index
        let hdkey_num = Optional::read(&mut reader, |r| r.read_u32::<LittleEndian>())?;

        // read "possible" encrypted key
        let enc_key = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        // read ""possible" nounce used for encryption
        let nonce = Optional::read(&mut reader, |r| Vector::read(r, |r| r.read_u8()))?;

        Ok(Self {
            keytype,
            locked,
            key,
            hdkey_num,
            enc_key,
            nonce,
            address,
        })
    }
}

impl fmt::Display for WalletTKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.keytype {
            WalletTKeyType::HdKey => {
                writeln!(f, "Type: HD key").unwrap();
            }
            WalletTKeyType::ImportedKey => {
                writeln!(f, "Type: Imported key").unwrap();
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

        if let Some(private_key) = &self.key {
            writeln!(
                f,
                "Private key: {}",
                hex::encode(private_key.secret_bytes())
            )
            .unwrap();
        }

        writeln!(f, "Address: {}", self.address).unwrap();

        Ok(())
    }
}

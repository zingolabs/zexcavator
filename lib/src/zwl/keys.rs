use std::io::{Read, self};
use byteorder::{ReadBytesExt, LittleEndian};
use zcash_encoding::Vector;

use crate::zwl::walletokey::WalletOKey;
use crate::zwl::walletzkey::WalletZKey;
use crate::zwl::wallettkey::WalletTKey;

#[derive(Debug, Clone)]
pub struct Keys {
    // Is the wallet encrypted? If it is, then when writing to disk, the seed is always encrypted
    // and the individual spending keys are not written
    pub encrypted: bool,

    pub enc_seed: [u8; 48], // If locked, this contains the encrypted seed
    pub nonce: Vec<u8>,     // Nonce used to encrypt the wallet.

    pub seed: [u8; 32], // Seed phrase for this wallet. If wallet is locked, this is 0

    // List of keys, actually in this wallet. This is a combination of HD keys derived from the seed,
    // viewing keys and imported spending keys.
    pub zkeys: Vec<WalletZKey>,

    // Transparent keys. If the wallet is locked, then the secret keys will be encrypted,
    // but the addresses will be present. This Vec contains both wallet and imported tkeys
    pub tkeys: Vec<WalletTKey>,

    // Unified address (Orchard) keys actually in this wallet.
    // If wallet is locked, only viewing keys are present.
    pub okeys: Vec<WalletOKey>,
}

impl Keys {
    pub fn serialized_version() -> u64 {
        22
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            let e = format!(
                "Don't know how to read wallet version {}. Do you have the latest version?",
                version
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }

        // Read if wallet is encrypted
        let encrypted = reader.read_u8()? > 0;

        // Read "possible" encypted seed
        let mut enc_seed = [0u8; 48];
        reader.read_exact(&mut enc_seed)?;

        // Read nounce used for encyption
        let nonce = Vector::read(&mut reader, |r| r.read_u8())?;

        // Read "possible" clear seed
        let mut seed_bytes = [0u8; 32];
        reader.read_exact(&mut seed_bytes)?;

        // TODO: read old versions of wallet file
        let okeys = Vector::read(&mut reader, |r| WalletOKey::read(r))?;

        // TODO: read old versions of wallet file
        let zkeys = Vector::read(&mut reader, |r| WalletZKey::read(r))?;

        // read wallet tkeys
        let tkeys = Vector::read(&mut reader, |r| WalletTKey::read(r))?;

        Ok(
            Self {
                encrypted,
                enc_seed,
                nonce,
                seed: seed_bytes,
                zkeys,
                tkeys,
                okeys
            }
        )
    }
}
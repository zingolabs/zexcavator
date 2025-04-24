use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt::Display;
use std::io::{self, Read};
use zcash_encoding::Vector;

use crate::zwl::walletokey::WalletOKey;
use crate::zwl::wallettkey::WalletTKey;
use crate::zwl::walletzkey::WalletZKey;

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

        Ok(Self {
            encrypted,
            enc_seed,
            nonce,
            seed: seed_bytes,
            zkeys,
            tkeys,
            okeys,
        })
    }
}

impl Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ">> Keys << ").unwrap();
        writeln!(f, "Version: {}", Keys::serialized_version()).unwrap();
        writeln!(f, "Encrypted: {}", self.encrypted).unwrap();

        match self.encrypted {
            true => {
                writeln!(f, "Encrypted seed: {}", hex::encode(self.enc_seed)).unwrap();
                writeln!(f, "Nonce: {}", hex::encode(&self.nonce)).unwrap();
            }
            false => {
                writeln!(f, "Seed: {}", hex::encode(self.seed)).unwrap();
            }
        }

        writeln!(f, "=== ORCHARD ===").unwrap();
        writeln!(f, "Orchard keys found: {}", self.okeys.len()).unwrap();

        for okey in &self.okeys {
            writeln!(f, "{}", okey).unwrap();
        }

        writeln!(f, "=== SAPLING ===").unwrap();
        writeln!(f, "Sapling keys found: {}", self.zkeys.len()).unwrap();

        for zkey in &self.zkeys {
            writeln!(f, "{}", zkey).unwrap();
        }

        writeln!(f, "=== TRANSPARENT ===").unwrap();
        writeln!(f, "Transparent keys found: {}", self.tkeys.len()).unwrap();
        for tkey in &self.tkeys {
            writeln!(f, "{}", tkey).unwrap();
        }
        Ok(())
    }
}

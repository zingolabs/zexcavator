pub (crate)mod keys;
pub (crate)mod walletokey;
pub (crate)mod walletzkey;
pub (crate)mod wallettkey;
pub (crate)mod data;

use crate::WalletParser;

use keys::Keys;
use data::BlockData;

use std::{io::{self, BufReader}, fs::File};
use byteorder::{ReadBytesExt, LittleEndian};
use zcash_encoding::Vector;

use bip0039::{Mnemonic, English};

pub struct ZecWalletLite {
    pub version: u64,
    pub keys: Keys,
    pub blocks: Vec<BlockData>
}

impl ZecWalletLite {
    fn serialized_version() -> u64 {
        return 25;
    }
}

impl WalletParser for ZecWalletLite {     
    fn read(filename: &str) -> io::Result<Self>{
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

        let blocks = Vector::read(&mut reader, |r| BlockData::read(r))?;

        Ok(
            Self {                  
                version, 
                keys,
                blocks                 
             }
        )
    }

    fn wallet_name(&self) -> String {
        "ZecWalletLite".to_string()
    }

    fn wallet_version(&self) -> u64 {
        self.version
    }

    fn wallet_seed(&self) -> String {
        let seed_entropy = self.keys.seed;
        let seed = <Mnemonic<English>>::from_entropy(seed_entropy).expect("Invalid seed entropy");
        let phrase = seed.phrase();
        phrase.to_string()
    }
}

#[cfg(test)]
mod tests {
    use bip0039::{Mnemonic, English};
    use super::*;  

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
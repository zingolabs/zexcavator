pub (crate)mod keys;
pub (crate)mod walletokey;
pub (crate)mod walletzkey;

use keys::Keys;

use std::io::{self, Read};
use byteorder::{ReadBytesExt, LittleEndian};

pub struct ZecWalletLite {
    pub version: u64,
    pub keys: Keys
}

impl ZecWalletLite {
    pub fn serialized_version() -> u64 {
        return 25;
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<ZecWalletLite>{
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            let e = format!(
                "Don't know how to read wallet version {}. Do you have the latest version?",
                version
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }

        // TODO: read old versions of wallet file
        let keys = Keys::read(reader)?;

        Ok(
            ZecWalletLite {
                version,
                keys
            }
        )
    }
}


#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};
    use bip0039::{Mnemonic, English};
    use super::*;  

    fn get_wallet() -> ZecWalletLite {
        let file = File::open("../zecwallet-light-wallet.dat")
            .map_err(|e| format!("Can't open file {}", e))
            .unwrap();
        let reader = BufReader::new(file);

        ZecWalletLite::read(reader)
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
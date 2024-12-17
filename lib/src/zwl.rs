pub (crate)mod keys;
pub (crate)mod walletokey;
pub (crate)mod walletzkey;
pub (crate)mod wallettkey;

use crate::{WalletParser, Wallet};

use keys::Keys;

use std::{io::{self, BufReader}, fs::File};
use byteorder::{ReadBytesExt, LittleEndian};

pub struct ZecWalletLite {
    inner: Wallet
}

impl ZecWalletLite {
    fn serialized_version() -> u64 {
        return 25;
    }

    fn wallet_name() -> &'static str {
        "ZecWalletLite"
    }
}

impl WalletParser for ZecWalletLite {   
    fn read(filename: &str) -> io::Result<Self>{
        let file = File::open(filename)
            .map_err(|e| format!("Can't open file {}", e))
            .unwrap();

        let mut reader = BufReader::new(file);
        
        let wallet_name = Self::wallet_name();

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
            Self { 
                inner: Wallet { 
                    wallet_name,
                    version, 
                    keys 
                }
             }
        )
    }

    fn inner(&self) -> &Wallet {
        &self.inner
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
        assert!(wallet.inner.version > 0);
    }

    #[test]    
    fn test_zwl_seed() {
        let wallet = get_wallet();
        let seed_entropy = wallet.inner.keys.seed;
        let seed = <Mnemonic<English>>::from_entropy(seed_entropy).expect("Invalid seed entropy");
        let phrase = seed.phrase();
        assert_eq!(phrase, "clerk family rack dragon cannon wait vendor penalty absent country better coast expand true middle stable assist clerk tent phone toilet knee female kitchen");
    }

}
pub mod zwl;

use std::{io::{self, BufReader}, fs::File};

use zwl::{ZecWalletLite, keys::Keys};

#[derive(Debug)]
pub struct Wallet {
    pub version: u64,
    pub keys: Keys
}

impl Wallet {
    pub fn parse(file: &str) -> io::Result<Self> {        
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let wallet = ZecWalletLite::read(reader)
            .map_err(|e| format!("Error: {}", e))
            .unwrap();

        Ok(
            Wallet { 
                version: wallet.version,
                keys: wallet.keys
            }
        )
    }
}
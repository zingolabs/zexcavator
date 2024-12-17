pub mod zwl;

use std::io;

use zwl::keys::Keys;

#[derive(Debug)]
pub struct Wallet {
    pub wallet_name: &'static str,
    pub version: u64,
    pub keys: Keys
}

pub trait WalletParser {
    fn read(filename: &str) -> io::Result<Self> where Self: Sized;
    fn inner(&self) -> &Wallet;
}

impl Wallet {
    pub fn parse<T>(filename: &str) -> io::Result<Self>
    where
        T: WalletParser
    {                
        let wallet = T::read(filename)
            .map_err(|e| format!("Error: {}", e))
            .unwrap();

        Ok(
            Self { 
                wallet_name: wallet.inner().wallet_name,
                version: wallet.inner().version,
                keys: wallet.inner().keys.clone()
            }
        )
    }
}
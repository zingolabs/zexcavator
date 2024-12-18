pub mod zwl;

use std::io;

#[derive(Debug)]
pub struct WalletKeys {

}
#[derive(Debug)]
pub struct Wallet {
    pub wallet_name: String,
    pub version: u64,
    pub seed: String,
    pub birthday: u64,
    pub keys: Vec<WalletKeys>
}

pub trait WalletParser {
    fn read(filename: &str) -> io::Result<Self> where Self: Sized;
    fn wallet_name(&self) -> String;
    fn wallet_version(&self) -> u64;
    fn wallet_seed(&self) -> String;
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
                wallet_name: wallet.wallet_name(),
                version: wallet.wallet_version(),
                seed: wallet.wallet_seed(),
                birthday: 2740940,
                keys: vec![]
            }
        )
    }
}
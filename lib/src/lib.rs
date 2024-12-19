pub mod zwl;

use std::io;

use orchard::keys::{SpendingKey, FullViewingKey};
use sapling::zip32::{ExtendedSpendingKey, ExtendedFullViewingKey};
use zcash_keys::keys::UnifiedFullViewingKey;

#[derive(Debug)]
pub enum WalletKeyType {
    HdKey = 0, // For HD drevied keys
    ImportedExtsk = 1, // For imported sapling extended spending key
    ImportedSpendingKey = 2, // For imported orchard spending key
    ImportedViewKey = 3, // For imported sapling viewing key
    ImportedFvk = 4, // For imported orchard full viewing key
    ImportedUfvk = 5, // For imported unified full viewing key
    ImportedPrivateKey = 6, // For imported transparent private key
}

#[derive(Debug)]
pub struct WalletTKey {
    pub pk: secp256k1::SecretKey,
    pub key_type: WalletKeyType,
    pub index: u32,    
    pub address: String
}

#[derive(Debug)]
pub struct WalletZKey {
    pub extsk: Option<ExtendedSpendingKey>,
    pub fvk: ExtendedFullViewingKey,
    pub key_type: WalletKeyType,
    pub index: u32,    
    pub address: String
}

#[derive(Debug)]
pub struct WalletOKey {
    pub sk: Option<SpendingKey>,
    pub fvk: Option<FullViewingKey>,
    pub ufvk: Option<UnifiedFullViewingKey>,
    pub key_type: WalletKeyType,
    pub index: u32,    
    pub address: String
}

#[derive(Debug)]
pub struct WalletKeys {
    pub tkeys: Option<Vec<WalletTKey>>,
    pub zkeys: Option<Vec<WalletZKey>>,
    pub okeys: Option<Vec<WalletOKey>>
}

#[derive(Debug)]
pub struct Wallet {
    pub wallet_name: String,
    pub version: u64,
    pub seed: String,
    pub birthday: u64,
    pub keys: WalletKeys
}

pub trait WalletParser {
    fn read(filename: &str) -> io::Result<Self> where Self: Sized;
    fn get_wallet_name(&self) -> String;
    fn get_wallet_version(&self) -> u64;
    fn get_wallet_seed(&self) -> String;
    fn get_wallet_keys(&self) -> io::Result<WalletKeys>;
}

impl Wallet {
    pub fn parse<P>(filename: &str) -> io::Result<Self>
    where
        P: WalletParser
    {                
        let wallet = P::read(filename)
            .map_err(|e| format!("Error: {}", e))
            .unwrap();        

        Ok(
            Self { 
                wallet_name: wallet.get_wallet_name(),
                version: wallet.get_wallet_version(),
                seed: wallet.get_wallet_seed(),
                birthday: 2740940,
                keys: wallet.get_wallet_keys()?
            }
        )
    }
}
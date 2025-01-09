pub mod ywallet;
pub mod zingolib;
pub mod zwl;

use std::io;

use orchard::keys::{FullViewingKey, SpendingKey};
use sapling::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
// use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::consensus::BlockHeight;

#[derive(Debug, Clone)]
pub enum WalletKeyType {
    // HdKey = 0, // For HD drevied keys
    // ImportedExtsk = 1, // For imported sapling extended spending key
    // ImportedSpendingKey = 2, // For imported orchard spending key
    // ImportedViewKey = 3, // For imported sapling viewing key
    // ImportedFvk = 4, // For imported orchard full viewing key
    // ImportedUfvk = 5, // For imported unified full viewing key
    // ImportedPrivateKey = 6, // For imported transparent private key
    HdDerived = 0,
    Imported = 1,
}

#[derive(Debug, Clone)]
pub struct WalletTKey {
    pub pk: secp256k1::SecretKey,
    pub key_type: WalletKeyType,
    pub index: u32,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct WalletZKey {
    pub extsk: Option<ExtendedSpendingKey>,
    pub fvk: ExtendedFullViewingKey,
    pub key_type: WalletKeyType,
    pub index: u32,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct WalletOKey {
    pub sk: Option<SpendingKey>,
    pub fvk: Option<FullViewingKey>,
    pub key_type: WalletKeyType,
    pub index: u32,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct WalletKeys {
    pub tkeys: Option<WalletTKey>,
    pub zkeys: Option<WalletZKey>,
    pub okeys: Option<WalletOKey>,
}

#[derive(Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub seed: Option<Vec<u8>>,
    // pub ufvk: Option<UnifiedFullViewingKey>,
    pub birthday: BlockHeight,
    pub keys: WalletKeys,
}

#[derive(Debug)]
pub struct Wallet {
    pub wallet_name: String,
    pub version: u64,
    pub accounts: Vec<WalletAccount>,
}

pub trait WalletParser {
    /// Read the wallet contents from disk
    fn read(filename: &str) -> io::Result<Self>
    where
        Self: Sized;
    fn get_wallet_name(&self) -> String;
    fn get_wallet_version(&self) -> u64;
    // fn get_wallet_seed(&self) -> String;
    fn get_wallet_accounts(&self) -> io::Result<Vec<WalletAccount>>;
}

pub trait WalletWriter {
    fn write(wallet: &Wallet, filename: &str) -> std::io::Result<()>;
}

impl Wallet {
    pub fn parse<P>(filename: &str) -> io::Result<Self>
    where
        P: WalletParser,
    {
        let wallet = P::read(filename)
            .map_err(|e| format!("Error: {}", e))
            .unwrap();

        Ok(Self {
            wallet_name: wallet.get_wallet_name(),
            version: wallet.get_wallet_version(),
            accounts: wallet.get_wallet_accounts()?,
        })
    }

    pub fn write<W>(&self, filename: &str) -> io::Result<()>
    where
        W: WalletWriter,
    {
        let _ = W::write(&self, filename);
        Ok(())
    }
}

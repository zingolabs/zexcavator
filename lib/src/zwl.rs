pub (crate)mod keys;
pub (crate)mod walletokey;
pub (crate)mod walletzkey;
pub (crate)mod wallettkey;
pub (crate)mod data;
// pub (crate)mod wallet_txns;

use crate::WalletParser;

use keys::Keys;
use data::BlockData;
use zcash_keys::{encoding::encode_payment_address, keys::UnifiedFullViewingKey};
use zcash_primitives::{consensus::{MainNetwork}, constants::mainnet::HRP_SAPLING_PAYMENT_ADDRESS};

use std::{io::{self, BufReader}, fs::File};
use byteorder::{ReadBytesExt, LittleEndian};
use zcash_encoding::Vector;

// use sapling::zip32::{ExtendedSpendingKey, ExtendedFullViewingKey};
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

    fn get_wallet_name(&self) -> String {
        "ZecWalletLite".to_string()
    }

    fn get_wallet_version(&self) -> u64 {
        self.version
    }

    fn get_wallet_seed(&self) -> String {
        let seed_entropy = self.keys.seed;
        let seed = <Mnemonic<English>>::from_entropy(seed_entropy).expect("Invalid seed entropy");
        let phrase = seed.phrase();
        phrase.to_string()
    }

    fn get_wallet_keys(&self) -> io::Result<crate::WalletKeys> {
        // construct a vector of tkeys
        let tkeys: Vec<crate::WalletTKey> = self.keys.tkeys
            .iter()
            .map(|t| {
                let key_type = match t.keytype {
                    wallettkey::WalletTKeyType::HdKey => crate::WalletKeyType::HdKey,
                    wallettkey::WalletTKeyType::ImportedKey => crate::WalletKeyType::ImportedPrivateKey
                };

                crate::WalletTKey {
                    pk: t.key.unwrap(),
                    key_type,
                    index: t.hdkey_num.unwrap_or(0),
                    address: t.address.clone(),                    
                }
            })
            .collect();  
        
        // construct a vector of zkeys
        let zkeys: Vec<crate::WalletZKey> = self.keys.zkeys
            .iter()
            .map(|z| {
                let key_type = match z.keytype {
                    walletzkey::WalletZKeyType::HdKey => crate::WalletKeyType::HdKey,
                    walletzkey::WalletZKeyType::ImportedSpendingKey => crate::WalletKeyType::ImportedExtsk,
                    walletzkey::WalletZKeyType::ImportedViewKey => crate::WalletKeyType::ImportedViewKey
                };

                let extsk = z.extsk.clone().unwrap();
                let fvk = z.clone().extfvk;
                let index = z.hdkey_num.unwrap_or(0);
                let address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &z.zaddress);

                crate::WalletZKey {
                    extsk: Some(extsk),
                    fvk,
                    key_type,
                    index,
                    address,
                }
            })
            .collect();

        // construct a vector of okeys
        let okeys: Vec<crate::WalletOKey> = self.keys.okeys
            .iter()
            .map(|o| {
                let key_type = match o.keytype {
                    walletokey::WalletOKeyType::HdKey => crate::WalletKeyType::HdKey,
                    walletokey::WalletOKeyType::ImportedSpendingKey => crate::WalletKeyType::ImportedSpendingKey,
                    walletokey::WalletOKeyType::ImportedFullViewKey => crate::WalletKeyType::ImportedFvk,
                };

                let sk = o.sk.unwrap();
                let fvk = o.clone().fvk;
                let address = o.unified_address.encode(&MainNetwork);

                let index = o.hdkey_num.unwrap_or(0);

                crate::WalletOKey {
                    sk: Some(sk),
                    fvk: Some(fvk),
                    ufvk: None,
                    key_type,
                    index,
                    address,
                }
            })
            .collect();
      Ok(
        crate::WalletKeys {
            tkeys: Some(tkeys),
            zkeys: Some(zkeys),
            okeys: Some(okeys)
        }
      )
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
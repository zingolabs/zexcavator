use std::io;

use crate::{WalletParser, WalletAccount, WalletKeys, WalletZKey, WalletTKey};

pub (crate)mod db;

use bip0039::{Mnemonic, English};
use orchard::keys::FullViewingKey;
use rusqlite::Connection;
use zcash_primitives::consensus::BlockHeight;

pub struct YWallet {
    pub version: u32,
    pub accounts: Vec<WalletAccount>
}

impl YWallet {
    fn get_account_seed(conn: &Connection, account: u32) -> Option<Vec<u8>> {
        let seed = conn.query_row(
            "SELECT name, seed, sk, ivk, address, aindex FROM accounts WHERE id_account = ?1",
            [account],
            |row| {
                let seed: Option<String> = row.get(1)?;
                Ok(seed)
            }).expect("Failed to fetch seed")
            .unwrap_or("".to_string());

        // println!("{}", seed);
        if seed.is_empty() {
            return None;
        }

        let mnemonic = <Mnemonic<English>>::from_phrase(&seed).expect("Invalid seed phrase");
        let entropy = mnemonic.entropy();

        Some(entropy.to_vec())
    }
    
    fn get_account_tkeys(conn: &Connection, id: u32) -> io::Result<Option<WalletTKey>> {
        let address = db::get_account_taddress(&conn, id)
            .map_err(|_|"No address")
            .unwrap();

        match db::get_account_t_keys(&conn, id) {
            Ok(sk) => {
                Ok(Some(WalletTKey{
                    pk: sk.expect("Invalid SecretKey"),
                    key_type: crate::WalletKeyType::HdKey,
                    index: 0u32,
                    address,
                }))
            },
            Err(_) => Ok(None),
        }
    }

    fn get_account_zkeys(conn: &Connection, id: u32, has_seed: bool) -> io::Result<Option<WalletZKey>> {
        let address = db::get_account_zaddress(&conn, id).unwrap();

        match db::get_account_z_keys(&conn, id) {
            Ok((extsk, ivk, index)) => {
                let key_type = if has_seed {
                    crate::WalletKeyType::HdKey
                } else if extsk.is_some() {
                    crate::WalletKeyType::ImportedExtsk
                } else {
                    crate::WalletKeyType::ImportedViewKey
                };      
        
                Ok(Some(WalletZKey {
                    extsk,
                    fvk: ivk.unwrap(),
                    key_type,
                    index: index.unwrap(),
                    address,
                })) 
            },
            Err(_) => Ok(None)
        }
    }

    // fn get_account_okeys() {

    // }
}

impl WalletParser for YWallet {
    fn read(filename: &str) -> io::Result<Self> {
        let conn = Connection::open(filename)
            .map_err(|_|format!("Couldn't open database file {}", filename))
            .unwrap();

        // get db schema version
        let version = db::get_schema_version(&conn);

        // get available accounts
        let acc = db::get_account_list(&conn).unwrap();

        let accounts: Vec<WalletAccount> = acc.accounts
            .ok_or("Empty account list")
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, a)| {
                // get account seed
                let seed = Self::get_account_seed(&conn, a.id);
                
                // get all keys for this account
                let zkeys = Self::get_account_zkeys(&conn, i as u32 + 1, seed.is_some()).unwrap();
                let tkeys = Self::get_account_tkeys(&conn, i as u32 + 1).unwrap();

                let keys = WalletKeys {
                    tkeys,
                    zkeys,
                    okeys: None
                };
                
                WalletAccount {
                    name: a.name.clone().unwrap_or(format!("Account {}", a.id)),
                    seed,
                    birthday: BlockHeight::from_u32(0),
                    keys,
                }
            })
            .collect();
        
        Ok(
            Self {
                version,
                accounts
            }
        )
    }

    fn get_wallet_name(&self) -> String {
        "YWallet".to_string()
    }

    fn get_wallet_version(&self) -> u64 {
        self.version as u64
    }

    fn get_wallet_accounts(&self) -> std::io::Result<Vec<crate::WalletAccount>> {
        Ok(self.accounts.clone())
    }
}
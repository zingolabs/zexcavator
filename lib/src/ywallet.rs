//! # YWallet Parser
//!
//! This module provides functionality to parse data from YWallet database files.
//! YWallet stores wallet information in a simple SQLite3 database format,
//! making it accessible via the `rusqlite` library.
//!
//! ## Overview
//! The parser extracts the following information from the YWallet database:
//! - **Account Names**: The names of the accounts (if available).
//! - **Seeds**: The entropy used to derive account keys (if available).
//! - **Keys**: The keys associated with each account.
//!
//! ## Caveats
//! - **Wallet Birthday**: The wallet birthday is not currently parsed. Further investigation is needed to determine
//!   where (or if) this information is stored in the database.
//! - **Incomplete Parsing**: The parser focuses on the core components (accounts, seeds, keys) and does not
//!   extract additional metadata or attributes that may be present in the database.
//!
//! ## Implementation Details
//! - **Database Access**: This module uses `rusqlite` to query the SQLite database.
//! - **Expected Format**: The parser assumes the database schema adheres to the standard YWallet format.
//!   Parsing may fail if the database is corrupted or uses a different schema version.
//!
//```

use std::{io, path::Path};

use crate::{
    Wallet, WalletAccount, WalletKeys, WalletOKey, WalletParser, WalletTKey, WalletWriter,
    WalletZKey,
};

pub(crate) mod db;

use bip0039::{English, Mnemonic};
// use orchard::keys::FullViewingKey;
use rusqlite::Connection;

pub struct YWallet {
    pub version: u32,
    pub accounts: Vec<WalletAccount>,
}

impl YWallet {
    fn get_account_seed(conn: &Connection, account: u32) -> Option<Vec<u8>> {
        let seed = conn
            .query_row(
                "SELECT seed FROM accounts WHERE id_account = ?1",
                [account],
                |row| {
                    let seed: Option<String> = row.get(0)?;
                    Ok(seed)
                },
            )
            .expect("Failed to fetch seed")
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
        let address = db::get_account_taddress(conn, id)
            .map_err(|_| "No address")
            .unwrap();

        match db::get_account_t_keys(conn, id) {
            Ok(sk) => {
                Ok(Some(WalletTKey {
                    pk: sk.expect("Invalid SecretKey"),
                    // key_type: crate::WalletKeyType::HdKey,
                    key_type: crate::WalletKeyType::HdDerived,
                    index: 0u32,
                    address,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    fn get_account_zkeys(
        conn: &Connection,
        id: u32,
        has_seed: bool,
    ) -> io::Result<Option<WalletZKey>> {
        let address = db::get_account_zaddress(conn, id).unwrap();

        match db::get_account_z_keys(conn, id) {
            Ok((extsk, ivk, index)) => {
                let key_type = if has_seed {
                    // crate::WalletKeyType::HdKey
                    crate::WalletKeyType::HdDerived
                }
                // else if extsk.is_some() {
                //     crate::WalletKeyType::ImportedExtsk
                // }
                else {
                    // crate::WalletKeyType::ImportedViewKey
                    crate::WalletKeyType::Imported
                };

                Ok(Some(WalletZKey {
                    extsk,
                    fvk: ivk.unwrap(),
                    key_type,
                    index: index.unwrap(),
                    address,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    fn get_account_okeys(
        conn: &Connection,
        id: u32,
        has_seed: bool,
    ) -> io::Result<Option<WalletOKey>> {
        match db::get_account_o_keys(conn, id) {
            Ok((sk, fvk, index, address)) => {
                let key_type = if has_seed {
                    // crate::WalletKeyType::HdKey
                    crate::WalletKeyType::HdDerived
                }
                // else if sk.is_some() {
                //     crate::WalletKeyType::ImportedExtsk
                // }
                else {
                    // crate::WalletKeyType::ImportedViewKey
                    crate::WalletKeyType::Imported
                };

                Ok(Some(WalletOKey {
                    sk,
                    fvk,
                    key_type,
                    index,
                    address,
                }))
            }
            Err(_) => Ok(None),
        }
    }
}

impl WalletParser for YWallet {
    fn read(filename: &str) -> io::Result<Self> {
        let conn = Connection::open(filename)
            .map_err(|_| format!("Couldn't open database file {}", filename))
            .unwrap();

        // get db schema version
        let version = db::get_schema_version(&conn);

        // get available accounts
        let acc = db::get_account_list(&conn).unwrap();

        let accounts: Vec<WalletAccount> = acc
            .accounts
            .ok_or("Empty account list")
            .unwrap()
            .iter()
            .map(|a| {
                // get account seed
                let seed = Self::get_account_seed(&conn, a.id);

                // get all keys for this account
                let zkeys = Self::get_account_zkeys(&conn, a.id, seed.is_some()).unwrap();
                let tkeys = Self::get_account_tkeys(&conn, a.id).unwrap();
                let okeys = Self::get_account_okeys(&conn, a.id, seed.is_some()).unwrap();

                let keys = WalletKeys {
                    tkeys,
                    zkeys,
                    okeys,
                };

                let birthday = db::get_account_birthday(&conn, a.id);

                WalletAccount {
                    name: a.name.clone().unwrap_or(format!("Account {}", a.id)),
                    seed,
                    birthday,
                    keys,
                }
            })
            .collect();

        Ok(Self { version, accounts })
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

impl WalletWriter for YWallet {
    fn write(wallet: &Wallet, filename: &str) -> std::io::Result<()> {
        let path = Path::new(filename);

        if path.exists() {
            println!("File {} already exist, will not overwrite.", filename);
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "File exists"));
        }

        let conn = Connection::open(path.file_name().unwrap())
            .map_err(|_| format!("Couldn't open database file {}", filename))
            .unwrap();

        println!("Exporting wallet to YWallet db format ...");
        let res = db::init_db(&conn).map_err(|_| "Error");

        if res.is_ok() {
            println!("ywallet db init sucess");

            wallet.accounts.iter().enumerate().for_each(|(i, w)| {
                // Handle on accounts with sapling keys
                // YWallet accounts table requires sapling keys
                if w.keys.zkeys.is_some() {
                    println!("Adding account {}", i + 1);
                    db::create_account_with_keys(&conn, w.clone(), i + 1)
                        .expect("unable to create account");
                } else {
                    println!("For transparent only accounts, use YWallet sweep function");
                }
            });
        }

        Ok(())
    }
}

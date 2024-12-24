use std::error::Error;

use rusqlite::Connection;
use sapling::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
use secp256k1::SecretKey;
use zcash_keys::encoding::{decode_extended_spending_key, decode_extended_full_viewing_key};
use zcash_primitives::constants::mainnet::{HRP_SAPLING_EXTENDED_SPENDING_KEY, HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY};

#[derive(Debug)]
pub struct AccountT {
    pub id: u32,
    pub name: Option<String>,
    // pub balance: u64,
}
#[derive(Debug)]
pub struct AccountVecT {
  pub accounts: Option<Vec<AccountT>>,
}

pub fn get_schema_version(connection: &Connection) -> u32 {
    let version: Option<u32> = connection
        .query_row(
            "SELECT version FROM schema_version WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_|"Fail")
        .unwrap();
    version.unwrap_or(0)
}

pub fn get_account_list(conn: &Connection) -> Result<AccountVecT, Box<dyn Error>> {
    let mut stmt = conn.prepare("WITH notes AS (SELECT a.id_account, a.name, CASE WHEN r.spent IS NULL THEN r.value ELSE 0 END AS nv FROM accounts a LEFT JOIN received_notes r ON a.id_account = r.account), \
                       accounts2 AS (SELECT id_account, name, COALESCE(sum(nv), 0) AS balance FROM notes GROUP by id_account) \
                       SELECT a.id_account, a.name, a.balance FROM accounts2 a")?;
    let rows = stmt.query_map([], |row| {
        let id: u32 = row.get("id_account")?;
        let name: String = row.get("name")?;
        let _balance: i64 = row.get("balance")?;
        let account = AccountT {
            id,
            name: Some(name),
            // balance: balance as u64,
        };
        Ok(account)
    })?;
    let mut accounts = vec![];
    for r in rows {
        accounts.push(r?);
    }
    let accounts = AccountVecT {
        accounts: Some(accounts),
    };
    Ok(accounts)
}

pub fn get_account_taddress(conn: &Connection, id: u32) -> Result<String, Box<dyn Error>> {
    let address = conn
        .query_row(
            "SELECT address FROM taddrs WHERE account = ?1",
            [id],
            |row| {
                let address: String = row.get(0)?;
                Ok(address)
            },
        );
        
    Ok(address.unwrap_or(String::new()))
}

pub fn get_account_t_keys(conn: &Connection, id: u32) -> Result<Option<secp256k1::SecretKey>, Box<dyn Error>> {
    let sk_str = conn.query_row(
        "SELECT sk FROM taddrs WHERE account = ?1",
        [id],
        |row| {
            let sk_str: Option<String> = row.get(0)?;
            Ok(sk_str)
        })?;

    let sk = match sk_str {
        Some(s) => {                        
            let sk_hex = hex::decode(s)?;
            let sk = SecretKey::from_slice(&sk_hex)?;
            Some(sk)
        },
        None => None
    };

    Ok(sk)
}

pub fn get_account_zaddress(conn: &Connection, id: u32) -> Result<String, Box<dyn Error>> {
    let address = conn
        .query_row(
            "SELECT address FROM accounts WHERE id_account = ?1",
            [id],
            |row| {
                let address: String = row.get(0)?;
                Ok(address)
            },
        )
        .map_err(|_|"Fail to get z address");
        
    Ok(address.unwrap_or(String::new()))
}

pub fn get_account_z_keys(conn: &Connection, id: u32) -> Result<(Option<ExtendedSpendingKey>, Option<ExtendedFullViewingKey>, Option<u32>), Box<dyn Error>> {
    let (sk_str, ivk_str, index) = conn.query_row(
        "SELECT name, seed, sk, ivk, address, aindex FROM accounts WHERE id_account = ?1",
        [id],
        |row| {
            let sk_str: Option<String> = row.get(2)?;
            let ivk_str: Option<String> = row.get(3)?;
            let index: Option<u32> = row.get(5)?;
            Ok((sk_str, ivk_str, index))
        })?;
        
    let ivk = decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &ivk_str.unwrap())?;

    let extsk = match sk_str {
        Some(s) => Some(decode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &s)?),
        None => None,
    };
    
    Ok(
        (extsk, Some(ivk), index)
    )
}

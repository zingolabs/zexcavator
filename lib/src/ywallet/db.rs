use std::error::Error;

use orchard::keys::{FullViewingKey, SpendingKey};
use rusqlite::Connection;
use sapling::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
use secp256k1::SecretKey;
use zcash_keys::{encoding::{decode_extended_spending_key, decode_extended_full_viewing_key}, address::UnifiedAddress};
use zcash_primitives::{constants::mainnet::{HRP_SAPLING_EXTENDED_SPENDING_KEY, HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY}, consensus::{MainNetwork, BlockHeight}};

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
            "SELECT version FROM schema_version LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_|"Fail").expect("No schema version");
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

pub fn get_account_o_keys(conn: &Connection, id: u32) -> Result<(Option<SpendingKey>, Option<FullViewingKey>, u32, String), Box<dyn Error>> {
    let (sk_blob, fvk_blob, index) = conn
        .query_row(
            "SELECT a.id_account, a.aindex, o.account, o.sk, o.fvk FROM accounts a LEFT JOIN orchard_addrs o ON a.id_account = o.account WHERE o.account = ?1",
            [id],
            |row| {
                let sk_blob:Option<[u8;32]> = row.get(3)?;
                let fvk_blob:Option<[u8;96]> = row.get(4)?;
                let index: Option<u32> = row.get(1)?;
                Ok(
                    (sk_blob, fvk_blob, index)
                )
            },
        )
        .map_err(|_|"Fail to get orchard fvk")?;
        

    let sk:Option<SpendingKey> = match sk_blob {
        Some(sk_bytes) => {
            let sk = SpendingKey::from_bytes(sk_bytes).expect("Invalid sk");
            Some(sk)
        },
        None => None,
    };
    
    let fvk = match fvk_blob {
        Some(f) => Some(FullViewingKey::from_bytes(&f).expect("Invalid fvk")),
        None => None
    };
    
    let address = if fvk.is_some() {
        let o = fvk.clone().unwrap().address_at(index.unwrap(), orchard::keys::Scope::External);
        let ua = UnifiedAddress::from_receivers(Some(o), None, None).expect("Invalid oaddrs");
        let address = ua.encode(&MainNetwork);   
        address
    }
    else {
        String::new()
    };
     
    Ok((
        sk,
        fvk,
        index.unwrap_or(0u32),      
        address
    ))
}

/// Since I don't know where YWallet stores birthday info,
/// I try to estimated birthday height based on first recevied not for this account
pub fn get_account_birthday(conn: &Connection, id: u32) -> BlockHeight {
    let height = conn
    .query_row(
        "SELECT height FROM received_notes WHERE account = ?1",
        [id],
        |row| {
            let height: Option<u32> = row.get(0)?;
            Ok(height)
        },
    )
    .map_err(|_|"Fail to get note height");

    let birthday = match height {
        Ok(h) => BlockHeight::from_u32(h.expect("Invalid height")),
        Err(_) => BlockHeight::from_u32(0),
    };

    birthday
}
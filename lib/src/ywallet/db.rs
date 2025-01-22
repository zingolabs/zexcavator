use std::error::Error;

use bip0039::{English, Mnemonic};
use orchard::keys::{FullViewingKey, SpendingKey};
use rusqlite::Connection;
use sapling::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
use secp256k1::SecretKey;
use zcash_keys::{
    address::UnifiedAddress,
    encoding::{
        decode_extended_full_viewing_key, decode_extended_spending_key,
        encode_extended_full_viewing_key, encode_extended_spending_key,
    },
};
use zcash_primitives::{
    consensus::{BlockHeight, MainNetwork},
    constants::mainnet::{
        HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY,
    },
};

use crate::WalletAccount;

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
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .map_err(|_| "Fail")
        .expect("No schema version");
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
    let address = conn.query_row(
        "SELECT address FROM taddrs WHERE account = ?1",
        [id],
        |row| {
            let address: String = row.get(0)?;
            Ok(address)
        },
    );

    Ok(address.unwrap_or_default())
}

pub fn get_account_t_keys(
    conn: &Connection,
    id: u32,
) -> Result<Option<secp256k1::SecretKey>, Box<dyn Error>> {
    let sk_str = conn.query_row("SELECT sk FROM taddrs WHERE account = ?1", [id], |row| {
        let sk_str: Option<String> = row.get(0)?;
        Ok(sk_str)
    })?;

    let sk = match sk_str {
        Some(s) => {
            let sk_hex = hex::decode(s)?;
            let sk = SecretKey::from_slice(&sk_hex)?;
            Some(sk)
        }
        None => None,
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
        .map_err(|_| "Fail to get z address");

    Ok(address.unwrap_or_default())
}

pub fn get_account_z_keys(
    conn: &Connection,
    id: u32,
) -> Result<
    (
        Option<ExtendedSpendingKey>,
        Option<ExtendedFullViewingKey>,
        Option<u32>,
    ),
    Box<dyn Error>,
> {
    let (sk_str, ivk_str, index) = conn.query_row(
        "SELECT name, seed, sk, ivk, address, aindex FROM accounts WHERE id_account = ?1",
        [id],
        |row| {
            let sk_str: Option<String> = row.get(2)?;
            let ivk_str: Option<String> = row.get(3)?;
            let index: Option<u32> = row.get(5)?;
            Ok((sk_str, ivk_str, index))
        },
    )?;

    let ivk =
        decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &ivk_str.unwrap())?;

    let extsk = match sk_str {
        Some(s) => Some(decode_extended_spending_key(
            HRP_SAPLING_EXTENDED_SPENDING_KEY,
            &s,
        )?),
        None => None,
    };

    Ok((extsk, Some(ivk), index))
}

pub fn get_account_o_keys(
    conn: &Connection,
    id: u32,
) -> Result<(Option<SpendingKey>, Option<FullViewingKey>, u32, String), Box<dyn Error>> {
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

    let sk: Option<SpendingKey> = match sk_blob {
        Some(sk_bytes) => {
            let sk = SpendingKey::from_bytes(sk_bytes).expect("Invalid sk");
            Some(sk)
        }
        None => None,
    };

    let fvk = fvk_blob.map(|f| FullViewingKey::from_bytes(&f).expect("Invalid fvk"));

    let address = if fvk.is_some() {
        let o = fvk
            .clone()
            .unwrap()
            .address_at(index.unwrap(), orchard::keys::Scope::External);
        let ua = UnifiedAddress::from_receivers(Some(o), None, None).expect("Invalid oaddrs");

        ua.encode(&MainNetwork)
    } else {
        String::new()
    };

    Ok((sk, fvk, index.unwrap_or(0u32), address))
}

/// Since I don't know where YWallet stores birthday info,
/// I try to estimate birthday height based on first recevied note for this account
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
        .map_err(|_| "Fail to get note height");

    match height {
        Ok(h) => BlockHeight::from_u32(h.expect("Invalid height")),
        Err(_) => BlockHeight::from_u32(0),
    }
}

pub fn init_db(conn: &Connection) -> std::io::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY NOT NULL,
            version INTEGER NOT NULL)",
        [],
    )
    .expect("Error creating schema table");

    conn.execute(
        "CREATE TABLE accounts (
            id_account INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            seed TEXT,
            aindex INTEGER NOT NULL,
            sk TEXT,
            ivk TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL)",
        [],
    )
    .expect("Error creating accounts table");

    conn.execute(
        "CREATE TABLE blocks (
            height INTEGER PRIMARY KEY,
            hash BLOB NOT NULL,
            timestamp INTEGER NOT NULL)",
        [],
    )
    .expect("Error creating blocks table");

    conn.execute(
        "CREATE TABLE transactions (
            id_tx INTEGER PRIMARY KEY,
            account INTEGER NOT NULL,
            txid BLOB NOT NULL,
            height INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            value INTEGER NOT NULL,
            address TEXT,
            memo TEXT,
            tx_index INTEGER, messages BLOB NULL,
            CONSTRAINT tx_account UNIQUE (height, tx_index, account))",
        [],
    )
    .expect("Error creating transactions table");

    conn.execute(
        "CREATE TABLE sapling_witnesses (
            id_witness INTEGER PRIMARY KEY,
            note INTEGER NOT NULL,
            height INTEGER NOT NULL,
            witness BLOB NOT NULL,
            CONSTRAINT witness_height UNIQUE (note, height))",
        [],
    )
    .expect("Error creating sapling_witnesses table");

    conn.execute(
        "CREATE TABLE diversifiers (
            account INTEGER PRIMARY KEY NOT NULL,
            diversifier_index BLOB NOT NULL)",
        [],
    )
    .expect("Error creating diversifiers table");

    conn.execute(
        "CREATE TABLE contacts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            dirty BOOL NOT NULL)",
        [],
    )
    .expect("Error creating contacts table");

    conn.execute("CREATE INDEX i_account ON accounts(address)", [])
        .expect("Error creating i_account index");

    conn.execute("CREATE INDEX i_contact ON contacts(address)", [])
        .expect("Error creating i_contact index");

    conn.execute("CREATE INDEX i_transaction ON transactions(account)", [])
        .expect("Error creating i_transaction index");

    conn.execute("CREATE INDEX i_witness ON sapling_witnesses(height)", [])
        .expect("Error creating i_witness index");

    conn.execute(
        "CREATE TABLE messages (
            id INTEGER PRIMARY KEY,
            account INTEGER NOT NULL,
            sender TEXT,
            recipient TEXT NOT NULL,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            height INTEGER NOT NULL,
            read BOOL NOT NULL, id_tx INTEGER, incoming BOOL NOT NULL DEFAULT true, vout INTEGER NOT NULL DEFAULT(0))",
        [],
    ).expect("Error creating messages table");

    conn.execute(
        "CREATE TABLE orchard_addrs(
            account INTEGER PRIMARY KEY,
            sk BLOB,
            fvk BLOB NOT NULL)",
        [],
    )
    .expect("Error creating orchard_addrs table");

    conn.execute(
        "CREATE TABLE ua_settings(
            account INTEGER PRIMARY KEY,
            transparent BOOL NOT NULL,
            sapling BOOL NOT NULL,
            orchard BOOL NOT NULL)",
        [],
    )
    .expect("Error creating ua_settings table");

    conn.execute(
        "CREATE TABLE sapling_tree(
            height INTEGER PRIMARY KEY,
            tree BLOB NOT NULL)",
        [],
    )
    .expect("Error creating sapling_tree table");

    conn.execute(
        "CREATE TABLE orchard_tree(
            height INTEGER PRIMARY KEY,
            tree BLOB NOT NULL)",
        [],
    )
    .expect("Error creating orchard_tree table");

    conn.execute(
        "CREATE TABLE received_notes (
            id_note INTEGER PRIMARY KEY,
            account INTEGER NOT NULL,
            position INTEGER NOT NULL,
            tx INTEGER NOT NULL,
            height INTEGER NOT NULL,
            output_index INTEGER NOT NULL,
            diversifier BLOB NOT NULL,
            value INTEGER NOT NULL,
            rcm BLOB NOT NULL,
            nf BLOB NOT NULL UNIQUE,
            rho BLOB,
            orchard BOOL NOT NULL DEFAULT false,
            spent INTEGER,
            excluded BOOL,
            CONSTRAINT tx_output UNIQUE (tx, orchard, output_index))",
        [],
    )
    .expect("Error creating received_notes table");

    conn.execute(
        "CREATE TABLE orchard_witnesses (
            id_witness INTEGER PRIMARY KEY,
            note INTEGER NOT NULL,
            height INTEGER NOT NULL,
            witness BLOB NOT NULL,
            CONSTRAINT witness_height UNIQUE (note, height))",
        [],
    )
    .expect("Error creating orchard_witnesses table");

    conn.execute(
        "CREATE INDEX i_orchard_witness ON orchard_witnesses(height)",
        [],
    )
    .expect("Error creating i_orchard_witnessx index");

    conn.execute(
        "CREATE TABLE send_templates (
            id_send_template INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            address TEXT NOT NULL,
            amount INTEGER NOT NULL,
            fiat_amount DECIMAL NOT NULL,
            fee_included BOOL NOT NULL,
            fiat TEXT,
            include_reply_to BOOL NOT NULL,
            subject TEXT NOT NULL,
            body TEXT NOT NULL)",
        [],
    )
    .expect("Error creating send_templates table");

    conn.execute(
        "CREATE TABLE properties (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL)",
        [],
    )
    .expect("Error creating properties table");

    conn.execute(
        "CREATE TABLE taddrs (
            account INTEGER PRIMARY KEY NOT NULL,
            sk TEXT,
            address TEXT NOT NULL, balance INTEGER, height INTEGER NOT NULL DEFAULT 0)",
        [],
    )
    .expect("Error creating taddrs table");

    conn.execute(
        "CREATE TABLE hw_wallets(
            account INTEGER PRIMARY KEY NOT NULL,
            ledger BOOL NOT NULL)",
        [],
    )
    .expect("Error creating hw_wallets table");

    conn.execute(
        "CREATE TABLE accounts2 (
            account INTEGER PRIMARY KEY NOT NULL,
            saved BOOL NOT NULL)",
        [],
    )
    .expect("Error creating accounts2 table");

    conn.execute(
        "CREATE TABLE account_properties (
            account INTEGER NOT NULL,
            name TEXT NOT NULL,
            value BLOB NOT NULL,
            PRIMARY KEY (account, name))",
        [],
    )
    .expect("Error creating account_properties table");

    conn.execute(
        "CREATE TABLE transparent_checkpoints (
            height INTEGER PRIMARY KEY)",
        [],
    )
    .expect("Error creating transparent_checkpoints table");

    conn.execute(
        "CREATE TABLE block_times (
            height INTEGER PRIMARY KEY,
            timestamp INTEGER NOT NULL)",
        [],
    )
    .expect("Error creating block_times table");

    conn.execute(
        "CREATE TABLE transparent_tins (
            id_tx INTEGER NOT NULL,
            idx INTEGER NOT NULL,
            hash BLOB NOT NULL,
            vout INTEGER NOT NULL,
            PRIMARY KEY (id_tx, idx))",
        [],
    )
    .expect("Error creating transparent_tins table");

    conn.execute(
        "CREATE TABLE transparent_touts (
            id_tx INTEGER PRIMARY KEY,
            address TEXT NOT NULL)",
        [],
    )
    .expect("Error creating transparent_touts table");

    conn.execute(
        "CREATE TABLE utxos (
            id_utxo INTEGER NOT NULL PRIMARY KEY,
            account INTEGER NOT NULL,
            height INTEGER NOT NULL,
            time INTEGER NOT NULL,
            txid BLOB NOT NULL,
            idx INTEGER NOT NULL,
            value INTEGER NOT NULL,
            spent INTEGER)",
        [],
    )
    .expect("Error creating utxos table");

    conn.execute(
        "CREATE TABLE swaps(
            id_swap INTEGER NOT NULL PRIMARY KEY,
            account INTEGER NOT NULL,
            provider TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            timestamp INTEGER,
            from_currency TEXT NOT NULL,
            from_amount TEXT NOT NULL,
            from_address TEXT NOT NULL,
            from_image TEXT NOT NULL,
            to_currency TEXT NOT NULL,
            to_amount TEXT NOT NULL,
            to_address TEXT NOT NULL,
            to_image TEXT NOT NULL)",
        [],
    )
    .expect("Error creating swaps table");

    conn.execute(
        "CREATE TABLE tins(
            id_tin INTEGER NOT NULL PRIMARY KEY,
            account INTEGER NOT NULL,
            height INTEGER NOT NULL,
            id_tx INTEGER NOT NULL,
            vout INTEGER NOT NULL,
            value INTEGER NOT NULL,
            spent INTEGER)",
        [],
    )
    .expect("Error creating tins table");

    conn.execute(
        "CREATE UNIQUE INDEX transactions_txid
        ON transactions (account, txid)",
        [],
    )
    .expect("Error creating transactions_txid index");

    // add shcema version
    conn.execute(
        "INSERT INTO schema_version (id, version) VALUES (?1, ?2)",
        (1, 15),
    )
    .expect("Unable to insert values");

    Ok(())
}

pub fn create_account_with_keys(
    conn: &Connection,
    account: WalletAccount,
    id: usize,
) -> std::io::Result<()> {
    let seed = match account.seed {
        Some(s) => {
            let mnemonic = <Mnemonic<English>>::from_entropy(s).expect("Invalid seed entropy");
            let phrase = mnemonic.phrase().to_string();
            Some(phrase)
        }
        None => None,
    };

    // Handle sapling keys first
    let (extsk, ivk, address) = match account.keys.zkeys {
        Some(z) => {
            let mut sk = String::new();

            let ivk = encode_extended_full_viewing_key(
                HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY,
                &z.clone().fvk,
            );
            let address = z.clone().address;

            if z.extsk.is_some() {
                sk = encode_extended_spending_key(
                    HRP_SAPLING_EXTENDED_SPENDING_KEY,
                    &z.extsk.unwrap(),
                );
            }
            (Some(sk), Some(ivk), Some(address))
        }
        None => (None, None, None),
    };

    // insert accounts table
    conn.execute(
        "INSERT INTO accounts (id_account, name, seed, aindex, sk, ivk, address) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            id as u32,
            account.name,
            seed,
            id,
            extsk,
            ivk,
            address
        )
    ).expect("Unable to insert values");

    // Then handle orchard keys
    if account.keys.okeys.is_some() {
        let sk = account
            .keys
            .okeys
            .clone()
            .unwrap()
            .sk
            .expect("Invalid orchard sk");
        let fvk = account
            .keys
            .okeys
            .clone()
            .unwrap()
            .fvk
            .expect("Invalid orchard fvk");

        // insert orchard_addrs table
        conn.execute(
            "INSERT INTO orchard_addrs (account, sk, fvk) VALUES (?1, ?2, ?3)",
            (id as u32, sk.to_bytes(), fvk.to_bytes()),
        )
        .expect("Unable to insert values");
    }

    // Add transparent addresses and keys
    if account.keys.tkeys.is_some() {
        let pk = account.keys.tkeys.clone().unwrap().pk;
        let taddress = account.keys.tkeys.unwrap().address;

        // insert taddrs table
        conn.execute(
            "INSERT INTO taddrs (account, sk, address, balance) VALUES (?1, ?2, ?3, ?4)",
            (
                id as u32,
                pk.display_secret().to_string() as String,
                taddress,
                0,
            ),
        )
        .expect("Unable to insert values");
    }

    // db extra configuration
    conn.execute(
        "INSERT INTO accounts2 (account, saved) VALUES (?1, ?2)",
        (id as u32, 0),
    )
    .expect("Unable to insert values");

    // configure ua settings
    conn.execute(
        "INSERT INTO ua_settings (account, transparent, sapling, orchard) VALUES (?1, ?2, ?3, ?4)",
        (
            id as u32,
            0,
            1,
            if account.keys.okeys.is_some() { 1 } else { 0 },
        ),
    )
    .expect("Unable to insert values");

    Ok(())
}

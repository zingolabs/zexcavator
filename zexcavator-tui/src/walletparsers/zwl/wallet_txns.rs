use std::{
    collections::HashMap,
    io::{self, Read},
};

use byteorder::{LittleEndian, ReadBytesExt};
use zcash_encoding::Vector;
use zcash_primitives::{consensus::BlockHeight, transaction::TxId};

use super::transactions::WalletTx;

pub struct WalletTxns {
    pub current: HashMap<TxId, WalletTx>,
    pub last_txid: Option<TxId>,
}

impl WalletTxns {
    pub fn serialized_version() -> u64 {
        21
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Can't read wallettxns because of incorrect version",
            ));
        }

        let txs_tuples = Vector::read(&mut reader, |r| {
            let mut txid_bytes = [0u8; 32];
            r.read_exact(&mut txid_bytes)?;

            Ok((TxId::from_bytes(txid_bytes), WalletTx::read(r).unwrap()))
        })?;

        let current = txs_tuples.into_iter().collect::<HashMap<TxId, WalletTx>>();
        let last_txid = current
            .values()
            .fold(None, |c: Option<(TxId, BlockHeight)>, w| {
                if c.is_none() || w.block > c.unwrap().1 {
                    Some((w.txid, w.block))
                } else {
                    c
                }
            })
            .map(|v| v.0);

        let _mempool = if version <= 20 {
            Vector::read(&mut reader, |r| {
                let mut txid_bytes = [0u8; 32];
                r.read_exact(&mut txid_bytes)?;
                let wtx = WalletTx::read(r)?;

                Ok((TxId::from_bytes(txid_bytes), wtx))
            })?
            .into_iter()
            .collect()
        } else {
            vec![]
        };

        Ok(Self { current, last_txid })
    }
}

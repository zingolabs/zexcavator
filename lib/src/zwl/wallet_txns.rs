use std::{
    collections::HashMap,
    io::{self, Read},
};

use byteorder::{ReadBytesExt, LittleEndian};
use zcash_primitives::transaction::TxId;

pub struct WalletTxns {
    pub current: HashMap<TxId, WalletTx>,
    pub last_txid: Option<TxId>,
}

impl WalletTxns {
    pub fn serialized_version() -> u64 {
        return 21;
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Can't read wallettxns because of incorrect version"
            ))
        }

        

        Ok()
    }
}
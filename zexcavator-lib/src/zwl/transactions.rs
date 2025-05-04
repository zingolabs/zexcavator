use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{self},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use incrementalmerkletree::Position;
use orchard_old::keys::FullViewingKey;
use sapling::{IncrementalWitness, Note, Nullifier, PaymentAddress, zip32::ExtendedFullViewingKey};
use zcash_encoding::{Optional, Vector};
use zcash_primitives::{
    consensus::BlockHeight,
    memo::{Memo, MemoBytes},
    transaction::{
        TxId,
        components::{OutPoint, TxOut},
    },
};

use super::{orchard_data::OrchardNoteData, sapling_data::SaplingNoteData};

pub const MAX_REORG: usize = 100;

#[derive(Clone, Debug)]
pub struct WalletTx {
    // Block in which this tx was included
    pub block: BlockHeight,

    // Is this Tx unconfirmed (i.e., not yet mined)
    pub unconfirmed: bool,

    // Timestamp of Tx. Added in v4
    pub datetime: u64,

    // Txid of this transaction. It's duplicated here (It is also the Key in the HashMap that points to this
    // WalletTx in LightWallet::txs)
    pub txid: TxId,

    // List of all nullifiers spent in this Tx. These nullifiers belong to the wallet.
    pub s_spent_nullifiers: Vec<sapling::Nullifier>,

    // List of all orchard nullifiers spent in this Tx.
    pub o_spent_nullifiers: Vec<orchard_old::note::Nullifier>,

    // List of all notes received in this tx. Some of these might be change notes.
    pub s_notes: Vec<SaplingNoteData>,

    // List of all orchard notes recieved in this tx. Some of these might be change.
    pub o_notes: Vec<OrchardNoteData>,

    // List of all Utxos received in this Tx. Some of these might be change notes
    pub utxos: Vec<Utxo>,

    // Total value of all orchard nullifiers that were spent in this Tx
    pub total_orchard_value_spent: u64,

    // Total value of all the sapling nullifiers that were spent in this Tx
    pub total_sapling_value_spent: u64,

    // Total amount of transparent funds that belong to us that were spent in this Tx.
    pub total_transparent_value_spent: u64,

    // All outgoing sapling sends to addresses outside this wallet
    pub outgoing_metadata: Vec<OutgoingTxMetadata>,

    // Whether this TxID was downloaded from the server and scanned for Memos
    pub full_tx_scanned: bool,

    // Price of Zec when this Tx was created
    pub zec_price: Option<f64>,
}

#[allow(dead_code)]
impl WalletTx {
    pub fn serialized_version() -> u64 {
        23
    }

    pub fn new_txid(txid: &Vec<u8>) -> TxId {
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(txid);
        TxId::from_bytes(txid_bytes)
    }

    pub fn new(height: BlockHeight, datetime: u64, txid: &TxId, unconfirmed: bool) -> Self {
        WalletTx {
            block: height,
            unconfirmed,
            datetime,
            txid: *txid,
            o_spent_nullifiers: vec![],
            s_spent_nullifiers: vec![],
            s_notes: vec![],
            o_notes: vec![],
            utxos: vec![],
            total_transparent_value_spent: 0,
            total_sapling_value_spent: 0,
            total_orchard_value_spent: 0,
            outgoing_metadata: vec![],
            full_tx_scanned: false,
            zec_price: None,
        }
    }

    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let block = BlockHeight::from_u32(reader.read_i32::<LittleEndian>()? as u32);

        let unconfirmed = if version <= 20 {
            false
        } else {
            reader.read_u8()? == 1
        };

        let datetime = if version >= 4 {
            reader.read_u64::<LittleEndian>()?
        } else {
            0
        };

        let mut txid_bytes = [0u8; 32];
        reader.read_exact(&mut txid_bytes)?;

        let txid = TxId::from_bytes(txid_bytes);

        let s_notes = Vector::read(&mut reader, |r| SaplingNoteData::read(r))?;
        let utxos = Vector::read(&mut reader, |r| Utxo::read(r))?;

        let total_orchard_value_spent = if version <= 22 {
            0
        } else {
            reader.read_u64::<LittleEndian>()?
        };
        let total_sapling_value_spent = reader.read_u64::<LittleEndian>()?;
        let total_transparent_value_spent = reader.read_u64::<LittleEndian>()?;

        // Outgoing metadata was only added in version 2
        let outgoing_metadata = Vector::read(&mut reader, |r| OutgoingTxMetadata::read(r))?;

        let full_tx_scanned = reader.read_u8()? > 0;

        let zec_price = if version <= 4 {
            None
        } else {
            Optional::read(&mut reader, |r| r.read_f64::<LittleEndian>())?
        };

        let s_spent_nullifiers = if version <= 5 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut n = [0u8; 32];
                r.read_exact(&mut n)?;
                Ok(sapling::Nullifier(n))
            })?
        };

        let o_notes = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| OrchardNoteData::read(r))?
        };

        let o_spent_nullifiers = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut rho_bytes = [0u8; 32];
                r.read_exact(&mut rho_bytes)?;
                Ok(orchard_old::note::Nullifier::from_bytes(&rho_bytes).unwrap())
            })?
        };

        Ok(Self {
            block,
            unconfirmed,
            datetime,
            txid,
            s_notes,
            o_notes,
            utxos,
            s_spent_nullifiers,
            o_spent_nullifiers,
            total_sapling_value_spent,
            total_orchard_value_spent,
            total_transparent_value_spent,
            outgoing_metadata,
            full_tx_scanned,
            zec_price,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        let block: u32 = self.block.into();
        writer.write_i32::<LittleEndian>(block as i32)?;

        writer.write_u8(if self.unconfirmed { 1 } else { 0 })?;

        writer.write_u64::<LittleEndian>(self.datetime)?;

        writer.write_all(self.txid.as_ref())?;

        Vector::write(&mut writer, &self.s_notes, |w, nd| nd.write(w))?;
        Vector::write(&mut writer, &self.utxos, |w, u| u.write(w))?;

        writer.write_u64::<LittleEndian>(self.total_orchard_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_sapling_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_transparent_value_spent)?;

        // Write the outgoing metadata
        Vector::write(&mut writer, &self.outgoing_metadata, |w, om| om.write(w))?;

        writer.write_u8(if self.full_tx_scanned { 1 } else { 0 })?;

        Optional::write(&mut writer, self.zec_price, |w, p| {
            w.write_f64::<LittleEndian>(p)
        })?;

        Vector::write(&mut writer, &self.s_spent_nullifiers, |w, n| {
            w.write_all(&n.0)
        })?;

        Vector::write(&mut writer, &self.o_notes, |w, n| n.write(w))?;

        Vector::write(&mut writer, &self.o_spent_nullifiers, |w, n| {
            w.write_all(&n.to_bytes())
        })?;

        Ok(())
    }

    pub fn total_funds_spent(&self) -> u64 {
        self.total_orchard_value_spent
            + self.total_sapling_value_spent
            + self.total_transparent_value_spent
    }
}

#[derive(Clone, Debug)]
pub struct Utxo {
    pub address: String,
    pub txid: TxId,
    pub output_index: u64,
    pub script: Vec<u8>,
    pub value: u64,
    pub height: i32,

    pub spent_at_height: Option<i32>,
    pub spent: Option<TxId>, // If this utxo was confirmed spent

    // If this utxo was spent in a send, but has not yet been confirmed.
    // Contains the txid and height at which the Tx was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
}

#[allow(dead_code)]
impl Utxo {
    pub fn serialized_version() -> u64 {
        3
    }

    pub fn to_outpoint(&self) -> OutPoint {
        OutPoint::new(*self.txid.as_ref(), self.output_index as u32)
    }

    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let address_len = reader.read_i32::<LittleEndian>()?;
        let mut address_bytes = vec![0; address_len as usize];
        reader.read_exact(&mut address_bytes)?;
        let address = String::from_utf8(address_bytes).unwrap();
        assert_eq!(address.chars().take(1).collect::<Vec<char>>()[0], 't');

        let mut txid_bytes = [0; 32];
        reader.read_exact(&mut txid_bytes)?;
        let txid = TxId::from_bytes(txid_bytes);

        let output_index = reader.read_u64::<LittleEndian>()?;
        let value = reader.read_u64::<LittleEndian>()?;
        let height = reader.read_i32::<LittleEndian>()?;

        let script = Vector::read(&mut reader, |r| {
            let mut byte = [0; 1];
            r.read_exact(&mut byte)?;
            Ok(byte[0])
        })?;

        let spent = Optional::read(&mut reader, |r| {
            let mut txbytes = [0u8; 32];
            r.read_exact(&mut txbytes)?;
            Ok(TxId::from_bytes(txbytes))
        })?;

        let spent_at_height = if version <= 1 {
            None
        } else {
            Optional::read(&mut reader, |r| r.read_i32::<LittleEndian>())?
        };

        let unconfirmed_spent = if version <= 2 {
            None
        } else {
            Optional::read(&mut reader, |r| {
                let mut txbytes = [0u8; 32];
                r.read_exact(&mut txbytes)?;

                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txbytes), height))
            })?
        };

        Ok(Utxo {
            address,
            txid,
            output_index,
            script,
            value,
            height,
            spent_at_height,
            spent,
            unconfirmed_spent,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        writer.write_u32::<LittleEndian>(self.address.len() as u32)?;
        writer.write_all(self.address.as_bytes())?;

        writer.write_all(self.txid.as_ref())?;

        writer.write_u64::<LittleEndian>(self.output_index)?;
        writer.write_u64::<LittleEndian>(self.value)?;
        writer.write_i32::<LittleEndian>(self.height)?;

        Vector::write(&mut writer, &self.script, |w, b| w.write_all(&[*b]))?;

        Optional::write(&mut writer, self.spent, |w, txid| {
            w.write_all(txid.as_ref())
        })?;

        Optional::write(&mut writer, self.spent_at_height, |w, s| {
            w.write_i32::<LittleEndian>(s)
        })?;

        Optional::write(&mut writer, self.unconfirmed_spent, |w, (txid, height)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(height)
        })?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct WitnessCache {
    pub witnesses: Vec<IncrementalWitness>,
    pub top_height: u64,
}

#[allow(dead_code)]
impl WitnessCache {
    pub fn new(witnesses: Vec<IncrementalWitness>, top_height: u64) -> Self {
        Self {
            witnesses,
            top_height,
        }
    }

    pub fn empty() -> Self {
        Self {
            witnesses: vec![],
            top_height: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    pub fn clear(&mut self) {
        self.witnesses.clear();
    }

    pub fn get(&self, i: usize) -> Option<&IncrementalWitness> {
        self.witnesses.get(i)
    }

    #[cfg(test)]
    pub fn get_from_last(&self, i: usize) -> Option<&IncrementalWitness> {
        self.witnesses.get(self.len() - i - 1)
    }

    pub fn last(&self) -> Option<&IncrementalWitness> {
        self.witnesses.last()
    }

    // pub fn into_fsb(self, fsb: &mut FixedSizeBuffer<IncrementalWitness>) {
    //     self.witnesses.into_iter().for_each(|w| fsb.push(w));
    // }

    pub fn pop(&mut self, at_height: u64) {
        while !self.witnesses.is_empty() && self.top_height >= at_height {
            self.witnesses.pop();
            self.top_height -= 1;
        }
    }

    // pub fn get_as_string(&self, i: usize) -> String {
    //     if i >= self.witnesses.len() {
    //         return "".to_string();
    //     }

    //     let mut buf = vec![];
    //     self.get(i).unwrap().write(&mut buf).unwrap();
    //     return hex::encode(buf);
    // }
}

#[derive(PartialEq, Clone, Debug)]
pub struct OutgoingTxMetadata {
    pub address: String,
    pub value: u64,
    pub memo: Memo,
}

impl OutgoingTxMetadata {
    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let address_len = reader.read_u64::<LittleEndian>()?;
        let mut address_bytes = vec![0; address_len as usize];
        reader.read_exact(&mut address_bytes)?;
        let address = String::from_utf8(address_bytes).unwrap();

        let value = reader.read_u64::<LittleEndian>()?;

        let mut memo_bytes = [0u8; 512];
        reader.read_exact(&mut memo_bytes)?;
        let memo = match MemoBytes::from_bytes(&memo_bytes) {
            Ok(mb) => match Memo::try_from(mb.clone()) {
                Ok(m) => Ok(m),
                Err(_) => Ok(Memo::Future(mb)),
            },
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Couldn't create memo: {}", e),
            )),
        }?;

        Ok(OutgoingTxMetadata {
            address,
            value,
            memo,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        // Strings are written as len + utf8
        writer.write_u64::<LittleEndian>(self.address.len() as u64)?;
        writer.write_all(self.address.as_bytes())?;

        writer.write_u64::<LittleEndian>(self.value)?;
        writer.write_all(self.memo.encode().as_array())
    }
}

/// List of all transactions in a wallet.
/// Note that the parent is expected to hold a RwLock, so we will assume that all accesses to
/// this struct are threadsafe/locked properly.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WalletTxns {
    pub(crate) current: HashMap<TxId, WalletTx>,
    pub(crate) last_txid: Option<TxId>,
}

impl Default for WalletTxns {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl WalletTxns {
    pub fn serialized_version() -> u64 {
        21
    }

    pub fn new() -> Self {
        Self {
            current: HashMap::new(),
            last_txid: None,
        }
    }

    pub fn read_old<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let txs_tuples = Vector::read(&mut reader, |r| {
            let mut txid_bytes = [0u8; 32];
            r.read_exact(&mut txid_bytes)?;

            Ok((TxId::from_bytes(txid_bytes), WalletTx::read(r).unwrap()))
        })?;

        let txs = txs_tuples.into_iter().collect::<HashMap<TxId, WalletTx>>();

        Ok(Self {
            current: txs,
            last_txid: None,
        })
    }

    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
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

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        // Write the version
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        // The hashmap, write as a set of tuples. Store them sorted so that wallets are
        // deterministically saved
        {
            let mut txns = self.current.iter().collect::<Vec<(&TxId, &WalletTx)>>();
            txns.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

            Vector::write(&mut writer, &txns, |w, (k, v)| {
                w.write_all(k.as_ref())?;
                v.write(w)
            })?;
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.current.clear();
    }

    pub fn adjust_spendable_status(&mut self, spendable_keys: Vec<ExtendedFullViewingKey>) {
        self.current.values_mut().for_each(|tx| {
            tx.s_notes.iter_mut().for_each(|nd| {
                nd.have_spending_key = spendable_keys.contains(&nd.extfvk);
                if !nd.have_spending_key {
                    nd.witnesses.clear();
                }
            })
        });
    }

    pub fn remove_txids(&mut self, txids_to_remove: Vec<TxId>) {
        for txid in &txids_to_remove {
            self.current.remove(txid);
        }

        // We also need to update any sapling note data and utxos in existing transactions that
        // were spent in any of the txids that were removed
        self.current.values_mut().for_each(|wtx| {
            // Update notes to rollback any spent notes
            wtx.s_notes.iter_mut().for_each(|nd| {
                // Mark note as unspent if the txid being removed spent it.
                if nd.spent.is_some() && txids_to_remove.contains(&nd.spent.unwrap().0) {
                    nd.spent = None;
                }

                // Remove unconfirmed spends too
                if nd.unconfirmed_spent.is_some()
                    && txids_to_remove.contains(&nd.unconfirmed_spent.unwrap().0)
                {
                    nd.unconfirmed_spent = None;
                }
            });

            // Update UTXOs to rollback any spent utxos
            wtx.utxos.iter_mut().for_each(|utxo| {
                if utxo.spent.is_some() && txids_to_remove.contains(&utxo.spent.unwrap()) {
                    utxo.spent = None;
                    utxo.spent_at_height = None;
                }

                if utxo.unconfirmed_spent.is_some()
                    && txids_to_remove.contains(&utxo.unconfirmed_spent.unwrap().0)
                {
                    utxo.unconfirmed_spent = None;
                }
            })
        });
    }

    // During reorgs, we need to remove all txns at a given height, and all spends that refer to any removed txns.
    pub fn remove_txns_at_height(&mut self, reorg_height: u64) {
        let reorg_height = BlockHeight::from_u32(reorg_height as u32);

        // First, collect txids that need to be removed
        let txids_to_remove = self
            .current
            .values()
            .filter_map(|wtx| {
                if wtx.block >= reorg_height {
                    Some(wtx.txid)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.remove_txids(txids_to_remove);

        // Of the notes that still remain, unroll the witness.
        // Trim all witnesses for the invalidated blocks
        for tx in self.current.values_mut() {
            // We only want to trim the witness for "existing" notes, i.e., notes that were created before the block that is being removed
            if tx.block < reorg_height {
                for nd in tx.s_notes.iter_mut() {
                    // The latest witness is at the last() position, so just pop() it.
                    // We should be checking if there is a witness at all, but if there is none, it is an
                    // empty vector, for which pop() is a no-op.
                    nd.witnesses.pop(u64::from(reorg_height));
                }
            }
        }
    }

    pub fn get_last_txid(&self) -> &'_ Option<TxId> {
        &self.last_txid
    }

    pub fn get_notes_for_updating(&self, before_block: u64) -> Vec<(TxId, sapling::Nullifier)> {
        let before_block = BlockHeight::from_u32(before_block as u32);

        self.current
            .iter()
            .filter(|(_, wtx)| !wtx.unconfirmed) // Update only confirmed notes
            .flat_map(|(txid, wtx)| {
                // Fetch notes that are before the before_block.
                wtx.s_notes.iter().filter_map(move |snd| {
                    if wtx.block <= before_block
                        && snd.have_spending_key
                        && !snd.witnesses.is_empty()
                        && snd.spent.is_none()
                    {
                        Some((*txid, snd.nullifier))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn total_funds_spent_in(&self, txid: &TxId) -> u64 {
        self.current
            .get(txid)
            .map(|t| t.total_funds_spent())
            .unwrap_or(0)
    }

    pub fn get_unspent_s_nullifiers(&self) -> Vec<(sapling::Nullifier, u64, TxId)> {
        self.current
            .iter()
            .flat_map(|(_, wtx)| {
                wtx.s_notes
                    .iter()
                    .filter(|nd| nd.spent.is_none())
                    .map(move |nd| (nd.nullifier, nd.note.value().inner(), wtx.txid))
            })
            .collect()
    }

    pub fn get_unspent_o_nullifiers(&self) -> Vec<(orchard_old::note::Nullifier, u64, TxId)> {
        self.current
            .iter()
            .flat_map(|(_, wtx)| {
                wtx.o_notes
                    .iter()
                    .filter(|nd| nd.spent.is_none())
                    .map(move |nd| {
                        (
                            nd.note.nullifier(&nd.fvk),
                            nd.note.value().inner(),
                            wtx.txid,
                        )
                    })
            })
            .collect()
    }

    pub(crate) fn get_note_witness(
        &self,
        txid: &TxId,
        nullifier: &Nullifier,
    ) -> Option<(WitnessCache, BlockHeight)> {
        self.current.get(txid).map(|wtx| {
            wtx.s_notes
                .iter()
                .find(|nd| nd.nullifier == *nullifier)
                .map(|nd| (nd.witnesses.clone(), wtx.block))
        })?
    }

    pub(crate) fn set_o_note_witness(
        &mut self,
        (height, tx_num, output_num): (u64, usize, u32),
        pos: Option<Position>,
    ) {
        self.current.iter_mut().for_each(|(_, wtx)| {
            wtx.o_notes
                .iter_mut()
                .filter(|on| on.witness_position.is_none())
                .find(|on| {
                    let (h, t, p) = on.created_at;
                    height == h && t == tx_num && output_num == p
                })
                .map(|on| on.witness_position = pos);
        });
    }

    pub(crate) fn set_s_note_witnesses(
        &mut self,
        txid: &TxId,
        nullifier: &Nullifier,
        witnesses: WitnessCache,
    ) {
        self.current
            .get_mut(txid)
            .unwrap()
            .s_notes
            .iter_mut()
            .find(|nd| nd.nullifier == *nullifier)
            .unwrap()
            .witnesses = witnesses;
    }

    pub(crate) fn clear_old_witnesses(&mut self, latest_height: u64) {
        let cutoff = (latest_height.saturating_sub(MAX_REORG as u64)) as u32;

        self.current.iter_mut().for_each(|(_, wtx)| {
            wtx.s_notes
                .iter_mut()
                .filter(|n| {
                    !n.witnesses.is_empty() && n.spent.is_some() && n.spent.unwrap().1 < cutoff
                })
                .for_each(|n| n.witnesses.clear());
        });
    }

    pub(crate) fn clear_expired_mempool(&mut self, latest_height: u64) {
        let cutoff = BlockHeight::from_u32((latest_height.saturating_sub(MAX_REORG as u64)) as u32);

        let txids_to_remove = self
            .current
            .iter()
            .filter(|(_, wtx)| wtx.unconfirmed && wtx.block < cutoff)
            .map(|(_, wtx)| wtx.txid)
            .collect::<Vec<_>>();

        txids_to_remove
            .iter()
            .for_each(|t| println!("Removing expired mempool tx {}", t));

        self.remove_txids(txids_to_remove);
    }

    // Will mark the nullifier of the given txid as spent. Returns the amount of the nullifier
    pub fn mark_txid_o_nf_spent(
        &mut self,
        txid: &TxId,
        nullifier: &orchard_old::note::Nullifier,
        spent_txid: &TxId,
        spent_at_height: BlockHeight,
    ) -> u64 {
        let note_data = self
            .current
            .get_mut(txid)
            .unwrap()
            .o_notes
            .iter_mut()
            .find(|n| n.note.nullifier(&n.fvk) == *nullifier)
            .unwrap();

        note_data.spent = Some((*spent_txid, spent_at_height.into()));
        note_data.unconfirmed_spent = None;
        note_data.note.value().inner()
    }

    // Will mark the nullifier of the given txid as spent. Returns the amount of the nullifier
    pub fn mark_txid_s_nf_spent(
        &mut self,
        txid: &TxId,
        nullifier: &Nullifier,
        spent_txid: &TxId,
        spent_at_height: BlockHeight,
    ) -> u64 {
        let note_data = self
            .current
            .get_mut(txid)
            .unwrap()
            .s_notes
            .iter_mut()
            .find(|n| n.nullifier == *nullifier)
            .unwrap();

        note_data.spent = Some((*spent_txid, spent_at_height.into()));
        note_data.unconfirmed_spent = None;
        note_data.note.value().inner()
    }

    // Check this transaction to see if it is an outgoing transaction, and if it is, mark all recieved notes in this
    // transction as change. i.e., If any funds were spent in this transaction, all recieved notes are change notes.
    pub fn check_notes_mark_change(&mut self, txid: &TxId) {
        if self.total_funds_spent_in(txid) > 0 {
            self.current.get_mut(txid).map(|wtx| {
                wtx.s_notes.iter_mut().for_each(|n| {
                    n.is_change = true;
                });

                wtx.o_notes.iter_mut().for_each(|n| {
                    n.is_change = true;
                });
            });
        }
    }

    fn get_or_create_tx(
        &mut self,
        txid: &TxId,
        height: BlockHeight,
        unconfirmed: bool,
        datetime: u64,
    ) -> &'_ mut WalletTx {
        if !self.current.contains_key(txid) {
            self.current
                .insert(*txid, WalletTx::new(height, datetime, txid, unconfirmed));
            self.last_txid = Some(*txid);
        }
        let wtx = self.current.get_mut(txid).expect("Txid should be present");

        // Make sure the unconfirmed status matches
        if wtx.unconfirmed != unconfirmed {
            wtx.unconfirmed = unconfirmed;
            wtx.block = height;
            wtx.datetime = datetime;
        }

        wtx
    }

    pub fn set_price(&mut self, txid: &TxId, price: Option<f64>) {
        price.map(|p| self.current.get_mut(txid).map(|tx| tx.zec_price = Some(p)));
    }

    pub fn add_new_o_spent(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        unconfirmed: bool,
        timestamp: u32,
        nullifier: orchard_old::note::Nullifier,
        value: u64,
        source_txid: TxId,
    ) -> Option<Position> {
        // Record this Tx as having spent some funds
        {
            let wtx = self.get_or_create_tx(&txid, height, unconfirmed, timestamp as u64);

            // Mark the height correctly, in case this was previously a mempool or unconfirmed tx.
            wtx.block = height;

            if !wtx.o_spent_nullifiers.iter().any(|nf| *nf == nullifier) {
                wtx.o_spent_nullifiers.push(nullifier);
                wtx.total_orchard_value_spent += value;
            }
        }

        // Since this Txid has spent some funds, output notes in this Tx that are sent to us are actually change.
        self.check_notes_mark_change(&txid);

        // Mark the source note's nullifier as spent
        if !unconfirmed {
            let wtx = self
                .current
                .get_mut(&source_txid)
                .expect("Txid should be present");

            wtx.o_notes
                .iter_mut()
                .find(|n| n.note.nullifier(&n.fvk) == nullifier)
                .and_then(|nd| {
                    // Record the spent height
                    nd.spent = Some((txid, height.into()));
                    nd.witness_position
                })
        } else {
            None
        }
    }

    // Records a TxId as having spent some nullifiers from the wallet.
    pub fn add_new_s_spent(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        unconfirmed: bool,
        timestamp: u32,
        nullifier: Nullifier,
        value: u64,
        source_txid: TxId,
    ) {
        // Record this Tx as having spent some funds
        {
            let wtx = self.get_or_create_tx(&txid, height, unconfirmed, timestamp as u64);

            // Mark the height correctly, in case this was previously a mempool or unconfirmed tx.
            wtx.block = height;

            if !wtx.s_spent_nullifiers.iter().any(|nf| *nf == nullifier) {
                wtx.s_spent_nullifiers.push(nullifier);
                wtx.total_sapling_value_spent += value;
            }
        }

        // Since this Txid has spent some funds, output notes in this Tx that are sent to us are actually change.
        self.check_notes_mark_change(&txid);

        // Mark the source note's nullifier as spent
        if !unconfirmed {
            let wtx = self
                .current
                .get_mut(&source_txid)
                .expect("Txid should be present");

            if let Some(nd) = wtx.s_notes.iter_mut().find(|n| n.nullifier == nullifier) {
                nd.spent = Some((txid, height.into()));
            }
        }
    }

    pub fn add_taddr_spent(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        unconfirmed: bool,
        timestamp: u64,
        total_transparent_value_spent: u64,
    ) {
        let wtx = self.get_or_create_tx(&txid, height, unconfirmed, timestamp);
        wtx.total_transparent_value_spent = total_transparent_value_spent;

        self.check_notes_mark_change(&txid);
    }

    pub fn mark_txid_utxo_spent(
        &mut self,
        spent_txid: TxId,
        output_num: u32,
        source_txid: TxId,
        source_height: u32,
    ) -> u64 {
        // Find the UTXO
        let value = if let Some(utxo_wtx) = self.current.get_mut(&spent_txid) {
            if let Some(spent_utxo) = utxo_wtx
                .utxos
                .iter_mut()
                .find(|u| u.txid == spent_txid && u.output_index == output_num as u64)
            {
                // Mark this one as spent
                spent_utxo.spent = Some(source_txid);
                spent_utxo.spent_at_height = Some(source_height as i32);
                spent_utxo.unconfirmed_spent = None;

                spent_utxo.value
            } else {
                println!("Couldn't find UTXO that was spent");
                0
            }
        } else {
            println!("Couldn't find TxID that was spent!");
            0
        };

        // Return the value of the note that was spent.
        value
    }

    pub fn add_new_taddr_output(
        &mut self,
        txid: TxId,
        taddr: String,
        height: u32,
        unconfirmed: bool,
        timestamp: u64,
        vout: &TxOut,
        output_num: u32,
    ) {
        // Read or create the current TxId
        let wtx = self.get_or_create_tx(&txid, BlockHeight::from(height), unconfirmed, timestamp);

        // Add this UTXO if it doesn't already exist
        if let Some(utxo) = wtx
            .utxos
            .iter_mut()
            .find(|utxo| utxo.txid == txid && utxo.output_index == output_num as u64)
        {
            // If it already exists, it is likely an mempool tx, so update the height
            utxo.height = height as i32
        } else {
            wtx.utxos.push(Utxo {
                address: taddr,
                txid,
                output_index: output_num as u64,
                script: vout.script_pubkey.0.clone(),
                value: vout.value.into(),
                height: height as i32,
                spent_at_height: None,
                spent: None,
                unconfirmed_spent: None,
            });
        }
    }

    pub fn add_pending_note(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        timestamp: u64,
        note: Note,
        to: PaymentAddress,
        extfvk: &ExtendedFullViewingKey,
    ) {
        // Check if this is a change note
        let is_change = self.total_funds_spent_in(&txid) > 0;

        let wtx = self.get_or_create_tx(&txid, height, true, timestamp);
        // Update the block height, in case this was a mempool or unconfirmed tx.
        wtx.block = height;

        if !wtx.s_notes.iter_mut().any(|n| n.note == note) {
            let nd = SaplingNoteData {
                extfvk: extfvk.clone(),
                diversifier: *to.diversifier(),
                note,
                witnesses: WitnessCache::empty(),
                nullifier: Nullifier([0u8; 32]),
                spent: None,
                unconfirmed_spent: None,
                memo: None,
                is_change,
                have_spending_key: false,
            };

            wtx.s_notes.push(nd);
        }
    }

    pub fn add_new_orchard_note(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        unconfirmed: bool,
        timestamp: u64,
        note: orchard_old::Note,
        created_at: (u64, usize, u32),
        fvk: &FullViewingKey,
        have_spending_key: bool,
    ) {
        // Check if this is a change note
        let is_change = self.total_funds_spent_in(&txid) > 0;

        let wtx = self.get_or_create_tx(&txid, height, unconfirmed, timestamp);
        // Update the block height, in case this was a mempool or unconfirmed tx.
        wtx.block = height;

        let note_nullifier = note.nullifier(fvk);

        match wtx
            .o_notes
            .iter_mut()
            .find(|n| n.note.nullifier(fvk) == note_nullifier)
        {
            None => {
                let nd = OrchardNoteData {
                    fvk: fvk.clone(),
                    note,
                    spent: None,
                    unconfirmed_spent: None,
                    created_at,
                    witness_position: None,
                    memo: None,
                    is_change,
                    have_spending_key,
                };

                wtx.o_notes.push(nd);

                // TODO: Remove pending notes for this tx.
            }
            Some(_) => {
                // If this note already exists, then just reset the witnesses, because we'll start scanning the witnesses
                // again after this.
                // This is likely to happen if the previous wallet wasn't synced properly or was aborted in the middle of a sync,
                // and has some dangling witnesses
                println!("Orchard note already exists in wallet!");
            }
        }
    }

    pub fn add_new_sapling_note(
        &mut self,
        txid: TxId,
        height: BlockHeight,
        unconfirmed: bool,
        timestamp: u64,
        note: Note,
        to: PaymentAddress,
        extfvk: &ExtendedFullViewingKey,
        have_spending_key: bool,
        witness: IncrementalWitness,
    ) {
        // Check if this is a change note
        let is_change = self.total_funds_spent_in(&txid) > 0;

        let wtx = self.get_or_create_tx(&txid, height, unconfirmed, timestamp);
        // Update the block height, in case this was a mempool or unconfirmed tx.
        wtx.block = height;

        let nullifier = note.nf(&extfvk.fvk.vk.nk, u64::from(witness.witnessed_position()));
        let witnesses = if have_spending_key {
            WitnessCache::new(vec![witness], u64::from(height))
        } else {
            WitnessCache::empty()
        };

        match wtx.s_notes.iter_mut().find(|n| n.nullifier == nullifier) {
            None => {
                let nd = SaplingNoteData {
                    extfvk: extfvk.clone(),
                    diversifier: *to.diversifier(),
                    note,
                    witnesses,
                    nullifier,
                    spent: None,
                    unconfirmed_spent: None,
                    memo: None,
                    is_change,
                    have_spending_key,
                };

                wtx.s_notes.push(nd);

                // Also remove any pending notes.
                wtx.s_notes.retain(|n| n.nullifier.0 != [0u8; 32]);
            }
            Some(n) => {
                // If this note already exists, then just reset the witnesses, because we'll start scanning the witnesses
                // again after this.
                // This is likely to happen if the previous wallet wasn't synced properly or was aborted in the middle of a sync,
                // and has some dangling witnesses
                n.witnesses = witnesses;
            }
        }
    }

    // Update the memo for a sapling note if it already exists. If the note doesn't exist, then nothing happens.
    pub fn add_memo_to_s_note(&mut self, txid: &TxId, note: Note, memo: Memo) {
        if let Some(wtx) = self.current.get_mut(txid) {
            wtx.s_notes
                .iter_mut()
                .find(|n| n.note == note)
                .map(|n| n.memo = Some(memo));
        }
    }

    // Update the memo for a orchard note if it already exists. The note has to already exist.
    pub fn add_memo_to_o_note(
        &mut self,
        txid: &TxId,
        fvk: &FullViewingKey,
        note: orchard_old::Note,
        memo: Memo,
    ) {
        // println!("Adding memo to orchard note");
        let note_nullifier = note.nullifier(fvk);

        if let Some(wtx) = self.current.get_mut(txid) {
            wtx.o_notes
                .iter_mut()
                .find(|n| n.note.nullifier(fvk) == note_nullifier)
                .map(|n| n.memo = Some(memo));
        }
    }

    pub fn add_outgoing_metadata(
        &mut self,
        txid: &TxId,
        outgoing_metadata: Vec<OutgoingTxMetadata>,
    ) {
        if let Some(wtx) = self.current.get_mut(txid) {
            // This is n^2 search, but this is likely very small struct, limited by the protocol, so...
            let new_omd: Vec<_> = outgoing_metadata
                .into_iter()
                .filter(|om| !wtx.outgoing_metadata.iter().any(|o| *o == *om))
                .collect();

            wtx.outgoing_metadata.extend(new_omd);
        } else {
            println!(
                "TxId {} should be present while adding metadata, but wasn't",
                txid
            );
        }
    }
}

impl Display for WalletTxns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, ">> WalletTxns <<").unwrap();

        match self.current.is_empty() {
            true => {
                writeln!(f, "No transactions in wallet").unwrap();
            }
            false => {
                writeln!(f, "Current transactions: {}", self.current.len()).unwrap();
            }
        }

        Ok(())
    }
}

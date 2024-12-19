use std::io::{Read, self};
use byteorder::{ReadBytesExt, LittleEndian};
use zcash_encoding::Vector;
use zcash_primitives::{merkle_tree::read_commitment_tree, consensus::BlockHeight, transaction::TxId};
use sapling::CommitmentTree;

#[derive(Clone, Debug)]
pub struct BlockData {
    pub ecb: Vec<u8>,
    pub height: u64,
}

impl BlockData {
    // pub fn serialized_version() -> u64 {
    //     return 20;
    // }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        // read height of CompactBlock
        let height = reader.read_i32::<LittleEndian>()? as u64;

        // read CompactBlock hash
        let mut hash_bytes = [0; 32];
        reader.read_exact(&mut hash_bytes)?;
        hash_bytes.reverse();
        let _hash = hex::encode(hash_bytes);

        // We don't need this, but because of a quirk, the version is stored later, so we can't actually
        // detect the version here. So we write an empty tree and read it back here
        let tree: CommitmentTree = read_commitment_tree(&mut reader)?;
        let _tree = if tree.size() == 0 { None } else { Some(tree) };

        // read version
        let _version = reader.read_u64::<LittleEndian>()?;

        // read "ecb" (encoded compact block?)
        let ecb =Vector::read(&mut reader, |r| r.read_u8()).unwrap_or(vec![]);

        Ok(
            Self {
                ecb,
                height
            }
        )
    }
}

// pub struct WalletTx {
//     // Block in which this tx was included
//     pub block: BlockHeight,

//     // Is this Tx unconfirmed (i.e., not yet mined)
//     pub unconfirmed: bool,

//     // Timestamp of Tx. Added in v4
//     pub datetime: u64,

//     // Txid of this transaction. It's duplicated here (It is also the Key in the HashMap that points to this
//     // WalletTx in LightWallet::txs)
//     pub txid: TxId,

//     // List of all nullifiers spent in this Tx. These nullifiers belong to the wallet.
//     pub s_spent_nullifiers: Vec<sapling::Nullifier>,

//     // List of all orchard nullifiers spent in this Tx.
//     pub o_spent_nullifiers: Vec<orchard::note::Nullifier>,

//     // List of all notes received in this tx. Some of these might be change notes.
//     pub s_notes: Vec<SaplingNoteData>,

//     // List of all orchard notes recieved in this tx. Some of these might be change.
//     pub o_notes: Vec<OrchardNoteData>,

//     // List of all Utxos received in this Tx. Some of these might be change notes
//     pub utxos: Vec<Utxo>,

//     // Total value of all orchard nullifiers that were spent in this Tx
//     pub total_orchard_value_spent: u64,

//     // Total value of all the sapling nullifiers that were spent in this Tx
//     pub total_sapling_value_spent: u64,

//     // Total amount of transparent funds that belong to us that were spent in this Tx.
//     pub total_transparent_value_spent: u64,

//     // All outgoing sapling sends to addresses outside this wallet
//     pub outgoing_metadata: Vec<OutgoingTxMetadata>,

//     // Whether this TxID was downloaded from the server and scanned for Memos
//     pub full_tx_scanned: bool,

//     // Price of Zec when this Tx was created
//     pub zec_price: Option<f64>,
// }
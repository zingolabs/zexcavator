use std::io::{Read, self};
use byteorder::{ReadBytesExt, LittleEndian};
use zcash_encoding::{Vector, Optional};
use zcash_primitives::{merkle_tree::read_commitment_tree, consensus::BlockHeight, transaction::TxId, memo::Memo};
use sapling::{CommitmentTree, zip32::ExtendedFullViewingKey, Diversifier, Rseed, IncrementalWitness};

#[derive(Clone, Debug)]
pub struct BlockData {
    pub ecb: Vec<u8>,
    pub height: u64,
}

impl BlockData {
    pub fn serialized_version() -> u64 {
        return 20;
    }

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
    pub o_spent_nullifiers: Vec<orchard::note::Nullifier>,

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

impl WalletTx {
    pub fn serialized_version() -> u64 {
        return 23;
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let block = BlockHeight::from_u32(reader.read_i32::<LittleEndian>()? as u32);

        // TODO read old wallet version
        let unconfirmed = reader.read_u8()? == 1;

        let datetime = reader.read_u64::<LittleEndian>()?;

        let mut txid_bytes = [0u8; 32];
        reader.read_exact(&mut txid_bytes)?;

        let txid = TxId::from_bytes(txid_bytes);

        let s_notes = Vector::read(&mut reader, |r| SaplingNoteData::read(r))?;
        let utxos = Vector::read(&mut reader, |r| Utxo::read(r))?;

        // TODO read old wallet version
        let total_orchard_value_spent =reader.read_u64::<LittleEndian>()?;

        let total_sapling_value_spent = reader.read_u64::<LittleEndian>()?;
        let total_transparent_value_spent = reader.read_u64::<LittleEndian>()?;

        // Outgoing metadata was only added in version 2
        let outgoing_metadata = Vector::read(&mut reader, |r| OutgoingTxMetadata::read(r))?;

        let full_tx_scanned = reader.read_u8()? > 0;

        // TODO read old wallet version
        let zec_price = Optional::read(&mut reader, |r| r.read_f64::<LittleEndian>())?;

        // TODO read old wallet version
        let s_spent_nullifiers = Vector::read(&mut reader, |r| {
            let mut n = [0u8; 32];
            r.read_exact(&mut n)?;
            Ok(sapling::Nullifier(n))
        })?;

        // TODO read old wallet version
        let o_notes = Vector::read(&mut reader, |r| OrchardNoteData::read(r))?;

        // TODO read old wallet version
        let o_spent_nullifiers = Vector::read(&mut reader, |r| {
            let mut rho_bytes = [0u8; 32];
            r.read_exact(&mut rho_bytes)?;
            Ok(orchard::note::Nullifier::from_bytes(&rho_bytes).unwrap())
        })?;

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
}

pub struct SaplingNoteData {
    // Technically, this should be recoverable from the account number,
    // but we're going to refactor this in the future, so I'll write it again here.
    pub extfvk: ExtendedFullViewingKey,

    pub diversifier: Diversifier,
    pub note: sapling::Note,

    // Witnesses for the last 100 blocks. witnesses.last() is the latest witness
    pub witnesses: WitnessCache,
    pub nullifier: sapling::Nullifier,
    pub spent: Option<(TxId, u32)>, // If this note was confirmed spent

    // If this note was spent in a send, but has not yet been confirmed.
    // Contains the txid and height at which it was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
    pub memo: Option<Memo>,
    pub is_change: bool,

    // If the spending key is available in the wallet (i.e., whether to keep witness up-to-date)
    pub have_spending_key: bool,
}

// Reading a note also needs the corresponding address to read from.
fn read_rseed<R: Read>(mut reader: R) -> io::Result<Rseed> {
    let note_type = reader.read_u8()?;

    let mut r_bytes: [u8; 32] = [0; 32];
    reader.read_exact(&mut r_bytes)?;

    let r = match note_type {
        1 => Rseed::BeforeZip212(jubjub::Fr::from_bytes(&r_bytes).unwrap()),
        2 => Rseed::AfterZip212(r_bytes),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Bad note type")),
    };

    Ok(r)
}

impl SaplingNoteData {
    fn serialized_version() -> u64 {
        20
    }

    // Reading a note also needs the corresponding address to read from.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let _account = if version <= 5 {
            reader.read_u64::<LittleEndian>()?
        } else {
            0
        };

        let extfvk = ExtendedFullViewingKey::read(&mut reader)?;

        let mut diversifier_bytes = [0u8; 11];
        reader.read_exact(&mut diversifier_bytes)?;
        let diversifier = Diversifier { 0: diversifier_bytes };

        // To recover the note, read the value and r, and then use the payment address
        // to recreate the note
        let (value, rseed) = if version <= 3 {
            let value = sapling::value::NoteValue::from_raw(reader.read_u64::<LittleEndian>()?);

            let mut r_bytes: [u8; 32] = [0; 32];
            reader.read_exact(&mut r_bytes)?;

            let r = jubjub::Fr::from_bytes(&r_bytes).unwrap();

            (value, Rseed::BeforeZip212(r))
        } else {
            let value = sapling::value::NoteValue::from_raw(reader.read_u64::<LittleEndian>()?);
            let rseed = read_rseed(&mut reader)?;

            (value, rseed)
        };

        let maybe_note = extfvk
            .fvk
            .vk
            .to_payment_address(diversifier)
            .ok_or("Couldn't create the note for the address")
            .unwrap()
            .create_note(value, rseed);

        // let note = match maybe_note {
        //     Some(n) => Ok(n),
        //     None => Err(io::Error::new(
        //         io::ErrorKind::InvalidInput,
        //         "Couldn't create the note for the address",
        //     )),
        // }?;

        let note = maybe_note;

        let witnesses_vec = Vector::read(&mut reader, |r| IncrementalWitness::read(r))?;
        let top_height = if version < 20 {
            0
        } else {
            reader.read_u64::<LittleEndian>()?
        };
        let witnesses = WitnessCache::new(witnesses_vec, top_height);

        let mut nullifier = [0u8; 32];
        reader.read_exact(&mut nullifier)?;
        let nullifier = sapling::Nullifier(nullifier);

        // Note that this is only the spent field, we ignore the unconfirmed_spent field.
        // The reason is that unconfirmed spents are only in memory, and we need to get the actual value of spent
        // from the blockchain anyway.
        let spent = if version <= 5 {
            let spent = Optional::read(&mut reader, |r| {
                let mut txid_bytes = [0u8; 32];
                r.read_exact(&mut txid_bytes)?;
                Ok(TxId::from_bytes(txid_bytes))
            })?;

            let spent_at_height = if version >= 2 {
                Optional::read(&mut reader, |r| r.read_i32::<LittleEndian>())?
            } else {
                None
            };

            if spent.is_some() && spent_at_height.is_some() {
                Some((spent.unwrap(), spent_at_height.unwrap() as u32))
            } else {
                None
            }
        } else {
            Optional::read(&mut reader, |r| {
                let mut txid_bytes = [0u8; 32];
                r.read_exact(&mut txid_bytes)?;
                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txid_bytes), height))
            })?
        };

        let unconfirmed_spent = if version <= 4 {
            None
        } else {
            Optional::read(&mut reader, |r| {
                let mut txbytes = [0u8; 32];
                r.read_exact(&mut txbytes)?;

                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txbytes), height))
            })?
        };

        let memo = Optional::read(&mut reader, |r| {
            let mut memo_bytes = [0u8; 512];
            r.read_exact(&mut memo_bytes)?;

            // Attempt to read memo, first as text, else as arbitrary 512 bytes
            match MemoBytes::from_bytes(&memo_bytes) {
                Ok(mb) => match Memo::try_from(mb.clone()) {
                    Ok(m) => Ok(m),
                    Err(_) => Ok(Memo::Future(mb)),
                },
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Couldn't create memo: {}", e),
                )),
            }
        })?;

        let is_change: bool = reader.read_u8()? > 0;

        let have_spending_key = if version <= 2 {
            true // Will get populated in the lightwallet::read() method, for now assume true
        } else {
            reader.read_u8()? > 0
        };
    }
}
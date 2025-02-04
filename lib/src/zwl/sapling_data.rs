use std::io::{self};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sapling::{value::NoteValue, zip32::ExtendedFullViewingKey, Diversifier, Rseed};
use zcash_encoding::{Optional, Vector};
use zcash_primitives::{
    memo::{Memo, MemoBytes},
    merkle_tree::{read_incremental_witness, write_incremental_witness},
    transaction::TxId,
};

use super::transactions::WitnessCache;

// Reading a note also needs the corresponding address to read from.
fn read_rseed<R: ReadBytesExt>(mut reader: R) -> io::Result<Rseed> {
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

fn write_rseed<W: WriteBytesExt>(mut writer: W, rseed: &Rseed) -> io::Result<()> {
    let note_type = match rseed {
        Rseed::BeforeZip212(_) => 1,
        Rseed::AfterZip212(_) => 2,
    };
    writer.write_u8(note_type)?;

    match rseed {
        Rseed::BeforeZip212(fr) => writer.write_all(&fr.to_bytes()),
        Rseed::AfterZip212(b) => writer.write_all(b),
    }
}

#[derive(Clone, Debug)]
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

impl SaplingNoteData {
    fn serialized_version() -> u64 {
        20
    }

    // Reading a note also needs the corresponding address to read from.
    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let _account = if version <= 5 {
            reader.read_u64::<LittleEndian>()?
        } else {
            0
        };

        let extfvk = ExtendedFullViewingKey::read(&mut reader)?;

        let mut diversifier_bytes = [0u8; 11];
        reader.read_exact(&mut diversifier_bytes)?;
        let diversifier = Diversifier(diversifier_bytes);

        // To recover the note, read the value and r, and then use the payment address
        // to recreate the note
        let (value, rseed) = if version <= 3 {
            let value = reader.read_u64::<LittleEndian>()?;

            let mut r_bytes: [u8; 32] = [0; 32];
            reader.read_exact(&mut r_bytes)?;

            let r = jubjub::Fr::from_bytes(&r_bytes).unwrap();

            (value, Rseed::BeforeZip212(r))
        } else {
            let value = reader.read_u64::<LittleEndian>()?;
            let rseed = read_rseed(&mut reader)?;

            (value, rseed)
        };

        let note = extfvk
            .fvk
            .vk
            .to_payment_address(diversifier)
            .unwrap()
            .create_note(NoteValue::from_raw(value), rseed);

        let witnesses_vec = Vector::read(&mut reader, |r| read_incremental_witness(r))?;
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

        Ok(SaplingNoteData {
            extfvk,
            diversifier,
            note,
            witnesses,
            nullifier,
            spent,
            unconfirmed_spent,
            memo,
            is_change,
            have_spending_key,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, mut writer: W) -> io::Result<()> {
        // Write a version number first, so we can later upgrade this if needed.
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        self.extfvk.write(&mut writer)?;

        writer.write_all(&self.diversifier.0)?;

        // Writing the note means writing the note.value and note.r. The Note is recoverable
        // from these 2 values and the Payment address.
        writer.write_u64::<LittleEndian>(self.note.value().inner())?;

        write_rseed(&mut writer, self.note.rseed())?;

        Vector::write(&mut writer, &self.witnesses.witnesses, |wr, wi| {
            write_incremental_witness(wi, wr)
        })?;
        writer.write_u64::<LittleEndian>(self.witnesses.top_height)?;

        writer.write_all(&self.nullifier.0)?;

        Optional::write(&mut writer, self.spent, |w, (txid, h)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(h)
        })?;

        Optional::write(&mut writer, self.unconfirmed_spent, |w, (txid, height)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(height)
        })?;

        Optional::write(&mut writer, self.memo.as_ref(), |w, m| {
            w.write_all(m.encode().as_array())
        })?;

        writer.write_u8(if self.is_change { 1 } else { 0 })?;

        writer.write_u8(if self.have_spending_key { 1 } else { 0 })?;

        // Note that we don't write the unconfirmed_spent field, because if the wallet is restarted,
        // we don't want to be beholden to any expired txns

        Ok(())
    }
}

use std::io::{self};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use incrementalmerkletree::Position;
use orchard::{keys::FullViewingKey, note::RandomSeed, value::NoteValue, Address};
use zcash_encoding::Optional;
use zcash_primitives::{
    memo::{Memo, MemoBytes},
    transaction::TxId,
};

#[derive(Clone, Debug)]
pub struct OrchardNoteData {
    pub(super) fvk: FullViewingKey,

    pub note: orchard::Note,

    // (Block number, tx_num, output_num)
    pub created_at: (u64, usize, u32),
    pub witness_position: Option<Position>,

    // Info needed to recreate note
    pub spent: Option<(TxId, u32)>, // If this note was confirmed spent

    // If this note was spent in a send, but has not yet been confirmed.
    // Contains the txid and height at which it was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
    pub memo: Option<Memo>,
    pub is_change: bool,

    // If the spending key is available in the wallet (i.e., whether to keep witness up-to-date)
    pub have_spending_key: bool,
}

impl OrchardNoteData {
    fn serialized_version() -> u64 {
        22
    }

    // Reading a note also needs the corresponding address to read from.
    pub fn read<R: ReadBytesExt>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        assert!(version <= Self::serialized_version());

        let fvk = FullViewingKey::read(&mut reader)?;

        // Read the parts of the note
        // Raw address bytes is 43
        let mut address_bytes = [0u8; 43];
        reader.read_exact(&mut address_bytes)?;
        let note_address = Address::from_raw_address_bytes(&address_bytes).unwrap();
        let note_value = reader.read_u64::<LittleEndian>()?;
        let mut rho_bytes = [0u8; 32];
        reader.read_exact(&mut rho_bytes)?;
        let note_rho = orchard::note::Rho::from_bytes(&rho_bytes).unwrap();
        let mut note_rseed_bytes = [0u8; 32];
        reader.read_exact(&mut note_rseed_bytes)?;
        let note_rseed = RandomSeed::from_bytes(note_rseed_bytes, &note_rho).unwrap();

        let note = orchard::Note::from_parts(
            note_address,
            NoteValue::from_raw(note_value),
            note_rho,
            note_rseed,
        )
        .unwrap();

        let witness_position = Optional::read(&mut reader, |r| {
            let pos = r.read_u64::<LittleEndian>()?;
            Ok(Position::from(pos))
        })?;

        let spent = Optional::read(&mut reader, |r| {
            let mut txid_bytes = [0u8; 32];
            r.read_exact(&mut txid_bytes)?;
            let height = r.read_u32::<LittleEndian>()?;
            Ok((TxId::from_bytes(txid_bytes), height))
        })?;

        let unconfirmed_spent = Optional::read(&mut reader, |r| {
            let mut txbytes = [0u8; 32];
            r.read_exact(&mut txbytes)?;

            let height = r.read_u32::<LittleEndian>()?;
            Ok((TxId::from_bytes(txbytes), height))
        })?;

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

        let have_spending_key = reader.read_u8()? > 0;

        Ok(OrchardNoteData {
            fvk,
            note,
            created_at: (0, 0, 0),
            witness_position,
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

        self.fvk.write(&mut writer)?;

        // Write the components of the note
        writer.write_all(&self.note.recipient().to_raw_address_bytes())?;
        writer.write_u64::<LittleEndian>(self.note.value().inner())?;
        writer.write_all(&self.note.rho().to_bytes())?;
        writer.write_all(self.note.rseed().as_bytes())?;

        // We don't write the created_at, because it should be temporary
        Optional::write(&mut writer, self.witness_position, |w, p| {
            w.write_u64::<LittleEndian>(p.into())
        })?;

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

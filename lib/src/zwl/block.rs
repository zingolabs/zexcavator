use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use sapling::CommitmentTree;
use zcash_encoding::Vector;
use zcash_primitives::merkle_tree::read_commitment_tree;

#[derive(Clone, Debug)]
pub struct BlockData {
    pub ecb: Vec<u8>,
    pub height: u64,
}

impl BlockData {
    pub fn serialized_version() -> u64 {
        20
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
        let ecb = Vector::read(&mut reader, |r| r.read_u8()).unwrap_or_default();

        Ok(Self { ecb, height })
    }
}

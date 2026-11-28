//! Block entity storage: the `0x31` ([`KeyVariant::BlockEntity`]) record of a chunk, which
//! holds the chests, signs, spawners and other block entities in it.
//!
//! The value is a bare concatenation of little-endian NBT compounds with no count prefix,
//! read until the value is exhausted -- the same shape as the legacy inline `0x32` actor
//! list in [`crate::actor`]. As with actors, a block entity's fields are game content, not
//! format, so each one decodes to a raw [`nbtx::Value`]; `bedrock_vanilla_data` reads
//! those into typed block entities.
//!
//! [`KeyVariant::BlockEntity`]: crate::key::KeyVariant::BlockEntity

use crate::actor::{decode_legacy_entities, encode_legacy_entities};
use crate::chunk::ChunkRecords;
use crate::error::Result;
use crate::key::{Key, KeyVariant};

/// Decodes a `0x31` value into its block entities, in on-disk order. An empty value is an
/// empty list: newer worlds keep an empty `0x31` record for every chunk without block
/// entities.
///
/// # Errors
///
/// Returns an error if the value is not a sequence of NBT compounds.
pub fn decode_block_entities(bytes: &[u8]) -> Result<Vec<nbtx::Value>> {
    decode_legacy_entities(bytes)
}

/// Encodes a `0x31` value: the inverse of [`decode_block_entities`].
///
/// # Errors
///
/// Returns an error if a value cannot be encoded as NBT.
pub fn encode_block_entities(values: &[nbtx::Value]) -> Result<Vec<u8>> {
    encode_legacy_entities(values)
}

/// Reads the block entities of one chunk from its record group. A chunk without the
/// record has none.
///
/// # Errors
///
/// Returns an error if the record is not a sequence of NBT compounds.
pub fn read_block_entities(records: &ChunkRecords) -> Result<Vec<nbtx::Value>> {
    match records.get(KeyVariant::BlockEntity) {
        Some(bytes) => decode_block_entities(bytes),
        None => Ok(Vec::new()),
    }
}

/// Replaces the block entities of one chunk in its record group; written to disk with
/// [`ChunkRecords::write`]. An empty list leaves a chunk without the record as it is, and
/// empties the record of a chunk that has one, as the game keeps it.
///
/// # Errors
///
/// Returns an error if a value cannot be encoded as NBT.
pub fn write_block_entities(records: &mut ChunkRecords, values: &[nbtx::Value]) -> Result<()> {
    if values.is_empty() && !records.contains(KeyVariant::BlockEntity) {
        return Ok(());
    }
    let key = Key {
        chunk: records.position(),
        dimension: records.dimension(),
        data: KeyVariant::BlockEntity,
    };
    records.insert(key, encode_block_entities(values)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChunkPosition;
    use bedrock_shared::world::dimension::Dimension;

    #[test]
    fn an_empty_list_adds_no_record_to_a_chunk_without_one() {
        let mut records = ChunkRecords::new(ChunkPosition(0, 0), Dimension::Overworld);
        write_block_entities(&mut records, &[]).unwrap();
        assert!(!records.contains(KeyVariant::BlockEntity));
    }

    #[test]
    fn an_empty_list_empties_the_record_of_a_chunk_with_one() {
        let mut records = ChunkRecords::new(ChunkPosition(0, 0), Dimension::Overworld);
        let chest = nbtx::Value::Compound(nbtx::Compound::from_iter([(
            "id".into(),
            nbtx::Value::String("Chest".into()),
        )]));
        write_block_entities(&mut records, &[chest]).unwrap();
        write_block_entities(&mut records, &[]).unwrap();
        assert_eq!(records.get(KeyVariant::BlockEntity), Some(&[][..]));
    }
}

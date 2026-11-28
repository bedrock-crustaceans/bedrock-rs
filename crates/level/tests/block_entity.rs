//! `crate::block_entity` against the fixture worlds: every `0x31` record decodes, and
//! writes back unchanged, and a chunk's block entities can be replaced or emptied through
//! its record group.

pub mod support;

use std::io::Cursor;

use bedrock_level::block_entity::{
    decode_block_entities, encode_block_entities, read_block_entities, write_block_entities,
};
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level_chunk::Chunk;

/// Every `0x31` value of `db`, with the key it was stored under.
fn block_entity_records(db: &Database) -> Vec<(Key, Vec<u8>)> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        if let Ok(key) = Key::deserialize(&mut Cursor::new(key_bytes.as_ref()))
            && key.data == KeyVariant::BlockEntity
        {
            out.push((key, kv.value().as_ref().to_vec()));
        }
    }
    out
}

/// The first `0x31` record of `db` that holds at least one block entity.
fn first_filled_record(db: &Database) -> (Key, Vec<u8>) {
    block_entity_records(db)
        .into_iter()
        .find(|(_, bytes)| !decode_block_entities(bytes).unwrap().is_empty())
        .expect("the fixture has a chunk with block entities")
}

/// Every record, empty ones included, decodes and writes back to its own bytes.
#[test]
fn block_entity_records_round_trip_across_all_fixtures() {
    let mut filled = 0usize;
    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();
        for (key, bytes) in block_entity_records(&db) {
            let values = decode_block_entities(&bytes)
                .unwrap_or_else(|e| panic!("{}: {key:?}: {e}", fixture.name));
            let written = encode_block_entities(&values).unwrap();
            assert!(
                written == bytes,
                "{}: {key:?} not written back byte for byte",
                fixture.name
            );
            if !values.is_empty() {
                filled += 1;
            }
        }
    }
    assert!(filled > 0, "no fixture holds a block entity");
}

/// A chunk's block entities, replaced through its record group, read back from the
/// database; an empty list empties the record and keeps it, as the game does.
#[test]
fn block_entities_are_replaced_and_emptied_through_the_record_group() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_20_81")
        .expect("v1_20_81 is part of the checked-in fixture set");
    let (_tmp, db) = fixture.open();
    let (key, _) = first_filled_record(&db);

    let mut records = ChunkRecords::read(&db, key.chunk, key.dimension)
        .unwrap()
        .expect("the chunk of a block entity record exists");
    let mut values = read_block_entities(&records).unwrap();
    let first = values[0].clone();
    values.push(first);
    write_block_entities(&mut records, &values).unwrap();
    records.write(&db).unwrap();

    let reread = ChunkRecords::read(&db, key.chunk, key.dimension)
        .unwrap()
        .unwrap();
    assert_eq!(read_block_entities(&reread).unwrap(), values);

    let mut records = reread;
    write_block_entities(&mut records, &[]).unwrap();
    records.write(&db).unwrap();
    let reread = ChunkRecords::read(&db, key.chunk, key.dimension)
        .unwrap()
        .unwrap();
    assert_eq!(reread.get(KeyVariant::BlockEntity), Some(&[][..]));
    assert!(read_block_entities(&reread).unwrap().is_empty());
}

/// A decoded chunk reads its block entities, and an encode writes the replaced list.
#[test]
fn a_decoded_chunk_reads_and_replaces_its_block_entities() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_20_81")
        .expect("v1_20_81 is part of the checked-in fixture set");
    let (_tmp, db) = fixture.open();
    let (key, bytes) = first_filled_record(&db);

    let records = ChunkRecords::read(&db, key.chunk, key.dimension)
        .unwrap()
        .unwrap();
    let mut chunk = Chunk::decode(records, &db).unwrap();
    let mut values = chunk.block_entities().unwrap();
    assert_eq!(values, decode_block_entities(&bytes).unwrap());

    values.truncate(1);
    chunk.set_block_entities(&values).unwrap();
    let encoded = chunk.encode().unwrap();
    assert_eq!(read_block_entities(&encoded).unwrap(), values);

    chunk.set_block_entities(&[]).unwrap();
    let encoded = chunk.encode().unwrap();
    assert_eq!(encoded.get(KeyVariant::BlockEntity), Some(&[][..]));
}

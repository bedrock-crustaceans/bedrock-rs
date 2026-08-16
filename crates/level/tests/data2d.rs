//! Exercises `0x2d` (heightmap + 2D biomes) decode/encode against the
//! version-diverse imported fixtures, as opposed to `test.rs`'s single
//! hardcoded world.

mod support;

use std::collections::BTreeMap;
use std::io::Cursor;

use bedrock_level::height_map::HeightMap;
use bedrock_level::key::{Key, KeyVariant};

/// Every `0x2d` record in every `biomes_2d` fixture decodes, its heightmap
/// values fall within the dimension's observed subchunk height range, and
/// re-encoding it reproduces the exact on-disk bytes.
#[test]
fn decodes_and_round_trips_every_record() {
    let fixtures: Vec<_> = support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.biomes_2d)
        .collect();
    assert!(
        fixtures.len() >= 2,
        "expected at least two biomes_2d fixtures, found {}",
        fixtures.len()
    );

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for fixture in &fixtures {
        let (_tmp, db) = fixture.open();
        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));

        let mut records = 0usize;
        for kv in &mut keys {
            let mut key_cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut key_cursor) else {
                continue;
            };
            if key.data != KeyVariant::HeightMap {
                continue;
            }

            let dim_name = match key.dimension {
                bedrock_shared::world::dimension::Dimension::Overworld => "overworld".to_string(),
                bedrock_shared::world::dimension::Dimension::Nether => "nether".to_string(),
                bedrock_shared::world::dimension::Dimension::End => "end".to_string(),
                bedrock_shared::world::dimension::Dimension::Undefined => "undefined".to_string(),
                // No vanilla fixture carries an add-on dimension (see
                // `crates/level/tests/key_dimension_widening.rs`), so this arm is
                // structurally required but never exercised by the corpus.
                bedrock_shared::world::dimension::Dimension::Other(id) => format!("dimension {id}"),
            };

            let bytes: Vec<u8> = kv.value().into();
            let mut reader = Cursor::new(bytes.as_slice());
            let decoded = HeightMap::from_disk(&mut reader).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?} dim {dim_name}: failed to decode 0x2d record: {e}",
                    fixture.name, key.chunk
                )
            });

            // Sanity: the heightmap value is the height of the column's
            // topmost solid block (or the dimension floor for an empty
            // column), so it must fall within the block range the fixture's
            // subchunks actually span, with one block of slack below the
            // floor for an all-air column.
            if let Some([min, max]) = fixture.subchunk_height_range.get(&dim_name) {
                for &h in decoded.heights().iter() {
                    let h = h as i32;
                    assert!(
                        h >= *min - 1 && h <= *max,
                        "{}: chunk {:?}: heightmap value {h} outside dimension range {min}-1..={max}",
                        fixture.name,
                        key.chunk
                    );
                }
            }

            let mut writer = Cursor::new(Vec::new());
            decoded.to_disk(&mut writer).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?}: failed to re-encode 0x2d record: {e}",
                    fixture.name, key.chunk
                )
            });
            assert_eq!(
                writer.into_inner(),
                bytes,
                "{}: chunk {:?}: re-encoded 0x2d record is not byte-identical",
                fixture.name,
                key.chunk
            );

            records += 1;
        }

        assert!(
            records > 0,
            "{}: capability biomes_2d is set but no 0x2d records were found",
            fixture.name
        );
        counts.insert(fixture.name.clone(), records);
    }

    // Printed rather than asserted on: a useful record of coverage, not an
    // invariant with a "right" number.
    println!("0x2d records decoded and round-tripped per fixture: {counts:?}");
}

/// A record whose length isn't the fixed 768 bytes is a decode error, not a
/// panic.
#[test]
fn rejects_wrong_length_record_as_error_not_panic() {
    let too_short = vec![0u8; 767];
    let mut reader = Cursor::new(too_short.as_slice());
    assert!(HeightMap::from_disk(&mut reader).is_err());

    let too_long = vec![0u8; 769];
    let mut reader = Cursor::new(too_long.as_slice());
    assert!(HeightMap::from_disk(&mut reader).is_err());

    let empty: Vec<u8> = Vec::new();
    let mut reader = Cursor::new(empty.as_slice());
    assert!(HeightMap::from_disk(&mut reader).is_err());
}

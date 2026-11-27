//! `crate::hardcoded_spawn_areas` against real fixture worlds: byte-identical
//! round trips for every real `0x39` `HardcodedSpawnAreas` record the corpus
//! contains.

mod support;

use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::hardcoded_spawn_areas::{
    decode_hardcoded_spawn_areas, encode_hardcoded_spawn_areas,
};
use bedrock_level::key::{Key, KeyVariant};

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// All `0x39` `HardcodedSpawnAreas` values found in `db`, in scan order.
fn scan_hardcoded_spawn_areas(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::HardcodedSpawnAreas {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Only two fixtures in the corpus carry any `0x39` records at all: 1 in
/// `v1_17_20/caves_and_cliffs`, 31 in `v1_18`. This pins the exact counts so
/// any drift in either fixture (or the scan itself) is noticed.
#[test]
fn v1_17_20_caves_and_cliffs_has_exactly_one_hardcoded_spawn_areas_record() {
    let (_tmp, db) = fixture_named("v1_17_20_caves_and_cliffs").open();
    let records = scan_hardcoded_spawn_areas(&db);
    assert_eq!(records.len(), 1);
}

#[test]
fn v1_18_has_exactly_thirty_one_hardcoded_spawn_areas_records() {
    let (_tmp, db) = fixture_named("v1_18").open();
    let records = scan_hardcoded_spawn_areas(&db);
    assert_eq!(records.len(), 31);
}

/// Every real `0x39` record across the corpus decodes and re-encodes
/// byte-identically, and every decoded value's tag byte is one of the four
/// values actually observed corpus-wide -- a guard against silently
/// accepting a byte layout other than the one established.
#[test]
fn every_real_record_round_trips_byte_identically() {
    let mut total = 0usize;
    for fixture_name in ["v1_17_20_caves_and_cliffs", "v1_18"] {
        let (_tmp, db) = fixture_named(fixture_name).open();
        for bytes in scan_hardcoded_spawn_areas(&db) {
            let areas = decode_hardcoded_spawn_areas(&bytes)
                .unwrap_or_else(|e| panic!("{fixture_name}: failed to decode: {e}"));
            for area in &areas {
                assert!(
                    [1, 2, 3, 5].contains(&area.tag),
                    "{fixture_name}: unexpected tag byte {} -- either real data has grown a \
                     new tag value, or the decoder misaligned the record boundaries",
                    area.tag
                );
            }
            let re_encoded = encode_hardcoded_spawn_areas(&areas).unwrap();
            assert_eq!(
                re_encoded, bytes,
                "{fixture_name}: hardcodedSpawnAreas record did not round-trip byte-identically"
            );
            total += areas.len();
        }
    }
    assert_eq!(
        total, 45,
        "expected exactly 45 real 0x39 entries total across all records"
    );
}

#[test]
fn malformed_synthetic_cases_error_never_panic() {
    assert!(decode_hardcoded_spawn_areas(&[]).is_err());
    assert!(decode_hardcoded_spawn_areas(&[0x01, 0x00, 0x00, 0x00]).is_err());
    assert!(decode_hardcoded_spawn_areas(&(-1i32).to_le_bytes()).is_err());
}

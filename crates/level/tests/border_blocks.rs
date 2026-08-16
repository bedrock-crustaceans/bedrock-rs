//! `crate::border_blocks` against real fixture worlds: byte-identical
//! round trips for every real `0x38` BorderBlocks record the corpus
//! contains, plus the malformed synthetic cases.

mod support;

use std::io::Cursor;

use bedrock_level::border_blocks::{decode_border_blocks, encode_border_blocks};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// All `0x38` BorderBlocks values found in `db`, in scan order.
fn scan_border_blocks(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::BorderBlocks {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Only two fixtures in the corpus carry any `0x38` records at all: 9 in
/// `v1_18_30`, 2 in `v1_19_30`. This pins the exact counts so any drift in
/// either fixture (or the scan itself) is noticed.
#[test]
fn v1_18_30_has_exactly_nine_border_blocks_records() {
    let (_tmp, db) = fixture_named("v1_18_30").open();
    let records = scan_border_blocks(&db);
    assert_eq!(records.len(), 9);
}

#[test]
fn v1_19_30_has_exactly_two_border_blocks_records() {
    let (_tmp, db) = fixture_named("v1_19_30").open();
    let records = scan_border_blocks(&db);
    assert_eq!(records.len(), 2);
}

/// Every real `0x38` record across the corpus decodes and re-encodes
/// byte-identically, and every decoded position list is nonempty (a
/// guard against a decoder that silently accepts and returns nothing).
#[test]
fn every_real_record_round_trips_byte_identically() {
    let mut total = 0usize;
    for fixture_name in ["v1_18_30", "v1_19_30"] {
        let (_tmp, db) = fixture_named(fixture_name).open();
        for bytes in scan_border_blocks(&db) {
            let positions = decode_border_blocks(&bytes)
                .unwrap_or_else(|e| panic!("{fixture_name}: failed to decode: {e}"));
            assert!(
                !positions.is_empty(),
                "{fixture_name}: every real record observed has at least one position"
            );
            let re_encoded = encode_border_blocks(&positions).unwrap();
            assert_eq!(
                re_encoded, bytes,
                "{fixture_name}: border blocks record did not round-trip byte-identically"
            );
            total += 1;
        }
    }
    assert_eq!(total, 11, "expected exactly 11 real 0x38 records total");
}

#[test]
fn malformed_synthetic_cases_error_never_panic() {
    assert!(decode_border_blocks(&[]).is_err());
    assert!(decode_border_blocks(&[0x05, 0x01, 0x02]).is_err());
    assert!(decode_border_blocks(&[0x00, 0xff]).is_err());
}

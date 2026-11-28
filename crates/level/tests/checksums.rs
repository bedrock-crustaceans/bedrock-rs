//! `crate::checksums` against real fixture worlds: byte-identical round
//! trips for every real `0x3b` Checksums record the corpus contains.

pub mod support;

use std::io::Cursor;

use bedrock_level::checksums::{decode_checksums, encode_checksums};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// All `0x3b` Checksums values found in `db`, in scan order.
fn scan_checksums(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::Checksums {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Exactly three fixtures in the corpus carry any `0x3b` records at all --
/// `v1_16`, `v1_17_20`, and `v1_17_20/caves_and_cliffs` -- every fixture
/// past that era carries none. This pins the exact counts so any drift in a
/// fixture (or the scan itself) is noticed.
#[test]
fn v1_16_has_exactly_four_hundred_thirty_six_checksums_records() {
    let (_tmp, db) = fixture_named("v1_16").open();
    assert_eq!(scan_checksums(&db).len(), 436);
}

#[test]
fn v1_17_20_has_exactly_eighty_two_checksums_records() {
    let (_tmp, db) = fixture_named("v1_17_20").open();
    assert_eq!(scan_checksums(&db).len(), 82);
}

#[test]
fn v1_17_20_caves_and_cliffs_has_exactly_one_hundred_eighty_two_checksums_records() {
    let (_tmp, db) = fixture_named("v1_17_20_caves_and_cliffs").open();
    assert_eq!(scan_checksums(&db).len(), 182);
}

#[test]
fn no_other_fixture_carries_checksums_records() {
    for fixture in support::fixtures() {
        if matches!(
            fixture.name.as_str(),
            "v1_16" | "v1_17_20" | "v1_17_20_caves_and_cliffs"
        ) {
            continue;
        }
        let (_tmp, db) = fixture.open();
        assert_eq!(
            scan_checksums(&db).len(),
            0,
            "{}: expected no 0x3b records outside the three known-carrying fixtures",
            fixture.name
        );
    }
}

/// Every real `0x3b` record across the corpus decodes and re-encodes
/// byte-identically, and every decoded record's tag equals the on-disk tag
/// byte of one of this crate's own chunk-scoped record kinds -- the
/// structural fact the module docs describe, checked here against the full
/// corpus rather than only the handful of examples in the unit tests.
#[test]
fn every_real_record_round_trips_and_tags_match_known_key_variants() {
    let known_tags: &[i16] = &[0x2b, 0x2d, 0x2f, 0x31, 0x32];
    let mut total = 0usize;
    for fixture_name in ["v1_16", "v1_17_20", "v1_17_20_caves_and_cliffs"] {
        let (_tmp, db) = fixture_named(fixture_name).open();
        for bytes in scan_checksums(&db) {
            let records = decode_checksums(&bytes)
                .unwrap_or_else(|e| panic!("{fixture_name}: failed to decode: {e}"));
            for record in &records {
                assert!(
                    known_tags.contains(&record.tag),
                    "{fixture_name}: unexpected checksum tag 0x{:x}",
                    record.tag
                );
            }
            let re_encoded = encode_checksums(&records).unwrap();
            assert_eq!(
                re_encoded, bytes,
                "{fixture_name}: checksums record did not round-trip byte-identically"
            );
            total += 1;
        }
    }
    assert_eq!(total, 700, "expected exactly 700 real 0x3b records total");
}

#[test]
fn malformed_synthetic_cases_error_never_panic() {
    assert!(decode_checksums(&[]).is_err());
    assert!(decode_checksums(&[0x01, 0x00, 0x00, 0x00]).is_err());
    assert!(decode_checksums(&(-1i32).to_le_bytes()).is_err());
}

//! `crate::aabb_volumes` against the real fixture world: byte-identical
//! round trips for every real `0x77` AabbVolumes record the corpus
//! contains, plus the malformed synthetic cases.
//!
//! Only one world in the corpus carries this key at all -- the 1.26 world
//! at `tests/level.tar.gz` -- so this suite opens that world directly
//! rather than going through `support::fixtures()`, mirroring
//! `tests/test.rs`'s `open_test_db`.

use std::fs::File;
use std::io::Cursor;

use bedrock_level::aabb_volumes::{decode_aabb_volumes, encode_aabb_volumes};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};

use flate2::read::GzDecoder;
use tar::Archive;

fn open_1_26_world() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz = File::open("tests/level.tar.gz").expect("seed missing");
    let tar = GzDecoder::new(tar_gz);
    Archive::new(tar)
        .unpack(tmp.path())
        .expect("failed to unpack seed");
    let db_path = tmp.path().join("debug/db");
    let db = Database::open(db_path.to_str().unwrap()).expect("failed to open database");
    (tmp, db)
}

/// All `0x77` AabbVolumes values found in `db`, in scan order.
fn scan_aabb_volumes(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::AabbVolumes {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Exactly 39 real `0x77` records exist in the whole corpus, all in the
/// 1.26 world. This pins the exact count so any drift is noticed.
#[test]
fn the_1_26_world_has_exactly_thirty_nine_aabb_volumes_records() {
    let (_tmp, db) = open_1_26_world();
    assert_eq!(scan_aabb_volumes(&db).len(), 39);
}

/// Every real `0x77` record decodes and re-encodes byte-identically, and
/// every decoded record has at least one entry (a guard against a decoder
/// that silently accepts and returns nothing) -- the corpus contains no
/// real `N = 0` record, only synthetic ones in `src/aabb_volumes.rs`.
#[test]
fn every_real_record_round_trips_byte_identically() {
    let (_tmp, db) = open_1_26_world();
    let records = scan_aabb_volumes(&db);
    assert_eq!(records.len(), 39);

    for bytes in &records {
        let value = decode_aabb_volumes(bytes).unwrap_or_else(|e| panic!("failed to decode: {e}"));
        assert!(
            !value.entries.is_empty(),
            "every real record observed has at least one entry"
        );
        let re_encoded = encode_aabb_volumes(&value).unwrap();
        assert_eq!(
            &re_encoded, bytes,
            "aabbVolumes record did not round-trip byte-identically"
        );
    }
}

/// Every real record's header is `(1, 1, 0)` and its structure id is
/// `minecraft:trial_chambers` -- true of the whole corpus, but neither is
/// asserted by the decoder itself (see the module docs for why).
#[test]
fn every_real_record_shares_the_same_header_and_structure_id() {
    let (_tmp, db) = open_1_26_world();
    for bytes in scan_aabb_volumes(&db) {
        let value = decode_aabb_volumes(&bytes).unwrap();
        assert_eq!(value.header, (1, 1, 0));
        assert_eq!(value.structure_id, b"minecraft:trial_chambers");
    }
}

/// The trailer length formula (`4 * (3*N + 2)` bytes) holds for every real
/// record, with `N` ranging from 1 to 39 entries across the corpus.
#[test]
fn every_real_record_trailer_matches_the_length_formula() {
    let (_tmp, db) = open_1_26_world();
    for bytes in scan_aabb_volumes(&db) {
        let value = decode_aabb_volumes(&bytes).unwrap();
        let n = value.entries.len();
        assert_eq!(value.trailer.len(), 4 * (3 * n + 2));
    }
}

#[test]
fn malformed_synthetic_cases_error_never_panic() {
    assert!(decode_aabb_volumes(&[]).is_err());
    assert!(decode_aabb_volumes(&[0u8; 11]).is_err());

    // Well-formed header and empty structure id, but a claimed entry count
    // with no entry bytes behind it.
    let mut truncated_entries = vec![0u8; 12];
    truncated_entries.extend_from_slice(&0u16.to_le_bytes());
    truncated_entries.extend_from_slice(&3i32.to_le_bytes());
    assert!(decode_aabb_volumes(&truncated_entries).is_err());

    // Well-formed up through a zero entry count, but a trailer one byte
    // short of the 8 the length formula requires.
    let mut short_trailer = vec![0u8; 12];
    short_trailer.extend_from_slice(&0u16.to_le_bytes());
    short_trailer.extend_from_slice(&0i32.to_le_bytes());
    short_trailer.extend_from_slice(&[0u8; 7]);
    assert!(decode_aabb_volumes(&short_trailer).is_err());
}

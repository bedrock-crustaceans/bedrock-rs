//! `crate::finalized_state` against every real fixture world plus the 1.26
//! seed world (`tests/level.tar.gz`): byte-identical round trips, on-disk
//! width, and the observed value distribution for every real `0x36`
//! `FinalizedState` record the corpus contains.

pub mod support;

use std::fs::File;
use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::finalized_state::{
    FinalizedState, Stage, decode_finalized_state, encode_finalized_state,
};
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

fn scan(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::FinalizedState {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

fn all_databases() -> Vec<(String, tempfile::TempDir, Database)> {
    let mut databases: Vec<(String, tempfile::TempDir, Database)> = support::fixtures()
        .into_iter()
        .map(|fixture| {
            let (tmp, db) = fixture.open();
            (fixture.name, tmp, db)
        })
        .collect();
    let (tmp, db) = open_1_26_world();
    databases.push(("level.tar.gz (1.26)".to_string(), tmp, db));
    databases
}

/// Every real `0x36` record across the whole corpus is exactly 4 bytes
/// (`FinalizedState::Int`), and decodes/re-encodes byte-identically. This
/// pins the exact corpus-wide count so any drift is noticed: 11,331
/// records, zero of them the legacy 1-byte width -- see the module docs
/// for why the 1-byte width is still modeled despite that.
#[test]
fn every_real_record_is_four_bytes_and_round_trips_byte_identically() {
    let databases = all_databases();
    let mut total = 0usize;
    for (world_name, _tmp, db) in &databases {
        for bytes in scan(db) {
            total += 1;
            assert_eq!(
                bytes.len(),
                4,
                "{world_name}: every real FinalizedState record found is 4 bytes"
            );
            let value = decode_finalized_state(&bytes)
                .unwrap_or_else(|e| panic!("{world_name}: failed to decode: {e}"));
            assert!(matches!(value, FinalizedState::Int(_)));
            let re_encoded = encode_finalized_state(&value);
            assert_eq!(
                re_encoded, bytes,
                "{world_name}: finalizedState record did not round-trip byte-identically"
            );
        }
    }
    assert_eq!(total, 11331);
}

/// Every real `0x36` record's value is `0` (needs ticking) or `2` (done).
/// `1` (needs population) never appears in the corpus -- see the module
/// docs.
#[test]
fn every_real_record_is_needs_ticking_or_done() {
    let databases = all_databases();
    let mut seen_stages: std::collections::BTreeSet<&'static str> =
        std::collections::BTreeSet::new();
    for (world_name, _tmp, db) in &databases {
        for bytes in scan(db) {
            let value = decode_finalized_state(&bytes).unwrap();
            match value.stage() {
                Stage::NeedsTicking => {
                    seen_stages.insert("NeedsTicking");
                }
                Stage::Done => {
                    seen_stages.insert("Done");
                }
                other => panic!(
                    "{world_name}: unexpected stage {other:?} (raw {})",
                    value.raw_value()
                ),
            }
        }
    }
    // Both stages are exercised somewhere in the corpus.
    assert!(seen_stages.contains("NeedsTicking"));
    assert!(seen_stages.contains("Done"));
}

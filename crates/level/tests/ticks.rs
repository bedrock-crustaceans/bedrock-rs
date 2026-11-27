//! `crate::ticks` against every real fixture world plus the 1.26 seed world
//! (`tests/level.tar.gz`): byte-identical round trips for every real
//! `0x33` `PendingTicks` and `0x3a` `RandomTicks` record the corpus contains.

pub mod support;

use std::fs::File;
use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::ticks::{decode_tick_list, encode_tick_list};

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

/// All values found under `variant` in `db`, in scan order.
fn scan(db: &Database, variant: KeyVariant) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == variant {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Every real `0x33`/`0x3a` record across the whole corpus (every fixture
/// plus the 1.26 seed world) decodes and re-encodes byte-identically, and
/// every one decodes to a well-formed `{currentTick, tickList}` value with
/// at least one entry -- the corpus contains no real record with an empty
/// tick list. Also pins the exact corpus-wide count for each tag so any
/// drift is noticed: 141 `0x33` records, 182 `0x3a` records.
fn every_real_record_round_trips_byte_identically(variant: KeyVariant, expected_total: usize) {
    let mut total = 0usize;
    let mut databases: Vec<(String, tempfile::TempDir, Database)> = support::fixtures()
        .into_iter()
        .map(|fixture| {
            let (tmp, db) = fixture.open();
            (fixture.name, tmp, db)
        })
        .collect();
    let (tmp, db) = open_1_26_world();
    databases.push(("level.tar.gz (1.26)".to_string(), tmp, db));

    for (world_name, _tmp, db) in &databases {
        for bytes in scan(db, variant) {
            total += 1;
            let list = decode_tick_list(&bytes)
                .unwrap_or_else(|e| panic!("{world_name}: failed to decode: {e}"));
            assert!(
                list.entries().count() > 0,
                "{world_name}: every real record observed has at least one tick-list entry"
            );
            assert!(
                list.current_tick().is_some(),
                "{world_name}: currentTick must decode as an Int"
            );
            for entry in list.entries() {
                assert!(entry.x().is_some(), "{world_name}: entry missing x");
                assert!(entry.y().is_some(), "{world_name}: entry missing y");
                assert!(entry.z().is_some(), "{world_name}: entry missing z");
                assert!(entry.time().is_some(), "{world_name}: entry missing time");
                assert!(
                    entry.block_state().is_some(),
                    "{world_name}: entry missing blockState"
                );
            }
            let re_encoded = encode_tick_list(&list).unwrap();
            assert_eq!(
                &re_encoded, &bytes,
                "{world_name}: tick list record did not round-trip byte-identically"
            );
        }
    }
    assert_eq!(total, expected_total);
}

#[test]
fn every_real_pending_ticks_record_round_trips_byte_identically() {
    every_real_record_round_trips_byte_identically(KeyVariant::PendingTicks, 141);
}

#[test]
fn every_real_random_ticks_record_round_trips_byte_identically() {
    every_real_record_round_trips_byte_identically(KeyVariant::RandomTicks, 182);
}

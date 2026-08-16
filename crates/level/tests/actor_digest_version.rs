//! `crate::actor::{decode,encode}_actor_digest_version` against real
//! fixture worlds: byte-identical round trips for every real `0x41`
//! ActorDigestVersion record the corpus contains, plus the malformed
//! synthetic cases.

mod support;

use std::fs::File;
use std::io::Cursor;

use bedrock_level::actor::{decode_actor_digest_version, encode_actor_digest_version};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};

use flate2::read::GzDecoder;
use tar::Archive;

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// Opens the non-index-catalogued 1.26 world fixture (`tests/level.tar.gz`),
/// mirroring `tests/test.rs`'s `open_test_db`.
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

/// All `0x41` ActorDigestVersion values found in `db`, in scan order.
fn scan_actor_digest_versions(db: &Database) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::ActorDigestVersion {
            out.push(kv.value().as_ref().to_vec());
        }
    }
    out
}

/// Exactly three worlds in the corpus carry any `0x41` records: 210 in
/// `v1_19_30`, 297 in `v1_20_81`, and 90 in the 1.26 world
/// (`tests/level.tar.gz`). Notably `v1_18_30` -- itself `digp`-era, with 793
/// `digp` keys of its own (`tests/actor.rs`) -- carries zero: the `0x41`
/// stamp is not coextensive with the `digp` actor-digest scheme it
/// describes, only with a later subset of the worlds that use it. This pins
/// the exact per-world counts so any drift is noticed.
#[test]
fn exact_counts_across_the_corpus() {
    let (_tmp, db) = fixture_named("v1_19_30").open();
    assert_eq!(scan_actor_digest_versions(&db).len(), 210);

    let (_tmp, db) = fixture_named("v1_20_81").open();
    assert_eq!(scan_actor_digest_versions(&db).len(), 297);

    let (_tmp, db) = open_1_26_world();
    assert_eq!(scan_actor_digest_versions(&db).len(), 90);
}

/// Every real `0x41` record across the corpus is exactly one byte holding
/// `0`, decodes and re-encodes byte-identically, and the corpus is
/// nonempty (a guard against a scan that silently finds nothing).
#[test]
fn every_real_record_round_trips_byte_identically() {
    let mut total = 0usize;
    let (_tmp, v1_19_30) = fixture_named("v1_19_30").open();
    let (_tmp, v1_20_81) = fixture_named("v1_20_81").open();
    let (_tmp, world_1_26) = open_1_26_world();

    for (name, db) in [
        ("v1_19_30", &v1_19_30),
        ("v1_20_81", &v1_20_81),
        ("level.tar.gz (1.26)", &world_1_26),
    ] {
        for bytes in scan_actor_digest_versions(db) {
            let version = decode_actor_digest_version(&bytes)
                .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
            assert_eq!(
                version, 0,
                "{name}: every real record observed is version 0"
            );
            let re_encoded = encode_actor_digest_version(version);
            assert_eq!(
                re_encoded, bytes,
                "{name}: actorDigestVersion record did not round-trip byte-identically"
            );
            total += 1;
        }
    }
    assert_eq!(total, 597, "expected exactly 597 real 0x41 records total");
}

#[test]
fn malformed_synthetic_cases_error_never_panic() {
    assert!(decode_actor_digest_version(&[]).is_err());
    assert!(decode_actor_digest_version(&[0x00, 0x00]).is_err());
    assert!(decode_actor_digest_version(&[0x01, 0x02, 0x03]).is_err());
}

//! Corpus-driven tests for the actor storage-key derivation
//! (`crate::actor::{derive_storage_key, storage_key_for_record}`), the
//! TODO.md task that follows on from the "Write-side maintenance for
//! actors" record and its incidental finding: across the four fixtures
//! that carry `actorprefix` records (`v1_18_30`, `v1_19_30`, `v1_20_81`,
//! and the 1.26 seed world), every one of 7,712 real records matches
//! exactly one of two formulas, dispatched on whether the record's NBT
//! carries an `internalComponents` compound.
//!
//! What is pinned here, exhaustively rather than by sample:
//! - `storage_key_for_record` reproduces every real record's actual
//!   `actorprefix` key, world by world (`for_record_matches_every_real_record`).
//! - The embedded-key branch (`internalComponents.EntityStorageKeyComponent
//!   .StorageKey`) is present, and equals the real key, for exactly the
//!   records that carry `internalComponents` -- the present/absent split per
//!   world (`embedded_key_branch_counts_match_the_corpus`).
//! - `derive_storage_key` against one hand-picked real formula-A record, one
//!   real formula-B record, and one real formula-A record whose session is
//!   `0` (the case the ordinary negative-session examples do not cover),
//!   named by world and identifier rather than only by raw bytes
//!   (`derive_storage_key_matches_a_real_formula_a_record`,
//!   `derive_storage_key_matches_a_real_formula_b_record`,
//!   `derive_storage_key_matches_a_real_zero_session_record`).
//! - `ChunkActors::add_with_derived_key` round-tripping a derived key
//!   through `Chunk::encode`/`encode_actor_writes` and a real `Level`
//!   (`add_with_derived_key_round_trips_through_a_real_chunk_and_level`).
//! - A record with no `UniqueID` and no embedded key errors rather than
//!   fabricating one
//!   (`storage_key_for_record_and_add_with_derived_key_reject_a_record_with_no_unique_id`).

mod support;

use std::fs::File;
use std::io::Cursor;

use bstr::BStr;
use flate2::read::GzDecoder;
use tar::Archive;

use bedrock_level::actor::{
    ActorStorageKey, actorprefix_key_bytes, derive_storage_key, parse_actorprefix_key,
    storage_key_for_record,
};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level::Level;

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// Unpacks `fixture`'s tarball into a fresh `TempDir` and returns the path to
/// the unpacked world directory, mirroring `tests/actor_write.rs::extract_fixture`
/// and `tests/actor_counter.rs::extract_fixture` -- needed (rather than
/// `Fixture::open`) wherever a test also needs a `Level` handle, since
/// `Level` opens its own `Database` and a `Fixture::open` handle held open on
/// the same directory would collide with its exclusive lock.
fn extract_fixture(fixture: &Fixture) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz_path = format!("tests/fixtures/{}", fixture.tarball);
    let tar_gz = File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
    Archive::new(GzDecoder::new(tar_gz))
        .unpack(tmp.path())
        .unwrap_or_else(|e| panic!("{tar_gz_path}: failed to unpack: {e}"));
    let world_dir = tmp.path().join("world");
    (tmp, world_dir)
}

/// The 1.26 seed world's world directory (`tests/level.tar.gz`), same
/// pattern as `tests/actor_counter.rs::extract_1_26_world`.
fn extract_1_26_world() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz = File::open("tests/level.tar.gz").expect("seed missing");
    Archive::new(GzDecoder::new(tar_gz))
        .unpack(tmp.path())
        .expect("failed to unpack seed");
    let world_dir = tmp.path().join("debug");
    (tmp, world_dir)
}

/// One real `actorprefix` record: its on-disk storage key (the `actorprefix`
/// key's 8-byte suffix) and its decoded NBT value.
struct RealRecord {
    storage_key: ActorStorageKey,
    value: nbtx::Value,
}

/// Every `actorprefix` record in `db` that decodes as NBT -- every real
/// record this format's corpus contains for that world, none skipped.
fn all_actorprefix_records(db: &Database) -> Vec<RealRecord> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let Some(storage_key) = parse_actorprefix_key(kv.key().as_ref()) else {
            continue;
        };
        let Some(bytes) = db.get(kv.key().as_ref()).expect("db get") else {
            continue;
        };
        let mut cursor = Cursor::new(bytes.as_ref());
        let value: nbtx::Value =
            nbtx::from_le_bytes(&mut cursor).expect("every real actorprefix record decodes");
        out.push(RealRecord { storage_key, value });
    }
    out
}

/// Per-world corpus pin: `(world name, total actorprefix records with a
/// decodable ``UniqueID``, records carrying `internalComponents` [formula
/// A / embedded-key branch], records without it [formula B])`. Matches the
/// counts TODO.md's Phase 5 incidental finding and the "Actor storage-key
/// derivation" task cite: 7,631 + 81 = 7,712 total across all four worlds.
const WORLD_COUNTS: &[(&str, usize, usize, usize)] = &[
    ("v1_18_30", 3748, 3720, 28),
    ("v1_19_30", 3754, 3726, 28),
    ("v1_20_81", 121, 96, 25),
];

/// The 1.26 seed world's counts, checked separately since it is not
/// index-catalogued (same split as `tests/actor_counter.rs` uses for it) --
/// it carries zero formula-B (painting) records, the only one of the four
/// worlds that does.
const SEED_WORLD_COUNTS: (usize, usize, usize) = (89, 89, 0);

/// Exhaustive, per-world: `storage_key_for_record` reproduces every real
/// `actorprefix` record's actual on-disk storage key, across all 7,712 real
/// records in the four fixtures with actor data. This is the corpus claim
/// the "Actor storage-key derivation" task exists to establish.
#[test]
fn for_record_matches_every_real_record() {
    let mut total_checked = 0usize;

    for &(name, expected_total, _, _) in WORLD_COUNTS {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let records = all_actorprefix_records(&db);

        let mut checked = 0usize;
        for record in &records {
            let derived = storage_key_for_record(&record.value);
            assert_eq!(
                derived,
                Some(record.storage_key),
                "{name}: actorprefix {:02x?} did not derive back to its own key",
                record.storage_key
            );
            checked += 1;
        }
        assert_eq!(checked, expected_total, "{name}: unexpected record count");
        total_checked += checked;
    }

    let (_tmp, world_dir) = extract_1_26_world();
    let db = Database::open(world_dir.join("db").to_str().unwrap()).unwrap();
    let records = all_actorprefix_records(&db);
    assert_eq!(records.len(), SEED_WORLD_COUNTS.0);
    for record in &records {
        assert_eq!(
            storage_key_for_record(&record.value),
            Some(record.storage_key),
            "1_26_seed: actorprefix {:02x?} did not derive back to its own key",
            record.storage_key
        );
    }
    total_checked += records.len();

    assert_eq!(
        total_checked, 7712,
        "total real actorprefix records checked"
    );
}

/// The embedded-key branch (`internalComponents.EntityStorageKeyComponent
/// .StorageKey`) is present, and equal to the record's real key, for exactly
/// the records carrying `internalComponents` -- pinned per world against the
/// exact present/absent split the corpus survey found, not merely "some
/// records have it."
#[test]
fn embedded_key_branch_counts_match_the_corpus() {
    fn count_present_absent(records: &[RealRecord]) -> (usize, usize) {
        let mut present = 0;
        let mut absent = 0;
        for record in records {
            let nbtx::Value::Compound(fields) = &record.value else {
                continue;
            };
            if fields.get(BStr::new("internalComponents")).is_some() {
                present += 1;
            } else {
                absent += 1;
            }
        }
        (present, absent)
    }

    for &(name, _, expected_present, expected_absent) in WORLD_COUNTS {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let records = all_actorprefix_records(&db);
        let (present, absent) = count_present_absent(&records);
        assert_eq!(
            present, expected_present,
            "{name}: internalComponents-present count"
        );
        assert_eq!(
            absent, expected_absent,
            "{name}: internalComponents-absent count"
        );
    }

    let (_tmp, world_dir) = extract_1_26_world();
    let db = Database::open(world_dir.join("db").to_str().unwrap()).unwrap();
    let records = all_actorprefix_records(&db);
    let (present, absent) = count_present_absent(&records);
    assert_eq!(
        present, SEED_WORLD_COUNTS.1,
        "1_26_seed: internalComponents-present count"
    );
    assert_eq!(
        absent, SEED_WORLD_COUNTS.2,
        "1_26_seed: internalComponents-absent count"
    );
}

/// `derive_storage_key` against one hand-picked real formula-A record:
/// `v1_18_30`'s first `minecraft:spider` with `internalComponents`,
/// `UniqueID = -8589934587` (session `-2`, low word `5`), whose real
/// on-disk key is `[0, 0, 0, 2, 0, 0, 0, 5]`.
#[test]
fn derive_storage_key_matches_a_real_formula_a_record() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let records = all_actorprefix_records(&db);

    let record = records
        .iter()
        .find(|r| {
            let nbtx::Value::Compound(fields) = &r.value else {
                return false;
            };
            let identifier = fields.get(BStr::new("identifier"));
            matches!(identifier, Some(nbtx::Value::String(s)) if s.as_slice() == b"minecraft:spider")
                && fields.get(BStr::new("internalComponents")).is_some()
        })
        .expect("v1_18_30 has at least one minecraft:spider record with internalComponents");

    let nbtx::Value::Compound(fields) = &record.value else {
        unreachable!()
    };
    let uid = match fields.get(BStr::new("UniqueID")) {
        Some(nbtx::Value::Long(id)) => *id,
        _ => panic!("record has no Long UniqueID"),
    };

    assert_eq!(uid, -8589934587);
    assert_eq!(record.storage_key, [0, 0, 0, 2, 0, 0, 0, 5]);
    assert_eq!(derive_storage_key(uid, true), record.storage_key);
}

/// `derive_storage_key` against one hand-picked real formula-A record whose
/// session is `0` -- the case the ordinary negative-session example above
/// does not cover, and the corpus's only source of a real `0` session (see
/// `tests/actor_counter.rs`): the 1.26 seed world's first `actorprefix`
/// record with `UniqueID >> 32 == 0`, `UniqueID = 1` (session `0`, low word
/// `1`), whose real on-disk key is `[0, 0, 0, 0, 0, 0, 0, 1]`.
#[test]
fn derive_storage_key_matches_a_real_zero_session_record() {
    let (_tmp, world_dir) = extract_1_26_world();
    let db = Database::open(world_dir.join("db").to_str().unwrap()).unwrap();
    let records = all_actorprefix_records(&db);

    let record = records
        .iter()
        .find(|r| {
            let nbtx::Value::Compound(fields) = &r.value else {
                return false;
            };
            matches!(fields.get(BStr::new("UniqueID")), Some(nbtx::Value::Long(uid)) if *uid >> 32 == 0)
        })
        .expect("the 1.26 seed world has at least one actorprefix record with a zero session");

    let nbtx::Value::Compound(fields) = &record.value else {
        unreachable!()
    };
    let uid = match fields.get(BStr::new("UniqueID")) {
        Some(nbtx::Value::Long(id)) => *id,
        _ => panic!("record has no Long UniqueID"),
    };
    let has_internal_components = fields.get(BStr::new("internalComponents")).is_some();

    assert_eq!(uid, 1);
    assert!(has_internal_components);
    assert_eq!(record.storage_key, [0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(derive_storage_key(uid, true), record.storage_key);
}

/// `derive_storage_key` against one hand-picked real formula-B record:
/// `v1_18_30`'s first `minecraft:painting` lacking `internalComponents`,
/// `UniqueID = -4294967295` (low word `1`), whose real on-disk key is
/// `[1, 0, 0, 0, 0, 0, 0, 0]`.
#[test]
fn derive_storage_key_matches_a_real_formula_b_record() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let records = all_actorprefix_records(&db);

    let record = records
        .iter()
        .find(|r| {
            let nbtx::Value::Compound(fields) = &r.value else {
                return false;
            };
            let identifier = fields.get(BStr::new("identifier"));
            matches!(identifier, Some(nbtx::Value::String(s)) if s.as_slice() == b"minecraft:painting")
                && fields.get(BStr::new("internalComponents")).is_none()
        })
        .expect("v1_18_30 has at least one minecraft:painting record without internalComponents");

    let nbtx::Value::Compound(fields) = &record.value else {
        unreachable!()
    };
    let uid = match fields.get(BStr::new("UniqueID")) {
        Some(nbtx::Value::Long(id)) => *id,
        _ => panic!("record has no Long UniqueID"),
    };

    assert_eq!(uid, -4294967295);
    assert_eq!(record.storage_key, [1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(derive_storage_key(uid, false), record.storage_key);
}

/// `ChunkActors::add_with_derived_key` round-trips through `Chunk::encode`,
/// `Chunk::encode_actor_writes`, and a real `Level`: the derived key shows up
/// in the recomputed `digp` digest, the upsert list, and -- once
/// `ActorWriteSet::apply` (via `Level::put_chunk`) has run -- the actual
/// `actorprefix` record on disk, findable under exactly the key
/// `add_with_derived_key` returned.
#[test]
fn add_with_derived_key_round_trips_through_a_real_chunk_and_level() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let mut level = Level::open_rw(&world_dir).unwrap();

    // Any real chunk works -- the digest-bearing chunk this crate already
    // knows how to find in other actor tests.
    let (position, dimension) = {
        let mut keys = level.raw().keys().unwrap();
        let mut found = None;
        for kv in &mut keys {
            let mut cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };
            if key.data == KeyVariant::ActorDigest {
                found = Some((key.chunk, key.dimension));
                break;
            }
        }
        found.expect("v1_18_30 has at least one digp-bearing chunk")
    };

    let mut chunk = level.chunk(dimension, position).unwrap().unwrap();

    let mut root = nbtx::Compound::default();
    root.insert(
        "identifier".into(),
        nbtx::Value::String("minecraft:cow".into()),
    );
    // A UniqueID under a session no real record in this fixture uses, so the
    // derived key cannot collide with anything already on disk.
    root.insert("UniqueID".into(), nbtx::Value::Long(-4611686018427387899));
    let record = nbtx::Value::Compound(root);

    let derived_key = chunk
        .actors_mut()
        .add_with_derived_key(record.clone())
        .unwrap();
    assert_eq!(derived_key, derive_storage_key(-4611686018427387899, false));
    assert!(chunk.actors().digest_keys().contains(&derived_key));

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.upserts.len(), 1);
    assert_eq!(writes.upserts[0].0, derived_key);

    let _report = level.put_chunk(dimension, position, &chunk).unwrap();

    // Reopen fresh to confirm the record actually reached disk under the
    // derived key, not merely this process's in-memory state.
    drop(level);
    let level = Level::open(&world_dir).unwrap();
    let record_bytes = level
        .raw()
        .get(actorprefix_key_bytes(&derived_key))
        .unwrap()
        .expect("the added actor's record must be findable under its derived key");
    let mut cursor = Cursor::new(record_bytes.as_ref());
    let decoded: nbtx::Value = nbtx::from_le_bytes(&mut cursor).unwrap();
    assert_eq!(decoded, record);

    let reopened_chunk = level.chunk(dimension, position).unwrap().unwrap();
    assert!(reopened_chunk.actors().digest_keys().contains(&derived_key));
}

/// A record with neither an embedded storage key nor a `UniqueID` gives
/// `storage_key_for_record` no way to derive one -- `None`, not a fabricated
/// key -- and `ChunkActors::add_with_derived_key` propagates that as a typed
/// error rather than panicking or guessing.
#[test]
fn storage_key_for_record_and_add_with_derived_key_reject_a_record_with_no_unique_id() {
    let record = nbtx::Value::Compound(nbtx::Compound::default());
    assert_eq!(storage_key_for_record(&record), None);

    let mut actors = bedrock_level::actor::ChunkActors::default();
    let err = actors
        .add_with_derived_key(record)
        .expect_err("a record with no UniqueID and no embedded key must not silently succeed");
    assert!(matches!(err, bedrock_level::error::Error::Invalid(_)));
    assert!(actors.is_empty());
}

/// Compile-time sanity check that the corpus totals used throughout this
/// file (and the TODO.md task they come from) still add up: 7,631 formula-A
/// plus 81 formula-B is 7,712, matching the sum of every per-world total
/// pinned above. A `const` assertion rather than a `#[test]`: this checks
/// only that the constants above are self-consistent, not anything about
/// real data, so it belongs at compile time rather than as a test that
/// exercises no crate behavior.
const _: () = {
    let a_sum = WORLD_COUNTS[0].2 + WORLD_COUNTS[1].2 + WORLD_COUNTS[2].2 + SEED_WORLD_COUNTS.1;
    let b_sum = WORLD_COUNTS[0].3 + WORLD_COUNTS[1].3 + WORLD_COUNTS[2].3 + SEED_WORLD_COUNTS.2;
    assert!(a_sum == 7631);
    assert!(b_sum == 81);
    assert!(a_sum + b_sum == 7712);
};

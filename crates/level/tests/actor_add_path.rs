//! The full actor-add path: `Level::begin_actor_session` /
//! `ChunkActors::add_new`, the last piece of the TODO.md "Actor storage-key
//! derivation" task -- "pair it with `reserve_actor_session_id` so a full
//! add path exists: reserve a session, allocate low bits per run, derive
//! the key."
//!
//! # Survey backing `ActorSession::FIRST_LOW`
//!
//! Across the newest (least negative, or in the 1.26 seed world's case
//! zero) session in each of the four real worlds that carry `actorprefix`
//! data, the lowest low word any real actor record uses is exactly 1, and
//! the set of low words used is contiguous from there:
//!
//! | World | newest session | low-word min | low-word max | distinct count | contiguous? |
//! | --- | --- | --- | --- | --- | --- |
//! | `v1_18_30` | -1 | 1 | 28 | 28 | yes |
//! | `v1_19_30` | -1 | 1 | 28 | 28 | yes |
//! | `v1_20_81` | -1 | 1 | 25 | 25 | yes |
//! | 1.26 seed | 0 | 1 | 12 | 12 | yes |
//!
//! (`v1_18_30`/`v1_19_30`'s newest session holds only `minecraft:painting`
//! records, each low word represented twice -- once formula-A-keyed, once
//! formula-B-keyed, per the split `tests/actor_storage_key.rs` already
//! pins -- which is why a raw low-word count would read 56 rather than 28;
//! the *distinct* low-word set is what is contiguous, and is what is
//! tabulated above.) 0 never appears as a real actor's low word in any
//! session of any of the four worlds. It also does not appear on the
//! `~local_player` record of the three fixtures that have one decodable
//! with a `UniqueID`: all three store `UniqueID = -8589934591`, i.e. session
//! -2, low word 1 -- an *older* session than each world's newest, and still
//! low word 1, not 0. So 0 is not merely absent from the newest session; it
//! is unattested anywhere in the corpus. This is why `ActorSession` starts
//! its low-word counter at 1 (`actor.rs`'s own doc comment on
//! `ActorSession::FIRST_LOW` records the same evidence) -- though this is a
//! stylistic match to the corpus, not a correctness requirement: any start
//! value would be equally collision-safe, per the next paragraph.
//!
//! Because every session `Level::begin_actor_session` hands out is reserved
//! fresh (by construction of `LevelSettings::reserve_actor_session_id`: it
//! claims the current `next_actor_session_id` and immediately decrements
//! the persisted counter past it), no real actor in any of these worlds can
//! ever occupy a low word under a session this crate reserves -- the session
//! half alone already rules out collision, regardless of the low word's
//! start value or whether it has ever repeated across *other*, older
//! sessions in the same world (it has -- every one of these worlds restarts
//! its low-word count at 1 each session, which is expected of a per-run
//! counter and is exactly why the low half needs no persistence).

mod support;

use std::fs::File;
use std::io::Cursor;

use bedrock_level::actor::{ChunkActors, actorprefix_key_bytes, parse_actorprefix_key};
use bedrock_level::db::Database;
use bedrock_level::error::Error;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level::Level;
use bedrock_level::settings::LevelSettings;
use flate2::read::GzDecoder;
use tar::Archive;

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// Unpacks `fixture`'s tarball into a fresh `TempDir`, mirroring
/// `tests/actor_write.rs`/`tests/actor_counter.rs`/`tests/actor_storage_key.rs`'s
/// helper of the same name: a `Level` opens its own `Database` handle, which
/// holds an exclusive lock, so a test driving `Level` must not also hold a
/// `Fixture::open` handle on the same directory.
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

/// One real record's storage key plus its decoded NBT, and the session/low
/// word its `UniqueID` packs -- shared by the corpus-pin tests below.
struct RealRecord {
    session: i32,
    low: i32,
}

/// Every `actorprefix` record in `db` with a decodable `Long` `UniqueID`,
/// split into `(session, low)`.
fn all_actor_sessions(db: &Database) -> Vec<RealRecord> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        if parse_actorprefix_key(kv.key().as_ref()).is_none() {
            continue;
        }
        let Some(bytes) = db.get(kv.key().as_ref()).expect("db get") else {
            continue;
        };
        let mut cursor = Cursor::new(bytes.as_ref());
        let Ok(value) = nbtx::from_le_bytes::<nbtx::Value>(&mut cursor) else {
            continue;
        };
        let nbtx::Value::Compound(fields) = &value else {
            continue;
        };
        let uid = fields
            .iter()
            .find(|(k, _)| k.as_slice() == b"UniqueID")
            .and_then(|(_, v)| match v {
                nbtx::Value::Long(uid) => Some(*uid),
                _ => None,
            });
        if let Some(uid) = uid {
            out.push(RealRecord {
                session: (uid >> 32) as i32,
                low: uid as i32,
            });
        }
    }
    out
}

/// Per-world corpus pin: `(world, newest session, min low, max low, distinct
/// low-word count)` -- the survey tabulated in this file's module docs.
const NEWEST_SESSION_LOW_WORDS: &[(&str, i32, i32, i32, usize)] = &[
    ("v1_18_30", -1, 1, 28, 28),
    ("v1_19_30", -1, 1, 28, 28),
    ("v1_20_81", -1, 1, 25, 25),
];

/// Pins the survey table in this file's module docs against the three
/// index-catalogued fixtures: the newest session's low words are
/// contiguous from 1, never 0.
#[test]
fn newest_session_low_words_start_at_one_and_are_contiguous() {
    for &(name, expected_session, expected_min, expected_max, expected_distinct) in
        NEWEST_SESSION_LOW_WORDS
    {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let records = all_actor_sessions(&db);

        let newest_session = records
            .iter()
            .map(|r| r.session)
            .max()
            .unwrap_or_else(|| panic!("{name}: no actor sessions"));
        assert_eq!(newest_session, expected_session, "{name}: newest session");

        let mut distinct: std::collections::BTreeSet<i32> = records
            .iter()
            .filter(|r| r.session == newest_session)
            .map(|r| r.low)
            .collect();
        let min = *distinct.iter().next().unwrap();
        let max = *distinct.iter().next_back().unwrap();
        assert_eq!(min, expected_min, "{name}: newest-session low-word min");
        assert_eq!(max, expected_max, "{name}: newest-session low-word max");
        assert_eq!(
            distinct.len(),
            expected_distinct,
            "{name}: newest-session distinct low-word count"
        );
        assert_eq!(
            (max - min + 1) as usize,
            distinct.len(),
            "{name}: newest-session low words are not contiguous"
        );
        assert!(!distinct.remove(&0), "{name}: low word 0 must not appear");
    }
}

/// Same pin for the 1.26 seed world, whose newest session is `0` rather than
/// negative -- checked separately since it is not index-catalogued (same
/// split every other actor test file in this crate uses for it).
#[test]
fn seed_world_newest_session_low_words_start_at_one_and_are_contiguous() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz = File::open("tests/level.tar.gz").expect("seed missing");
    Archive::new(GzDecoder::new(tar_gz))
        .unpack(tmp.path())
        .expect("failed to unpack seed");
    let world_dir = tmp.path().join("debug");
    let db = Database::open(world_dir.join("db").to_str().unwrap()).unwrap();

    let records = all_actor_sessions(&db);
    let newest_session = records.iter().map(|r| r.session).max().unwrap();
    assert_eq!(newest_session, 0);

    let distinct: std::collections::BTreeSet<i32> = records
        .iter()
        .filter(|r| r.session == newest_session)
        .map(|r| r.low)
        .collect();
    assert_eq!(*distinct.iter().next().unwrap(), 1);
    assert_eq!(*distinct.iter().next_back().unwrap(), 12);
    assert_eq!(distinct.len(), 12);
    assert!(!distinct.contains(&0));
}

/// `Level::begin_actor_session`'s returned id is exactly what
/// `LevelSettings::next_actor_session_id` reported before the call, and
/// `level.dat` on disk shows the counter decremented past it -- mirroring
/// `tests/actor_counter.rs`'s equivalent check for
/// `Level::reserve_actor_session_id` directly, since `begin_actor_session`
/// is built on it and must not weaken that guarantee.
#[test]
fn begin_actor_session_claims_the_advertised_next_session_and_persists_the_decrement() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let before_level_dat = std::fs::read(world_dir.join("level.dat")).unwrap();
    let before_settings = LevelSettings::read(before_level_dat.as_slice()).unwrap();
    let expected_session = before_settings.next_actor_session_id();

    let claimed_session = {
        let mut level = Level::open_rw(&world_dir).unwrap();
        let session = level.begin_actor_session().unwrap();
        session.session()
    };
    assert_eq!(claimed_session, expected_session);

    let after_level_dat = std::fs::read(world_dir.join("level.dat")).unwrap();
    let after_settings = LevelSettings::read(after_level_dat.as_slice()).unwrap();
    assert_eq!(after_settings.next_actor_session_id(), expected_session - 1);
}

/// Two `ActorSession::next_unique_id` calls on the same session produce
/// distinct ids that share the session's high 32 bits and whose low words
/// are sequential, starting at `ActorSession::FIRST_LOW` (1).
#[test]
fn allocating_two_ids_gives_the_same_session_and_sequential_low_words() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);
    let mut level = Level::open_rw(&world_dir).unwrap();

    let mut session = level.begin_actor_session().unwrap();
    let first = session.next_unique_id().unwrap();
    let second = session.next_unique_id().unwrap();

    assert_ne!(first, second);
    assert_eq!(first >> 32, session.session() as i64);
    assert_eq!(second >> 32, session.session() as i64);
    assert_eq!(first as i32, 1, "first allocated low word must be 1, not 0");
    assert_eq!(second as i32, 2);
}

/// `ChunkActors::add_new` on a real chunk: the added actor's `UniqueID` and
/// embedded `StorageKey` match what was allocated, `Level::put_chunk`
/// persists it, and reopening the level finds the record under the derived
/// key with the digest naming it and the embedded `StorageKey` still equal
/// to that key.
#[test]
fn add_new_round_trips_through_put_chunk_and_reopen() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let mut level = Level::open_rw(&world_dir).unwrap();

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
    let mut session = level.begin_actor_session().unwrap();
    let claimed_session = session.session();

    let mut root = nbtx::Compound::default();
    root.insert(
        "identifier".into(),
        nbtx::Value::String("minecraft:cow".into()),
    );
    let record = nbtx::Value::Compound(root);

    let storage_key = chunk
        .actors_mut()
        .add_new(&mut session, record.clone())
        .unwrap();

    // The expected UniqueID is the session's first allocated low word (1),
    // since this is the first `add_new` call on this session.
    let expected_unique_id = ((claimed_session as i64) << 32) | 1;
    assert_eq!(
        storage_key,
        bedrock_level::actor::derive_storage_key(expected_unique_id, true)
    );
    assert!(chunk.actors().digest_keys().contains(&storage_key));

    let resolved = chunk
        .actors()
        .resolved
        .iter()
        .find(|r| r.storage_key == storage_key)
        .expect("newly added actor must be resolved in this chunk's actor set");
    let nbtx::Value::Compound(fields) = &resolved.value else {
        panic!("added record is not a compound");
    };
    assert_eq!(
        fields.get("UniqueID".as_bytes()),
        Some(&nbtx::Value::Long(expected_unique_id))
    );
    let internal_components = fields
        .get("internalComponents".as_bytes())
        .expect("add_new must have created internalComponents");
    let nbtx::Value::Compound(internal_components) = internal_components else {
        panic!("internalComponents is not a compound");
    };
    let component = internal_components
        .get("EntityStorageKeyComponent".as_bytes())
        .expect("add_new must have created EntityStorageKeyComponent");
    let nbtx::Value::Compound(component) = component else {
        panic!("EntityStorageKeyComponent is not a compound");
    };
    match component.get("StorageKey".as_bytes()) {
        Some(nbtx::Value::String(key)) => assert_eq!(key.as_slice(), storage_key.as_slice()),
        other => panic!("embedded StorageKey missing or wrong type: {other:?}"),
    }

    let _report = level.put_chunk(dimension, position, &chunk).unwrap();

    drop(level);
    let level = Level::open(&world_dir).unwrap();

    let record_bytes = level
        .raw()
        .get(actorprefix_key_bytes(&storage_key))
        .unwrap()
        .expect("the added actor's record must be findable under its derived key after reopen");
    let mut cursor = Cursor::new(record_bytes.as_ref());
    let decoded: nbtx::Value = nbtx::from_le_bytes(&mut cursor).unwrap();
    let nbtx::Value::Compound(decoded_fields) = &decoded else {
        panic!("decoded record is not a compound");
    };
    assert_eq!(
        decoded_fields.get("UniqueID".as_bytes()),
        Some(&nbtx::Value::Long(expected_unique_id))
    );
    // The fourth post-reopen fact: the embedded StorageKey bytes survived
    // the round trip too, still equal to the derived key -- not just the
    // UniqueID and the digest membership checked above/below.
    let reopened_internal_components = decoded_fields
        .get("internalComponents".as_bytes())
        .expect("internalComponents must survive the round trip");
    let nbtx::Value::Compound(reopened_internal_components) = reopened_internal_components else {
        panic!("internalComponents is not a compound after reopen");
    };
    let reopened_component = reopened_internal_components
        .get("EntityStorageKeyComponent".as_bytes())
        .expect("EntityStorageKeyComponent must survive the round trip");
    let nbtx::Value::Compound(reopened_component) = reopened_component else {
        panic!("EntityStorageKeyComponent is not a compound after reopen");
    };
    match reopened_component.get("StorageKey".as_bytes()) {
        Some(nbtx::Value::String(key)) => assert_eq!(key.as_slice(), storage_key.as_slice()),
        other => panic!("embedded StorageKey missing or wrong type after reopen: {other:?}"),
    }

    let reopened_chunk = level.chunk(dimension, position).unwrap().unwrap();
    assert!(
        reopened_chunk.actors().digest_keys().contains(&storage_key),
        "the reopened chunk's digp digest must name the new actor's storage key"
    );
}

/// `ChunkActors::add_new` refuses a record that already has a `UniqueID`,
/// leaving `self` untouched -- never silently overwriting an existing id.
#[test]
fn add_new_refuses_a_record_that_already_has_a_unique_id() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);
    let mut level = Level::open_rw(&world_dir).unwrap();
    let mut session = level.begin_actor_session().unwrap();

    let mut chunk_actors = ChunkActors::default();
    let mut root = nbtx::Compound::default();
    root.insert("UniqueID".into(), nbtx::Value::Long(-42));
    let record = nbtx::Value::Compound(root);

    let err = chunk_actors
        .add_new(&mut session, record)
        .expect_err("a record that already has a UniqueID must be refused");
    assert!(matches!(err, Error::Invalid(_)));
    assert!(chunk_actors.is_empty());
}

/// `Level::begin_actor_session` refuses to reserve (or write) anything on a
/// read-only handle, matching `Level::reserve_actor_session_id`'s own
/// read-only guard -- `begin_actor_session` must not weaken it.
#[test]
fn begin_actor_session_rejects_a_read_only_handle() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let before = std::fs::read(world_dir.join("level.dat")).unwrap();

    let mut level = Level::open(&world_dir).unwrap();
    assert!(level.begin_actor_session().is_err());

    let after = std::fs::read(world_dir.join("level.dat")).unwrap();
    assert_eq!(
        before, after,
        "a rejected begin_actor_session must not touch level.dat"
    );
}

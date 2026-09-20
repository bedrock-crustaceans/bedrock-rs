//! `LevelSettings::{next_actor_session_id, reserve_actor_session_id}` and
//! `Level::reserve_actor_session_id` -- the level.dat "actor counter"
//! bookkeeping the Phase 5 "Write-side maintenance for actors" task left
//! open (`crates/level/TODO.md`).
//!
//! Three things are pinned here:
//!
//! - The corpus fact the plumbing is built on: across the fourteen real
//!   worlds, the four that carry `actorprefix` records with a decodable
//!   `UniqueID` all satisfy this exactly: `next_actor_session_id()` equals
//!   the lowest `UniqueID >> 32` in use, minus one. This is what
//!   `LevelSettings::next_actor_session_id`'s doc comment cites; this test
//!   is what keeps that citation honest against the fixture set as it
//!   stands today.
//! - The same relationship holds on the legacy `0x32` side of the format
//!   too, exhaustive over all ten real fixtures that carry a legacy inline
//!   entity with a `UniqueID` -- checked directly against those entities'
//!   `UniqueID`s rather than assumed from the modern-scheme finding above.
//! - The plumbing itself round-trips: reserving a session id advances the
//!   counter, persists immediately (not deferred to `Level::save`), and a
//!   fresh reopen of the same `db/` sees the reservation -- including the
//!   refusal (not a silent wraparound) once the counter is exhausted at
//!   `i32::MIN`.

mod support;

use std::fs::File;

use std::io::Cursor;

use bedrock_level::actor::{decode_actor_record, decode_legacy_entities, parse_actorprefix_key};
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

/// Extracts `fixture`'s tarball into a fresh `TempDir` and returns the path
/// to the unpacked world directory (holding `level.dat` and `db/`) --
/// mirrors `tests/actor_write.rs::extract_fixture`, needed here (rather than
/// `Fixture::open`) because `Level::open_rw` wants a directory, not an
/// already-open `Database` handle.
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
/// pattern as `tests/actor_digest_version.rs::open_1_26_world`.
fn extract_1_26_world() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz = File::open("tests/level.tar.gz").expect("seed missing");
    Archive::new(GzDecoder::new(tar_gz))
        .unpack(tmp.path())
        .expect("failed to unpack seed");
    let world_dir = tmp.path().join("debug");
    (tmp, world_dir)
}

/// The lowest (most negative) `UniqueID >> 32` session id among every
/// `actorprefix` record in `db` whose value decodes to NBT with a `Long`
/// `UniqueID` field. `None` if the world carries no such record.
fn min_session_id_in_use(db: &Database) -> Option<i32> {
    let mut min: Option<i32> = None;
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        if parse_actorprefix_key(kv.key().as_ref()).is_none() {
            continue;
        }
        let Some(bytes) = db.get(kv.key().as_ref()).expect("db get") else {
            continue;
        };
        let Ok(value) = decode_actor_record(&bytes) else {
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
            let session = (uid >> 32) as i32;
            min = Some(min.map_or(session, |m: i32| m.min(session)));
        }
    }
    min
}

/// The lowest (most negative) `UniqueID >> 32` session id among every
/// legacy inline `0x32` entity across every chunk in `db` whose entry
/// decodes with a `Long` `UniqueID` field. `None` if the world carries no
/// such entity. The `0x32`-era counterpart to [`min_session_id_in_use`]:
/// the ten fixtures that predate the `digp`/`actorprefix` scheme still
/// carry entities, just inline on the chunk record rather than as separate
/// `actorprefix` keys (see `crate::actor`'s module docs), and those
/// entities' `UniqueID`s are governed by the same `worldStartCount`
/// relationship -- confirmed below, not assumed.
fn min_session_id_in_use_legacy(db: &Database) -> Option<i32> {
    let mut min: Option<i32> = None;
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data != KeyVariant::Entity {
            continue;
        }
        let Some(bytes) = db.get(key_bytes.as_ref()).expect("db get") else {
            continue;
        };
        let Ok(values) = decode_legacy_entities(&bytes) else {
            continue;
        };
        for value in values {
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
                let session = (uid >> 32) as i32;
                min = Some(min.map_or(session, |m: i32| m.min(session)));
            }
        }
    }
    min
}

/// Per-world corpus pin: `(world name, expected worldStartCount-derived next
/// session id, expected minimum session id actually used by an actor)`.
/// Only the four worlds that carry `actorprefix` records with a decodable
/// `UniqueID` are listed here -- the other ten real fixtures carry zero
/// `digp`/`actorprefix` records (legacy `0x32`-only), and are checked
/// separately by [`next_actor_session_id_holds_for_every_legacy_0x32_world`]
/// against [`min_session_id_in_use_legacy`] instead.
const WORLDS_WITH_ACTOR_SESSIONS: &[(&str, i32, i32)] = &[
    ("v1_18_30", -5, -4),
    ("v1_19_30", -14, -13),
    ("v1_20_81", -4, -3),
];

/// Per-world corpus pin for the ten legacy `0x32`-only fixtures: `(world
/// name, expected next session id, expected minimum session id actually
/// used by a legacy inline entity)`. All ten of this corpus's legacy-scheme
/// worlds carry the relationship, exhaustive rather than a sample -- see
/// [`next_actor_session_id_holds_for_every_legacy_0x32_world`] for the
/// check this table backs.
const LEGACY_WORLDS_WITH_ACTOR_SESSIONS: &[(&str, i32, i32)] = &[
    ("v1_12", -16, -15),
    ("v1_16", -2, -1),
    ("v1_17_20", -2, -1),
    ("v1_17_20_caves_and_cliffs", -2, -1),
    ("v1_17_40", -2, -1),
    ("v1_17_40_caves_and_cliffs", -2, -1),
    ("v1_17_40_superflat", -2, -1),
    ("v1_18", -13, -12),
    ("v1_18_superflat", -2, -1),
    ("v1_18_updated_superflat", -3, -2),
];

/// The relationship `LevelSettings::next_actor_session_id`'s doc comment
/// cites: for every real world that has actors under the modern
/// `digp`/`actorprefix` scheme, the settings-derived "next session id" is
/// exactly one more negative than the most negative session id any of that
/// world's actors actually uses. Exhaustive over the fixture corpus (plus
/// the 1.26 seed world, checked separately below since it is not
/// index-catalogued), not a sample.
#[test]
fn next_actor_session_id_is_one_below_the_lowest_session_actually_used() {
    let mut checked = 0usize;
    for &(name, expected_next, expected_min) in WORLDS_WITH_ACTOR_SESSIONS {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();

        let level_dat = std::fs::read(fixture.level_dat_path(&_tmp))
            .unwrap_or_else(|e| panic!("{name}: read level.dat: {e}"));
        let settings = LevelSettings::read(level_dat.as_slice())
            .unwrap_or_else(|e| panic!("{name}: settings read: {e}"));

        let next = settings.next_actor_session_id();
        let min_used =
            min_session_id_in_use(&db).unwrap_or_else(|| panic!("{name}: no actor sessions"));

        assert_eq!(next, expected_next, "{name}: next_actor_session_id");
        assert_eq!(min_used, expected_min, "{name}: min session id in use");
        assert_eq!(
            next,
            min_used - 1,
            "{name}: next_actor_session_id must be exactly one below the lowest session in use"
        );
        checked += 1;
    }
    assert_eq!(checked, WORLDS_WITH_ACTOR_SESSIONS.len());
}

/// Same relationship, pinned separately for the 1.26 seed world (89 actors,
/// five distinct non-contiguous sessions including a `0` session -- the
/// world this relationship is *least* trivially true for in the corpus,
/// since its sessions are not a contiguous run down from -1 the way the
/// other three worlds' are).
#[test]
fn next_actor_session_id_holds_for_the_1_26_seed_world_despite_non_contiguous_sessions() {
    let (_tmp, world_dir) = extract_1_26_world();
    let level_dat = std::fs::read(world_dir.join("level.dat")).expect("read 1.26 seed level.dat");
    let settings = LevelSettings::read(level_dat.as_slice()).expect("settings read");

    let db = Database::open(world_dir.join("db").to_str().unwrap()).expect("db open");
    let min_used = min_session_id_in_use(&db).expect("1.26 seed world has actor sessions");

    assert_eq!(settings.next_actor_session_id(), -10);
    assert_eq!(min_used, -9);
    assert_eq!(settings.next_actor_session_id(), min_used - 1);
}

/// The same relationship holds on the legacy `0x32` side of the format,
/// exhaustive over all ten real fixtures that carry a legacy inline entity
/// with a `UniqueID` (every real fixture that predates the
/// `digp`/`actorprefix` scheme, per `crate::actor`'s module docs). Not
/// assumed from the modern-scheme finding above -- a session id allocated
/// for an inline `0x32` entity is drawn from the very same `worldStartCount`
/// field, and this checks that directly rather than inferring it.
#[test]
fn next_actor_session_id_holds_for_every_legacy_0x32_world() {
    let mut checked = 0usize;
    for &(name, expected_next, expected_min) in LEGACY_WORLDS_WITH_ACTOR_SESSIONS {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();

        let level_dat = std::fs::read(fixture.level_dat_path(&_tmp))
            .unwrap_or_else(|e| panic!("{name}: read level.dat: {e}"));
        let settings = LevelSettings::read(level_dat.as_slice())
            .unwrap_or_else(|e| panic!("{name}: settings read: {e}"));

        let next = settings.next_actor_session_id();
        let min_used = min_session_id_in_use_legacy(&db)
            .unwrap_or_else(|| panic!("{name}: no legacy actor sessions"));

        assert_eq!(next, expected_next, "{name}: next_actor_session_id");
        assert_eq!(
            min_used, expected_min,
            "{name}: min legacy session id in use"
        );
        assert_eq!(
            next,
            min_used - 1,
            "{name}: next_actor_session_id must be exactly one below the lowest legacy session \
             in use"
        );
        checked += 1;
    }
    assert_eq!(checked, LEGACY_WORLDS_WITH_ACTOR_SESSIONS.len());
}

/// `LevelSettings::reserve_actor_session_id` claims the current
/// `next_actor_session_id` and leaves the counter one further negative, so
/// immediately calling `next_actor_session_id` again shows the advanced
/// value rather than handing out the same session twice.
#[test]
fn reserve_actor_session_id_advances_the_in_memory_counter() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, _db) = fixture.open();
    let level_dat = std::fs::read(fixture.level_dat_path(&_tmp)).unwrap();
    let mut settings = LevelSettings::read(level_dat.as_slice()).unwrap();

    let before = settings.next_actor_session_id();
    let claimed = settings.reserve_actor_session_id().unwrap();
    let after = settings.next_actor_session_id();

    assert_eq!(claimed, before);
    assert_eq!(after, before - 1);

    let claimed_again = settings.reserve_actor_session_id().unwrap();
    assert_eq!(claimed_again, after);
    assert_eq!(settings.next_actor_session_id(), before - 2);
}

/// `reserve_actor_session_id` round-trips through the on-disk zero-extended
/// encoding: claiming a session, writing, and reading back gives the same
/// advanced counter -- exercised down through several sessions crossing a
/// couple of `i32` byte boundaries, not just the single decrement the corpus
/// worlds happen to show.
#[test]
fn reserve_actor_session_id_round_trips_through_write_and_read() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, _db) = fixture.open();
    let level_dat = std::fs::read(fixture.level_dat_path(&_tmp)).unwrap();
    let mut settings = LevelSettings::read(level_dat.as_slice()).unwrap();

    let mut claimed_ids = Vec::new();
    for _ in 0..300 {
        claimed_ids.push(settings.reserve_actor_session_id().unwrap());
    }

    // Strictly decreasing by exactly one each time, and never repeating --
    // the property that actually makes this collision-safe.
    for pair in claimed_ids.windows(2) {
        assert_eq!(pair[1], pair[0] - 1);
    }
    assert_eq!(
        claimed_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        claimed_ids.len()
    );

    let mut out = Vec::new();
    settings.write(&mut out).unwrap();
    let reread = LevelSettings::read(out.as_slice()).unwrap();
    assert_eq!(
        reread.next_actor_session_id(),
        *claimed_ids.last().unwrap() - 1
    );
}

/// `reserve_actor_session_id` refuses to hand out a session once the
/// counter is already `i32::MIN` -- the one value [`i32::checked_sub`]
/// cannot decrement -- rather than silently wrapping to `i32::MAX` and
/// handing out a *positive* high 32 bits, which would violate the
/// always-negative-and-never-yet-used invariant every other test here
/// relies on. Seeds `world_start_count` directly at the boundary (`i32::MIN`
/// zero-extended into the on-disk `i64` representation) rather than looping
/// billions of times to reach it.
#[test]
fn reserve_actor_session_id_refuses_to_wrap_past_i32_min() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, _db) = fixture.open();
    let level_dat = std::fs::read(fixture.level_dat_path(&_tmp)).unwrap();
    let mut settings = LevelSettings::read(level_dat.as_slice()).unwrap();

    settings.world_start_count = (i32::MIN as u32) as i64;
    assert_eq!(settings.next_actor_session_id(), i32::MIN);

    let result = settings.reserve_actor_session_id();
    assert!(matches!(result, Err(Error::Invalid(_))));

    // The counter must be left untouched by a refused reservation -- a
    // caller retrying (or just reading the state back) must not see it
    // silently advanced despite the error.
    assert_eq!(settings.next_actor_session_id(), i32::MIN);
}

/// End to end through `Level`: `Level::reserve_actor_session_id` persists
/// `level.dat` immediately, not deferred to `Level::save` -- a fresh
/// `Level::open` on the same directory (no `save` call in between) sees the
/// reservation already advanced past what a caller was handed.
#[test]
fn level_reserve_actor_session_id_persists_immediately_without_save() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let claimed = {
        let mut level = Level::open_rw(&world_dir).unwrap();
        level.reserve_actor_session_id().unwrap()
        // Deliberately no `level.save()` call -- and `level` is dropped
        // here, exercising that the write already reached disk.
    };

    // `Level` does not expose `LevelSettings` directly; re-read level.dat
    // from disk the same way the corpus tests do, to check what actually
    // landed rather than trusting the handle's own in-memory state. No
    // `Level` handle is open on `world_dir` at this point -- `level` above
    // was dropped at the end of its block, and a raw file read takes no
    // database lock -- so this does not race the next handle opened below.
    let level_dat = std::fs::read(world_dir.join("level.dat")).unwrap();
    let settings = LevelSettings::read(level_dat.as_slice()).unwrap();

    assert_eq!(settings.next_actor_session_id(), claimed - 1);

    // A second reservation on a freshly reopened handle continues from
    // there, never repeating the first handle's claim. Only one `Level`
    // handle is ever open on `world_dir` at a time -- `Database` holds an
    // exclusive lock on `db/` for as long as a handle is open (see
    // `tests/actor_write.rs::extract_fixture`'s own note on this).
    let mut reopened_rw = Level::open_rw(&world_dir).unwrap();
    let second = reopened_rw.reserve_actor_session_id().unwrap();
    assert_eq!(second, claimed - 1);
}

/// `Level::reserve_actor_session_id` refuses to reserve (or write) anything
/// on a read-only handle, matching `Level::save`'s and `Level::put_chunk`'s
/// own read-only guard.
#[test]
fn level_reserve_actor_session_id_rejects_a_read_only_handle() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, world_dir) = extract_fixture(&fixture);

    let before = std::fs::read(world_dir.join("level.dat")).unwrap();

    let mut level = Level::open(&world_dir).unwrap();
    assert!(level.reserve_actor_session_id().is_err());

    let after = std::fs::read(world_dir.join("level.dat")).unwrap();
    assert_eq!(
        before, after,
        "a rejected reservation must not touch level.dat"
    );
}

//! Write-side actor maintenance (`crate::level_chunk::Chunk::actors_mut`,
//! `Chunk::encode_actor_writes`, `Chunk::migrate_legacy_entities`) exercised
//! against real fixture chunks, plus the byte-identity guarantee an
//! unmodified chunk's `digp` record must keep even though it is now
//! recomputed on every encode rather than passed through untouched -- see
//! `Chunk::encode_at`'s doc comment for the policy this pins.

mod support;

use std::collections::HashSet;
use std::fs::File;
use std::io::Cursor;

use bedrock_level::actor::{
    ActorStorageKey, actorprefix_key_bytes, decode_actor_digest, decode_actor_record,
    encode_actor_digest,
};
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level::Level;
use bedrock_level::level_chunk::Chunk;
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;
use flate2::read::GzDecoder;
use tar::Archive;

use support::Fixture;

/// Unpacks `fixture`'s tarball into a fresh `TempDir`, mirroring
/// `tests/level_test.rs`'s helper of the same name: a `Level` opens its own
/// `Database` handle on `db/`, which holds an exclusive lock, so a test that
/// wants a `Level` must not also hold `Fixture::open`'s own `Database`
/// handle open on the same directory at the same time.
fn extract_fixture(fixture: &Fixture) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz_path = format!("tests/fixtures/{}", fixture.tarball);
    let tar_gz = File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
    let tar = GzDecoder::new(tar_gz);
    Archive::new(tar)
        .unpack(tmp.path())
        .unwrap_or_else(|e| panic!("{tar_gz_path}: failed to unpack: {e}"));
    tmp
}

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// The first `(chunk, dimension)` whose `digp` digest decodes to at least
/// `min_entries` entries, plus that raw digest's own bytes.
fn find_chunk_with_digest_len_at_least(
    db: &Database,
    min_entries: usize,
) -> (ChunkPosition, Dimension, Vec<u8>) {
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let mut cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data != KeyVariant::ActorDigest {
            continue;
        }
        let bytes = kv.value().as_ref().to_vec();
        if decode_actor_digest(&bytes).unwrap().len() >= min_entries {
            return (key.chunk, key.dimension, bytes);
        }
    }
    panic!("no chunk with a digp digest of at least {min_entries} entries found");
}

/// The first `(chunk, dimension)` whose `digp` key is present but decodes
/// to zero entries -- the common "chunk with no actors, but the digest key
/// still got written" shape (see `crate::actor`'s corpus counts: 793 keys
/// but only 108 entries in `v1_18_30`).
fn find_chunk_with_an_empty_present_digest(db: &Database) -> (ChunkPosition, Dimension) {
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let mut cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::ActorDigest && kv.value().as_ref().is_empty() {
            return (key.chunk, key.dimension);
        }
    }
    panic!("no chunk with an empty-but-present digp key found");
}

/// Every `(chunk, dimension, raw digp value bytes)` in `db` -- every chunk
/// that carries a `digp` key at all, empty or not.
fn all_digest_bearing_chunks(db: &Database) -> Vec<(ChunkPosition, Dimension, Vec<u8>)> {
    let mut out = Vec::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let mut cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::ActorDigest {
            out.push((key.chunk, key.dimension, kv.value().as_ref().to_vec()));
        }
    }
    out
}

/// Every `actorprefix` storage key present in `db`, regardless of which (if
/// any) `digp` names it.
fn all_actorprefix_keys(db: &Database) -> HashSet<ActorStorageKey> {
    let mut out = HashSet::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        if let Some(storage_key) = bedrock_level::actor::parse_actorprefix_key(kv.key().as_ref()) {
            out.insert(storage_key);
        }
    }
    out
}

/// Every storage key named by *any* chunk's `digp` digest in `db`.
fn all_digest_referenced_keys(db: &Database) -> HashSet<ActorStorageKey> {
    let mut out = HashSet::new();
    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let mut cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.data == KeyVariant::ActorDigest {
            out.extend(decode_actor_digest(kv.value().as_ref()).unwrap());
        }
    }
    out
}

fn decode_chunk(db: &Database, position: ChunkPosition, dimension: Dimension) -> Chunk {
    let records = ChunkRecords::read(db, position, dimension)
        .expect("read chunk records")
        .expect("chunk present");
    Chunk::decode(records, db).expect("chunk decodes")
}

/// Removing one resolved actor from a real chunk shrinks the recomputed
/// `digp` digest by exactly one entry and reports exactly that entry's
/// storage key in the deletion list -- the write-side counterpart to the
/// read-side dangling-entry finding.
#[test]
fn removing_an_actor_shrinks_the_digest_and_lists_it_for_deletion() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(&db, 1);
    let original_keys = decode_actor_digest(&digest_bytes).unwrap();

    let mut chunk = decode_chunk(&db, position, dimension);
    assert_eq!(chunk.actors().digest_keys(), original_keys);

    let removed_key = original_keys[0];
    let removed = chunk.actors_mut().remove(&removed_key);
    assert!(removed.is_some());

    let encoded = chunk.encode().unwrap();
    let new_keys = match encoded.get(KeyVariant::ActorDigest) {
        Some(bytes) => decode_actor_digest(bytes).unwrap(),
        None => Vec::new(),
    };
    assert_eq!(new_keys.len(), original_keys.len() - 1);
    assert!(!new_keys.contains(&removed_key));
    assert_eq!(
        new_keys,
        original_keys
            .iter()
            .copied()
            .filter(|k| *k != removed_key)
            .collect::<Vec<_>>()
    );

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.deletions, vec![removed_key]);
    assert!(writes.upserts.is_empty());
}

/// The GC scope named by the Phase 5 task: deletions cover exactly what
/// *this chunk's own* digest used to name and no longer does -- never a
/// pre-existing `actorprefix` orphan the wider database carries that this
/// chunk's digest never referenced in the first place.
#[test]
fn removal_does_not_sweep_up_pre_existing_unrelated_orphans() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();

    // Confirm the premise: this fixture really does carry `actorprefix`
    // orphans unrelated to any chunk's current digest (the read-side task
    // record pins 3,640 of 3,748 in this exact fixture), so "the deletion
    // list doesn't include them" is a meaningful assertion and not
    // vacuously true.
    let actorprefix_keys = all_actorprefix_keys(&db);
    let referenced_keys = all_digest_referenced_keys(&db);
    let orphans: HashSet<ActorStorageKey> = actorprefix_keys
        .difference(&referenced_keys)
        .copied()
        .collect();
    assert!(
        !orphans.is_empty(),
        "fixture has no pre-existing orphans -- premise of this test does not hold"
    );

    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(&db, 1);
    let original_keys = decode_actor_digest(&digest_bytes).unwrap();
    let removed_key = original_keys[0];

    let mut chunk = decode_chunk(&db, position, dimension);
    chunk.actors_mut().remove(&removed_key);

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.deletions, vec![removed_key]);
    for orphan in &orphans {
        assert!(
            !writes.deletions.contains(orphan),
            "deletion list swept up unrelated orphan {orphan:02x?}, which this chunk's own \
             digest never named"
        );
    }
}

/// A dangling digest entry (one naming no `actorprefix` record -- the
/// read-side finding `tests/actor.rs` synthesizes since no real fixture
/// carries one on its own) self-heals on an otherwise **unmodified** encode:
/// nothing removes it explicitly, but recomputing the digest from the
/// resolved set alone never re-includes something that resolved to nothing,
/// and `encode_actor_writes` reports exactly that key as a deletion -- the
/// automatic-GC half of decision 13's "always recomputed" policy, exercised
/// end to end rather than only at the `ChunkActors::write_plan` unit-test
/// level.
#[test]
fn a_synthetic_dangling_entry_self_heals_on_an_unmodified_encode() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(&db, 1);
    let real_entries = decode_actor_digest(&digest_bytes).unwrap();

    let existing_actorprefix_keys = all_actorprefix_keys(&db);
    let mut fake_key: ActorStorageKey = [0xEE; 8];
    while existing_actorprefix_keys.contains(&fake_key) {
        fake_key[7] = fake_key[7].wrapping_add(1);
    }

    let mut entries = real_entries.clone();
    entries.push(fake_key);
    let mut records = ChunkRecords::read(&db, position, dimension)
        .unwrap()
        .expect("chunk present in the scan must be readable");
    records.insert(
        Key {
            chunk: position,
            dimension,
            data: KeyVariant::ActorDigest,
        },
        encode_actor_digest(&entries),
    );

    let chunk = Chunk::decode(records, &db).unwrap();
    assert_eq!(chunk.actors().dangling, vec![fake_key]);
    assert_eq!(chunk.actors().digest_keys(), real_entries);

    // No mutation through `actors_mut` at all -- the self-heal must happen
    // purely from recomputing the digest off the resolved set on encode.
    let encoded = chunk.encode().unwrap();
    let new_digest = decode_actor_digest(encoded.get(KeyVariant::ActorDigest).unwrap()).unwrap();
    assert_eq!(
        new_digest, real_entries,
        "the dangling entry must not survive being recomputed"
    );

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.deletions, vec![fake_key]);
    assert!(writes.upserts.is_empty());
}

/// Adding a synthetic actor with an explicit storage key produces both
/// halves decision 13's recomputed-index policy requires on write: the new
/// key appears in the recomputed `digp` digest, and its NBT record appears
/// in the `actorprefix` upsert list, ready to be applied to a database.
#[test]
fn adding_an_actor_appears_in_both_the_digest_and_the_upsert_list() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(&db, 0);
    let original_keys = decode_actor_digest(&digest_bytes).unwrap();

    let existing_actorprefix_keys = all_actorprefix_keys(&db);
    let mut new_key: ActorStorageKey = [0xEE; 8];
    while existing_actorprefix_keys.contains(&new_key) {
        new_key[7] = new_key[7].wrapping_add(1);
    }

    let value = nbtx::Value::Byte(42);

    let mut chunk = decode_chunk(&db, position, dimension);
    chunk.actors_mut().add(new_key, value.clone());

    let encoded = chunk.encode().unwrap();
    let new_digest = decode_actor_digest(encoded.get(KeyVariant::ActorDigest).unwrap()).unwrap();
    assert_eq!(new_digest.len(), original_keys.len() + 1);
    assert!(new_digest.contains(&new_key));

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.upserts.len(), 1);
    assert_eq!(writes.upserts[0].0, new_key);
    let decoded_value = decode_actor_record(&writes.upserts[0].1).unwrap();
    assert_eq!(decoded_value, value);
    assert!(writes.deletions.is_empty());
}

/// A chunk decoded and re-encoded with no actor edits at all reproduces its
/// `digp` value byte-for-byte, and reports an empty write plan -- decision
/// 13's "always recomputed" digest policy must not, by itself, disturb a
/// chunk nothing touched. Exhaustive over every `digp`-bearing chunk (empty
/// or not) in each of the three digest-era fixtures, mirroring how the
/// raw-layer round-trip tests in `tests/actor.rs` check every real record
/// rather than a sample -- the recompute path gets the same exhaustive bar.
/// `ChunkRecords::read_many` keeps this to one linear scan per fixture
/// rather than one per chunk, so runtime stays modest.
#[test]
fn an_unmodified_chunk_reproduces_its_digest_byte_identically() {
    let mut total_checked = 0usize;

    for fixture_name in ["v1_18_30", "v1_19_30", "v1_20_81"] {
        let fixture = fixture_named(fixture_name);
        let (_tmp, db) = fixture.open();

        let chunks = all_digest_bearing_chunks(&db);
        assert!(
            !chunks.is_empty(),
            "{fixture_name}: no digp-bearing chunks found"
        );

        let wanted = chunks
            .iter()
            .map(|(position, dimension, _)| (*position, *dimension));
        let mut groups = ChunkRecords::read_many(&db, wanted).unwrap();

        for (position, dimension, digest_bytes) in &chunks {
            let records = groups.remove(&(*position, *dimension)).unwrap_or_else(|| {
                panic!("{fixture_name}: {position:?}/{dimension:?} missing from read_many")
            });
            let chunk = Chunk::decode(records, &db).unwrap_or_else(|e| {
                panic!("{fixture_name}: {position:?}/{dimension:?}: decode failed: {e}")
            });

            let encoded = chunk.encode().unwrap();
            match encoded.get(KeyVariant::ActorDigest) {
                Some(new_bytes) => assert_eq!(
                    new_bytes,
                    digest_bytes.as_slice(),
                    "{fixture_name}: {position:?}/{dimension:?}: unmodified digp did not \
                     round-trip byte-identically"
                ),
                None => assert!(
                    digest_bytes.is_empty(),
                    "{fixture_name}: {position:?}/{dimension:?}: digp key vanished though its \
                     original value was nonempty"
                ),
            }

            let writes = chunk.encode_actor_writes().unwrap();
            assert!(
                writes.is_empty(),
                "{fixture_name}: {position:?}/{dimension:?}: unmodified chunk produced a \
                 nonempty write plan"
            );

            total_checked += 1;
        }
    }

    assert!(
        total_checked > 0,
        "digest byte-identity check ran against zero chunks"
    );
}

/// A chunk whose `digp` key is present but already empty on disk keeps that
/// key present (and empty) through an unmodified encode, rather than having
/// it deleted -- deletion is reserved for a digest that genuinely *shrank*
/// to nothing because actors were removed (see `Chunk::encode_at`'s doc
/// comment), not for a chunk that never had any to begin with.
#[test]
fn an_unmodified_chunk_with_an_already_empty_digest_key_keeps_it_present() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let (position, dimension) = find_chunk_with_an_empty_present_digest(&db);

    let chunk = decode_chunk(&db, position, dimension);
    assert!(chunk.actors().digest_keys().is_empty());

    let encoded = chunk.encode().unwrap();
    assert!(
        encoded.contains(KeyVariant::ActorDigest),
        "an already-empty digp key must not be deleted on an unmodified encode"
    );
    assert!(encoded.get(KeyVariant::ActorDigest).unwrap().is_empty());
}

/// Removing every actor a chunk has (rather than merely one of several)
/// deletes the `digp` key entirely on encode, per the Phase 5 "deleting
/// empty digp" task -- distinct from the previous test's "was always empty"
/// case, which must *not* delete.
#[test]
fn removing_every_actor_deletes_the_digest_key_entirely() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(&db, 1);
    let original_keys = decode_actor_digest(&digest_bytes).unwrap();

    let mut chunk = decode_chunk(&db, position, dimension);
    for key in &original_keys {
        chunk.actors_mut().remove(key);
    }
    assert!(chunk.actors().digest_keys().is_empty());

    let encoded = chunk.encode().unwrap();
    assert!(
        !encoded.contains(KeyVariant::ActorDigest),
        "digp key should have been deleted once every actor was removed"
    );

    let writes = chunk.encode_actor_writes().unwrap();
    let mut deletions = writes.deletions.clone();
    deletions.sort();
    let mut expected = original_keys.clone();
    expected.sort();
    assert_eq!(deletions, expected);
}

/// `Chunk::migrate_legacy_entities` moves a real chunk's legacy inline
/// `0x32` entities into the modern scheme: each entity gets the
/// caller-supplied key in order, the `0x32` record is cleared on encode, and
/// every migrated entity shows up as an `actorprefix` upsert.
#[test]
fn migrate_legacy_entities_moves_a_real_chunks_0x32_list_into_the_modern_scheme() {
    let fixture = fixture_named("v1_18");
    let (_tmp, db) = fixture.open();

    let (position, dimension) = {
        let mut keys = db.keys().expect("iterate database");
        let mut found = None;
        for kv in &mut keys {
            let mut cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };
            if key.data == KeyVariant::Entity && !kv.value().as_ref().is_empty() {
                found = Some((key.chunk, key.dimension));
                break;
            }
        }
        found.expect("v1_18 has at least one chunk with a nonempty 0x32 record")
    };

    let mut chunk = decode_chunk(&db, position, dimension);
    let legacy_count = chunk.actors().legacy.len();
    assert!(legacy_count > 0);
    assert!(chunk.actors().digest_keys().is_empty());

    let keys: Vec<ActorStorageKey> = (0..legacy_count as u64).map(|i| i.to_be_bytes()).collect();

    let migrated = chunk.migrate_legacy_entities(keys.clone()).unwrap();
    assert_eq!(migrated, legacy_count);
    assert!(chunk.actors().legacy.is_empty());
    assert_eq!(chunk.actors().digest_keys(), keys);

    let encoded = chunk.encode().unwrap();
    assert!(
        !encoded.contains(KeyVariant::Entity),
        "the legacy 0x32 record must be cleared after migration"
    );
    assert!(encoded.contains(KeyVariant::ActorDigest));

    let writes = chunk.encode_actor_writes().unwrap();
    assert_eq!(writes.upserts.len(), legacy_count);
    assert!(writes.deletions.is_empty());
}

/// End to end through `Level`, not `Chunk` directly: removing an actor and
/// calling `Level::put_chunk` applies both halves without the caller having
/// to know `Chunk::encode_actor_writes` exists -- the `actorprefix` record
/// for the removed actor is actually gone from the database, and reopening
/// the level fresh confirms the shrunk digest was actually persisted, not
/// just held in the in-memory `Chunk`.
#[test]
fn level_put_chunk_applies_an_actor_removal_end_to_end() {
    let fixture = fixture_named("v1_18_30");
    let tmp = extract_fixture(&fixture);

    let (position, dimension, removed_key, original_len) = {
        let mut level = Level::open_rw(tmp.path().join("world")).unwrap();
        let (position, dimension, digest_bytes) =
            find_chunk_with_digest_len_at_least(level.raw(), 1);
        let original_keys = decode_actor_digest(&digest_bytes).unwrap();
        let removed_key = original_keys[0];

        assert!(
            level
                .raw()
                .get(actorprefix_key_bytes(&removed_key))
                .unwrap()
                .is_some(),
            "the actorprefix record must exist before the edit"
        );

        let mut chunk = level.chunk(dimension, position).unwrap().unwrap();
        chunk.actors_mut().remove(&removed_key);
        level.put_chunk(dimension, position, &chunk).unwrap();

        assert!(
            level
                .raw()
                .get(actorprefix_key_bytes(&removed_key))
                .unwrap()
                .is_none(),
            "put_chunk must have deleted the removed actor's actorprefix record"
        );

        (position, dimension, removed_key, original_keys.len())
    };

    // Reopen fresh -- a new `Database` handle over the same `db/` directory
    // -- to confirm the edit actually reached disk rather than only this
    // process's in-memory state.
    let level = Level::open(tmp.path().join("world")).unwrap();
    let reopened = level.chunk(dimension, position).unwrap().unwrap();
    let new_keys = reopened.actors().digest_keys();
    assert_eq!(new_keys.len(), original_len - 1);
    assert!(!new_keys.contains(&removed_key));
    assert!(
        level
            .raw()
            .get(actorprefix_key_bytes(&removed_key))
            .unwrap()
            .is_none(),
        "the deletion must have survived a reopen"
    );
}

/// `Level::remove_chunk` deletes the removed chunk's own `actorprefix`
/// records along with the rest of its `ChunkRecords` group -- every storage
/// key its `digp` named, not merely the group itself.
#[test]
fn level_remove_chunk_deletes_its_own_actor_records_too() {
    let fixture = fixture_named("v1_18_30");
    let tmp = extract_fixture(&fixture);
    let mut level = Level::open_rw(tmp.path().join("world")).unwrap();

    let (position, dimension, digest_bytes) = find_chunk_with_digest_len_at_least(level.raw(), 1);
    let original_keys = decode_actor_digest(&digest_bytes).unwrap();
    assert!(!original_keys.is_empty());

    for storage_key in &original_keys {
        assert!(
            level
                .raw()
                .get(actorprefix_key_bytes(storage_key))
                .unwrap()
                .is_some(),
            "actorprefix {storage_key:02x?} must exist before remove_chunk"
        );
    }

    level.remove_chunk(dimension, position).unwrap();

    assert!(level.chunk(dimension, position).unwrap().is_none());
    for storage_key in &original_keys {
        assert!(
            level
                .raw()
                .get(actorprefix_key_bytes(storage_key))
                .unwrap()
                .is_none(),
            "actorprefix {storage_key:02x?} survived remove_chunk"
        );
    }
}

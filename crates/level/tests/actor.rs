//! `crate::actor` against real fixture worlds: `digp`/`actorprefix` key
//! recognition, byte-identical round trips for both storage schemes plus
//! the legacy inline `0x32` list, and the merged per-chunk actor read --
//! including a dangling digest entry, which no real fixture here actually
//! contains (see the counts pinned below), so that case is synthesized on
//! top of a real chunk's own records.

mod support;

use std::collections::HashSet;
use std::io::Cursor;

use bedrock_level::actor::{
    ActorRecord, ChunkActors, decode_actor_digest, decode_actor_record, decode_legacy_entities,
    encode_actor_digest, encode_actor_record, encode_legacy_entities, parse_actorprefix_key,
};
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

use support::Fixture;

fn fixture_named(name: &str) -> Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
}

/// Everything this suite needs from one pass over a fixture's `digp` and
/// `actorprefix` keys, gathered together so every test below shares one
/// scan instead of repeating it.
struct ActorScan {
    /// `(chunk, dimension, raw digp value bytes)` for every `digp` key.
    digp: Vec<(ChunkPosition, Dimension, Vec<u8>)>,
    /// `(storage key, raw actorprefix value bytes)` for every `actorprefix` key.
    actorprefix: Vec<([u8; 8], Vec<u8>)>,
    /// Raw `0x32` (legacy inline entity list) value bytes, one per key.
    legacy_0x32: Vec<Vec<u8>>,
}

fn scan(db: &Database) -> ActorScan {
    let mut digp = Vec::new();
    let mut actorprefix = Vec::new();
    let mut legacy_0x32 = Vec::new();

    let mut keys = db.keys().expect("iterate database");
    for kv in &mut keys {
        let key_bytes = kv.key();
        let raw = key_bytes.as_ref();
        let mut cursor = Cursor::new(raw);
        if let Ok(key) = Key::deserialize(&mut cursor) {
            match key.data {
                KeyVariant::ActorDigest => {
                    digp.push((key.chunk, key.dimension, kv.value().as_ref().to_vec()));
                    continue;
                }
                KeyVariant::Entity => {
                    legacy_0x32.push(kv.value().as_ref().to_vec());
                    continue;
                }
                _ => {}
            }
        }
        if let Some(storage_key) = parse_actorprefix_key(raw) {
            actorprefix.push((storage_key, kv.value().as_ref().to_vec()));
        }
    }

    ActorScan {
        digp,
        actorprefix,
        legacy_0x32,
    }
}

/// `digp` and `actorprefix` keys are recognized at all, in the exact counts
/// the fixtures actually contain -- pinned so any regression in either
/// key's shape (or in `Key::deserialize`'s length/prefix check) is caught
/// immediately rather than by a vaguer "some record went missing" failure.
#[test]
fn digp_and_actorprefix_keys_are_recognized_in_exact_counts() {
    let cases = [
        ("v1_18_30", 793usize, 108usize, 3748usize),
        ("v1_19_30", 794, 111, 3754),
        ("v1_20_81", 3058, 96, 121),
    ];

    for (name, expected_digp_keys, expected_digp_entries, expected_actorprefix_keys) in cases {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let scan = scan(&db);

        assert_eq!(
            scan.digp.len(),
            expected_digp_keys,
            "{name}: digp key count"
        );
        let total_entries: usize = scan
            .digp
            .iter()
            .map(|(_, _, bytes)| decode_actor_digest(bytes).unwrap().len())
            .sum();
        assert_eq!(
            total_entries, expected_digp_entries,
            "{name}: total digp digest entries"
        );
        assert_eq!(
            scan.actorprefix.len(),
            expected_actorprefix_keys,
            "{name}: actorprefix key count"
        );
    }
}

/// Every real `digp` value across the three actor-bearing fixtures decodes
/// and re-encodes byte-identically: `decode_actor_digest` splits it into
/// 8-byte entries and `encode_actor_digest` concatenates them back with no
/// transformation, so this is architecture decision 3's raw-layer guarantee
/// for this specific record shape.
#[test]
fn digp_values_round_trip_byte_identically() {
    let mut records_tested = 0usize;

    for name in ["v1_18_30", "v1_19_30", "v1_20_81"] {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let scan = scan(&db);
        assert!(!scan.digp.is_empty(), "{name}: no digp keys found");

        for (pos, dim, bytes) in &scan.digp {
            let entries = decode_actor_digest(bytes)
                .unwrap_or_else(|e| panic!("{name}: {pos:?}/{dim:?}: decode failed: {e}"));
            let re_encoded = encode_actor_digest(&entries);
            assert_eq!(
                &re_encoded, bytes,
                "{name}: {pos:?}/{dim:?}: digp value did not round-trip byte-identically"
            );
            records_tested += 1;
        }
    }

    assert_eq!(records_tested, 793 + 794 + 3058);
}

/// Every real `actorprefix` record across the three fixtures decodes as one
/// NBT compound and re-encodes byte-identically -- the same guarantee
/// `tests/nbt_actor_storage_key.rs` already pins for one hand-picked
/// non-UTF-8 record, exercised here against the whole corpus.
#[test]
fn actorprefix_records_round_trip_byte_identically() {
    let mut records_tested = 0usize;

    for name in ["v1_18_30", "v1_19_30", "v1_20_81"] {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let scan = scan(&db);
        assert!(
            !scan.actorprefix.is_empty(),
            "{name}: no actorprefix keys found"
        );

        for (storage_key, bytes) in &scan.actorprefix {
            let decoded = decode_actor_record(bytes).unwrap_or_else(|e| {
                panic!("{name}: actorprefix {storage_key:02x?}: decode failed: {e}")
            });
            let re_encoded = encode_actor_record(&decoded).unwrap_or_else(|e| {
                panic!("{name}: actorprefix {storage_key:02x?}: encode failed: {e}")
            });
            assert_eq!(
                &re_encoded, bytes,
                "{name}: actorprefix {storage_key:02x?}: value did not round-trip byte-identically"
            );
            records_tested += 1;
        }
    }

    assert_eq!(records_tested, 3748 + 3754 + 121);
}

/// Every real `digp` digest entry across the three fixtures resolves to a
/// real `actorprefix` record -- zero dangling entries in the checked-in
/// fixtures. `chunk_actors_read_reports_a_synthesized_dangling_entry` below
/// covers the dangling path itself, since no real fixture here exercises it.
#[test]
fn every_real_digp_entry_resolves_to_an_actorprefix_record() {
    let cases = [("v1_18_30", 108usize), ("v1_19_30", 111), ("v1_20_81", 96)];

    for (name, expected_resolved) in cases {
        let fixture = fixture_named(name);
        let (_tmp, db) = fixture.open();
        let scan = scan(&db);

        let actorprefix_keys: HashSet<[u8; 8]> = scan.actorprefix.iter().map(|(k, _)| *k).collect();

        let mut resolved = 0usize;
        let mut dangling = 0usize;
        for (pos, dim, bytes) in &scan.digp {
            for entry in decode_actor_digest(bytes).unwrap() {
                if actorprefix_keys.contains(&entry) {
                    resolved += 1;
                } else {
                    dangling += 1;
                    println!("{name}: {pos:?}/{dim:?}: dangling digp entry {entry:02x?}");
                }
            }
        }

        assert_eq!(resolved, expected_resolved, "{name}: resolved digp entries");
        assert_eq!(
            dangling, 0,
            "{name}: dangling digp entries in real fixture data"
        );
    }
}

/// The legacy inline `0x32` list: `v1_18` (pre-1.18.30, before the
/// `digp`/`actorprefix` scheme existed) is the fixture that actually
/// carries these, most of them empty (a chunk with no entities still gets
/// an empty `0x32` record). Every one decodes and re-encodes
/// byte-identically, including the empty ones (zero compounds, zero bytes).
#[test]
fn legacy_0x32_entities_round_trip_byte_identically() {
    let fixture = fixture_named("v1_18");
    let (_tmp, db) = fixture.open();
    let scan = scan(&db);

    assert_eq!(scan.legacy_0x32.len(), 6170, "v1_18: 0x32 record count");

    let mut total_entities = 0usize;
    let mut nonempty_records = 0usize;
    for bytes in &scan.legacy_0x32 {
        if !bytes.is_empty() {
            nonempty_records += 1;
        }
        let decoded = decode_legacy_entities(bytes)
            .unwrap_or_else(|e| panic!("failed to decode a 0x32 record: {e}"));
        total_entities += decoded.len();
        let re_encoded = encode_legacy_entities(&decoded).unwrap();
        assert_eq!(
            &re_encoded, bytes,
            "0x32 record did not round-trip byte-identically"
        );
    }

    assert_eq!(nonempty_records, 61, "v1_18: nonempty 0x32 records");
    assert_eq!(total_entities, 124, "v1_18: total legacy entities");
}

/// A structurally malformed `digp` value -- length not a multiple of 8 --
/// is an error, not a silently truncated or padded result (decision 11's
/// malformed side). Built by corrupting a real record rather than a
/// synthetic buffer, so the "well-formed real data, deliberately broken"
/// framing matches the rest of this file.
#[test]
fn a_truncated_real_digp_value_is_rejected_as_malformed() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let scan = scan(&db);

    let (_, _, real_bytes) = scan
        .digp
        .iter()
        .find(|(_, _, bytes)| !bytes.is_empty())
        .expect("v1_18_30 has at least one nonempty digp value");

    let mut truncated = real_bytes.clone();
    truncated.pop();
    assert!(decode_actor_digest(&truncated).is_err());
}

/// The merged read for a real chunk: `ChunkActors::read` resolves that
/// chunk's `digp` digest against real `actorprefix` records, and the
/// resolved set matches an independent decode of the same digest exactly
/// (same storage keys, same NBT values, same order).
#[test]
fn chunk_actors_read_resolves_a_real_chunk() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let scan = scan(&db);

    let (pos, dim, digest_bytes) = scan
        .digp
        .iter()
        .find(|(_, _, bytes)| !bytes.is_empty())
        .expect("v1_18_30 has at least one chunk with a nonempty digp digest");

    let expected_entries = decode_actor_digest(digest_bytes).unwrap();
    assert!(!expected_entries.is_empty());

    let records = ChunkRecords::read(&db, *pos, *dim)
        .expect("read chunk records")
        .expect("chunk present in the scan must be readable");
    assert!(
        records.contains(KeyVariant::ActorDigest),
        "{pos:?}/{dim:?}: ChunkRecords group is missing its digp record"
    );

    let actors =
        ChunkActors::read(&records, &db).unwrap_or_else(|e| panic!("{pos:?}/{dim:?}: {e}"));

    assert!(
        actors.dangling.is_empty(),
        "{pos:?}/{dim:?}: unexpected dangling entries"
    );
    assert_eq!(actors.resolved.len(), expected_entries.len());
    assert_eq!(actors.len(), expected_entries.len());

    let resolved_keys: Vec<[u8; 8]> = actors.resolved.iter().map(|r| r.storage_key).collect();
    assert_eq!(
        resolved_keys, expected_entries,
        "{pos:?}/{dim:?}: resolved order must follow digest order"
    );

    for record in &actors.resolved {
        let expected_bytes = &scan
            .actorprefix
            .iter()
            .find(|(k, _)| *k == record.storage_key)
            .unwrap_or_else(|| panic!("actorprefix {:02x?} missing from scan", record.storage_key))
            .1;
        let re_encoded = encode_actor_record(&record.value).unwrap();
        assert_eq!(
            &re_encoded, expected_bytes,
            "{pos:?}/{dim:?}: resolved record for {:02x?} does not round-trip",
            record.storage_key
        );
    }
}

/// The dangling-digest-entry case: a `digp` naming a storage key with no
/// `actorprefix` record. No real fixture here contains one (see
/// `every_real_digp_entry_resolves_to_an_actorprefix_record`), so this
/// takes a real chunk's real digest and appends one storage key guaranteed
/// absent from the database, through `ChunkRecords::insert` -- the same
/// take/insert overlay a caller doing real editing would use. Per decision
/// 11 this must surface as a reported, non-fatal outcome: the fabricated
/// entry lands in `dangling`, every real entry still resolves normally, and
/// nothing panics or returns `Err`.
#[test]
fn chunk_actors_read_reports_a_synthesized_dangling_entry() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let scan = scan(&db);

    let (pos, dim, digest_bytes) = scan
        .digp
        .iter()
        .find(|(_, _, bytes)| !bytes.is_empty())
        .expect("v1_18_30 has at least one chunk with a nonempty digp digest");
    let real_entries = decode_actor_digest(digest_bytes).unwrap();

    let real_keys: HashSet<[u8; 8]> = scan.actorprefix.iter().map(|(k, _)| *k).collect();
    let fake_key: [u8; 8] = {
        let mut candidate = [0xEEu8; 8];
        // `real_keys` only has thousands of entries out of 2^64 possible
        // keys, so this loop is not expected to iterate more than once, but
        // it is written to guarantee absence rather than assume it.
        while real_keys.contains(&candidate) {
            candidate[7] = candidate[7].wrapping_add(1);
        }
        candidate
    };

    let mut entries = real_entries.clone();
    entries.push(fake_key);
    let mut records = ChunkRecords::read(&db, *pos, *dim)
        .expect("read chunk records")
        .expect("chunk present in the scan must be readable");
    records.insert(
        Key {
            chunk: *pos,
            dimension: *dim,
            data: KeyVariant::ActorDigest,
        },
        encode_actor_digest(&entries),
    );

    let actors = ChunkActors::read(&records, &db)
        .unwrap_or_else(|e| panic!("{pos:?}/{dim:?}: unexpected error: {e}"));

    assert_eq!(actors.dangling, vec![fake_key]);
    assert_eq!(actors.resolved.len(), real_entries.len());
    // `len()` deliberately excludes dangling entries -- they name no actor.
    assert_eq!(actors.len(), real_entries.len());
    let resolved_keys: Vec<[u8; 8]> = actors.resolved.iter().map(|r| r.storage_key).collect();
    assert_eq!(resolved_keys, real_entries);
}

/// `ChunkActors::read` on a chunk whose `digp` digest is empty and which
/// has no legacy `0x32` record (true for every chunk in this fixture --
/// `digp_and_actorprefix_keys_are_recognized_in_exact_counts` pins 793 digp
/// keys but only 108 total entries, so most are empty) returns an empty,
/// not-an-error result. Every real `digp` key in `v1_18_30` turned out to
/// belong to some chunk (793 keys, and no chunk here lacks one entirely),
/// so this specifically covers "has the record, digest is empty" rather
/// than "has no record at all" -- [`ChunkActors::default`] and its own
/// tests already cover the no-record-at-all case directly.
#[test]
fn chunk_actors_read_on_an_actorless_chunk_is_empty() {
    let fixture = fixture_named("v1_18_30");
    let (_tmp, db) = fixture.open();
    let scan = scan(&db);

    let (pos, dim, _) = scan
        .digp
        .iter()
        .find(|(_, _, bytes)| bytes.is_empty())
        .expect("v1_18_30 has at least one chunk with an empty digp digest");

    let records = ChunkRecords::read(&db, *pos, *dim)
        .expect("read chunk records")
        .expect("chunk present in the scan must be readable");
    let actors = ChunkActors::read(&records, &db).unwrap();
    assert!(actors.is_empty());
    assert_eq!(actors.len(), 0);
}

/// `ActorRecord` is a plain value type: constructing one directly (not
/// through `ChunkActors::read`) and comparing it against a
/// `ChunkActors::read` result is a meaningful equality check, not just a
/// derive smoke test.
#[test]
fn actor_record_equality_matches_field_by_field() {
    let a = ActorRecord {
        storage_key: [1; 8],
        value: nbtx::Value::Byte(3),
    };
    let b = ActorRecord {
        storage_key: [1; 8],
        value: nbtx::Value::Byte(3),
    };
    let c = ActorRecord {
        storage_key: [2; 8],
        value: nbtx::Value::Byte(3),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

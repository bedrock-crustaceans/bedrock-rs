//! Evidence for the Phase 2 `LevelChunkMetaDataDictionary` task (see
//! `crates/level/TODO.md`), gathered against every fixture world plus the
//! 1.26 seed world (`tests/level.tar.gz`):
//!
//! - Which worlds actually carry the `LevelChunkMetaDataDictionary` string
//!   key, and their exact entry counts
//!   (`every_real_dictionary_has_the_expected_entry_count`).
//! - `MetadataDictionary::decode` followed by `MetadataDictionary::encode`
//!   reproduces every real dictionary's bytes exactly
//!   (`decode_then_encode_reproduces_every_real_dictionary_byte_for_byte`).
//! - A chunk's `0x3f` `MetadataDictionaryKey` record value is exactly one of the
//!   dictionary's entry keys, for every chunk in every world that carries
//!   the dictionary (`0x3f_link_resolves_for_every_chunk_in_every_dictionary_world`).
mod support;

use std::collections::HashMap;
use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::metadata_dictionary::{self, MetadataDictionary};

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Every `(world, db)` this crate's fixture corpus can offer: the thirteen
/// imported fixtures plus the separately-shipped 1.26 seed world, matching
/// the corpus other Phase 1 evidence files (e.g. `tests/bit_layers.rs`)
/// scan.
fn all_worlds() -> Vec<(String, tempfile::TempDir, Database)> {
    let mut worlds = Vec::new();
    for fixture in support::fixtures() {
        let (tmp, db) = fixture.open();
        worlds.push((fixture.name.clone(), tmp, db));
    }
    let (tmp, db) = open_1_26();
    worlds.push(("1_26_seed".to_string(), tmp, db));
    worlds
}

/// The exact set of worlds found to carry `LevelChunkMetaDataDictionary`,
/// with their entry counts. Established by a direct scan
/// (`db.get(metadata_dictionary::KEY)`) of the whole corpus: every other
/// world in `all_worlds()` -- `v1_12`, `v1_16`, both `v1_17_20` worlds, all
/// three `v1_17_40` worlds, and all three `v1_18` worlds -- has no such key
/// at all. Those are all pre-1.18.30 saves or `v1_18`'s own line, which
/// predates the dictionary; nothing here claims that boundary is exact,
/// only that it is what this corpus shows.
fn expected_counts() -> HashMap<&'static str, u32> {
    HashMap::from([
        ("v1_18_30", 3),
        ("v1_19_30", 7),
        ("v1_20_81", 3),
        ("1_26_seed", 10),
    ])
}

#[test]
fn every_real_dictionary_has_the_expected_entry_count() {
    let expected = expected_counts();
    let mut found: HashMap<String, u32> = HashMap::new();

    for (name, _tmp, db) in all_worlds() {
        if let Some(buf) = db.get(metadata_dictionary::KEY).unwrap() {
            let dict = MetadataDictionary::decode(&buf)
                .unwrap_or_else(|e| panic!("{name}: failed to decode dictionary: {e}"));
            found.insert(name, dict.len() as u32);
        }
    }

    assert_eq!(
        found.len(),
        expected.len(),
        "expected exactly {:?} to carry the dictionary, found {:?}",
        expected.keys().collect::<Vec<_>>(),
        found.keys().collect::<Vec<_>>()
    );
    for (name, count) in &expected {
        assert_eq!(
            found.get(*name).copied(),
            Some(*count),
            "{name}: unexpected entry count"
        );
    }

    // Nonzero guard: every dictionary found actually has entries, and the
    // corpus as a whole is not a false positive from an all-empty scan.
    let total: u32 = found.values().sum();
    assert!(
        total > 0,
        "no dictionary entries found anywhere in the corpus"
    );
    assert_eq!(total, 23, "total entries across the whole corpus");
}

#[test]
fn decode_then_encode_reproduces_every_real_dictionary_byte_for_byte() {
    let mut checked = 0;
    for (name, _tmp, db) in all_worlds() {
        let Some(buf) = db.get(metadata_dictionary::KEY).unwrap() else {
            continue;
        };
        let original: &[u8] = &buf;
        let dict = MetadataDictionary::decode(original)
            .unwrap_or_else(|e| panic!("{name}: failed to decode dictionary: {e}"));
        let re_encoded = dict
            .encode()
            .unwrap_or_else(|e| panic!("{name}: failed to encode dictionary: {e}"));
        assert_eq!(
            re_encoded, original,
            "{name}: re-encoded dictionary is not byte-identical to the original"
        );
        checked += 1;
    }
    assert_eq!(checked, 4, "expected exactly four dictionaries to check");
}

/// For every world that carries a dictionary, every chunk's `0x3f`
/// `MetadataDictionaryKey` value resolves to one of the dictionary's entry keys.
/// This is the direct evidence that the 8-byte value under a chunk's
/// `0x3f` record is exactly a [`metadata_dictionary::DictionaryKey`], not
/// merely a same-length coincidence: across all four worlds, every single
/// `0x3f` record found resolves, and none are left dangling.
#[test]
fn x3f_link_resolves_for_every_chunk_in_every_dictionary_world() {
    let expected_chunks = HashMap::from([
        ("v1_18_30", 230u64),
        ("v1_19_30", 260u64),
        ("v1_20_81", 246u64),
        ("1_26_seed", 308u64),
    ]);

    let mut checked_worlds = 0;
    for (name, _tmp, db) in all_worlds() {
        let Some(buf) = db.get(metadata_dictionary::KEY).unwrap() else {
            continue;
        };
        let dict = MetadataDictionary::decode(&buf).unwrap();

        let mut resolved = 0u64;
        let mut unresolved = Vec::new();
        let mut keys = db.keys().unwrap();
        for kv in &mut keys {
            let key_bytes = kv.key();
            let mut cursor = Cursor::new(key_bytes.as_ref());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };
            if key.data != KeyVariant::MetadataDictionaryKey {
                continue;
            }

            let value = kv.value();
            let link: Option<metadata_dictionary::DictionaryKey> =
                <[u8; 8]>::try_from(value.as_ref()).ok();
            match link {
                Some(link) if dict.get(&link).is_some() => resolved += 1,
                _ => unresolved.push(key.chunk),
            }
        }

        assert!(
            unresolved.is_empty(),
            "{name}: {} chunk(s) had a 0x3f value that did not resolve: {:?}",
            unresolved.len(),
            unresolved
        );
        assert_eq!(
            Some(resolved),
            expected_chunks.get(name.as_str()).copied(),
            "{name}: unexpected resolved-chunk count"
        );
        checked_worlds += 1;
    }
    assert_eq!(checked_worlds, 4, "expected exactly four worlds to check");
}

/// Sanity check on the typed accessors against real data: the 1.26 seed
/// world's dictionary carries both `Overworld` (extended, -64..320) and
/// `TheEnd` (0..256) entries, confirming `dimension_name` and the height
/// range accessors read real fields correctly rather than only synthetic
/// ones (see `src/metadata_dictionary.rs`'s unit tests for the synthetic
/// coverage).
#[test]
fn typed_accessors_read_real_fields_from_the_1_26_seed_world() {
    let (_tmp, db) = open_1_26();
    let buf = db.get(metadata_dictionary::KEY).unwrap().unwrap();
    let dict = MetadataDictionary::decode(&buf).unwrap();

    let dimension_names: std::collections::HashSet<_> = dict
        .iter()
        .filter_map(|(_, entry)| entry.dimension_name())
        .collect();
    assert!(dimension_names.contains("Overworld"));
    assert!(dimension_names.contains("TheEnd"));

    let mut saw_extended_overworld = false;
    let mut saw_end_range = false;
    for (_, entry) in dict.iter() {
        match entry.dimension_name() {
            Some("Overworld") => {
                if let Some(range) = entry.original_dimension_height_range() {
                    assert_eq!(range.min, -64);
                    assert_eq!(range.max, 320);
                    saw_extended_overworld = true;
                }
            }
            Some("TheEnd") => {
                if let Some(range) = entry.original_dimension_height_range() {
                    assert_eq!(range.min, 0);
                    assert_eq!(range.max, 256);
                    saw_end_range = true;
                }
            }
            _ => {}
        }
    }
    assert!(saw_extended_overworld, "no Overworld height range found");
    assert!(saw_end_range, "no TheEnd height range found");

    // Some real entries lack `LastSavedDimensionHeightRange`/
    // `LastSavedBaseGameVersion` entirely -- a chunk saved only once has
    // nothing to distinguish "last saved" from "original" -- so the
    // accessor must return `None` rather than erroring or panicking.
    assert!(
        dict.iter()
            .any(|(_, e)| e.last_saved_dimension_height_range().is_none()),
        "expected at least one real entry with no LastSavedDimensionHeightRange"
    );
}

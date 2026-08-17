//! Evidence for the Phase 4 string-keyed record task (see
//! `crates/level/TODO.md`): `map_<id>`, `portals`, `PosTrackDB-<id>` +
//! `PositionTrackDB-LastId`, and `BiomeIdsTable`, gathered against every
//! fixture world plus the 1.26 seed world (`tests/level.tar.gz`).
//!
//! `DimensionNameIdTable` has no test here: it is left unimplemented (see
//! `src/biome_ids_table.rs`'s module docs) because it is single-sourced --
//! and, consistent with that, is also absent from the whole corpus below.

mod support;

use bedrock_level::biome_ids_table;
use bedrock_level::db::Database;
use bedrock_level::map_data;
use bedrock_level::portals;
use bedrock_level::position_track;

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Every `(world, db)` this crate's fixture corpus can offer: the thirteen
/// imported fixtures plus the separately-shipped 1.26 seed world.
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

/// `map_<id>` is present in exactly three worlds -- established by a
/// database-wide prefix scan ([`map_data::ids`]) of the whole corpus.
/// `map_data::decode` then `encode` reproduces every one of the 65 real
/// records found byte-identically, covering both field orderings the
/// corpus shows (see `src/map_data.rs`'s module docs).
#[test]
fn decode_then_encode_reproduces_every_real_map_record() {
    let expected_counts =
        std::collections::HashMap::from([("v1_12", 15), ("v1_18_30", 25), ("v1_19_30", 25)]);

    let mut found = std::collections::HashMap::new();
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let ids = map_data::ids(&db).unwrap();
        if ids.is_empty() {
            continue;
        }
        found.insert(name.clone(), ids.len() as u32);

        for id in ids {
            let bytes = db.get(map_data::key(id)).unwrap().unwrap();
            let record = map_data::decode(&bytes)
                .unwrap_or_else(|e| panic!("{name}: map_{id}: failed to decode: {e}"));
            let re_encoded = map_data::encode(&record)
                .unwrap_or_else(|e| panic!("{name}: map_{id}: failed to encode: {e}"));
            assert_eq!(
                &re_encoded, &*bytes,
                "{name}: map_{id}: re-encoded record is not byte-identical"
            );
            assert_eq!(
                record.map_id(),
                Some(id),
                "{name}: map_{id}: mapId field does not match its own key"
            );
            total += 1;
        }
    }

    assert_eq!(found.len(), expected_counts.len());
    for (name, count) in &expected_counts {
        assert_eq!(found.get(*name).copied(), Some(*count), "{name}");
    }
    assert_eq!(total, 65, "total map records across the whole corpus");
}

/// `portals` is present in exactly four worlds, one `PortalRecords` entry
/// each except the seed world's two. `portals::decode` then `encode`
/// reproduces every real record byte-identically.
#[test]
fn decode_then_encode_reproduces_every_real_portals_record() {
    let expected_counts = std::collections::HashMap::from([
        ("v1_18", 1),
        ("v1_18_30", 1),
        ("v1_19_30", 1),
        ("1_26_seed", 2),
    ]);

    let mut found = std::collections::HashMap::new();
    for (name, _tmp, db) in all_worlds() {
        let Some(bytes) = db.get(portals::KEY).unwrap() else {
            continue;
        };
        let decoded = portals::decode(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode portals: {e}"));
        assert_eq!(
            &portals::encode(&decoded).unwrap(),
            &*bytes,
            "{name}: re-encoded portals record is not byte-identical"
        );
        found.insert(name.clone(), decoded.records.len() as u32);
    }

    assert_eq!(found.len(), expected_counts.len());
    for (name, count) in &expected_counts {
        assert_eq!(found.get(*name).copied(), Some(*count), "{name}");
    }
}

/// `PosTrackDB-<id>` / `PositionTrackDB-LastId` are present in exactly one
/// world in the whole corpus (the 1.26 seed). `position_track::read_all`
/// (driven off the `LastId` counter, per the module docs) finds the one
/// real entry, and both records re-encode byte-identically.
#[test]
fn decode_then_encode_reproduces_the_one_real_position_track_world() {
    let mut worlds_with_last_id = 0;
    let mut worlds_with_entries = 0;

    for (name, _tmp, db) in all_worlds() {
        let Some(last_id_bytes) = db.get(position_track::LAST_ID_KEY).unwrap() else {
            continue;
        };
        worlds_with_last_id += 1;

        let last_id = position_track::decode_last_id(&last_id_bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode LastId: {e}"));
        assert_eq!(
            &position_track::encode_last_id(&last_id).unwrap(),
            &*last_id_bytes,
            "{name}: re-encoded LastId is not byte-identical"
        );

        let entries = position_track::read_all(&db).unwrap();
        assert_eq!(entries.len(), 1, "{name}: expected exactly one entry");
        worlds_with_entries += entries.len();

        for (id, entry) in &entries {
            let entry = entry
                .as_ref()
                .unwrap_or_else(|e| panic!("{name}: PosTrackDB-0x{id:08x}: failed to decode: {e}"));
            let key_bytes = position_track::key(*id);
            let original = db.get(&key_bytes).unwrap().unwrap();
            assert_eq!(
                &position_track::encode_entry(entry).unwrap(),
                &*original,
                "{name}: PosTrackDB-0x{id:08x}: re-encoded entry is not byte-identical"
            );
            assert_eq!(entry.key(), Some(key_bytes));
        }
    }

    assert_eq!(
        worlds_with_last_id, 1,
        "expected exactly one world with a LastId record"
    );
    assert_eq!(
        worlds_with_entries, 1,
        "expected exactly one real entry across the corpus"
    );
}

/// `BiomeIdsTable` is absent from every world in the corpus -- confirmed
/// here as a standing assertion so growth of the corpus that starts
/// carrying the key is noticed. It is still implemented (see
/// `src/biome_ids_table.rs`'s module docs for why an unattested key is
/// modeled here) and self-round-trips through synthetic data in its own
/// unit tests.
#[test]
fn biome_ids_table_is_absent_from_the_whole_corpus() {
    let mut found = 0;
    for (_name, _tmp, db) in all_worlds() {
        if db.get(biome_ids_table::KEY).unwrap().is_some() {
            found += 1;
        }
    }
    assert_eq!(found, 0, "expected BiomeIdsTable in zero real worlds");
}

/// `DimensionNameIdTable` is absent from every world in the corpus too --
/// consistent with leaving it unimplemented as single-sourced (see
/// `src/biome_ids_table.rs`'s module docs).
#[test]
fn dimension_name_id_table_is_absent_from_the_whole_corpus() {
    let mut found = 0;
    for (_name, _tmp, db) in all_worlds() {
        if db
            .get(b"DimensionNameIdTable".as_slice())
            .unwrap()
            .is_some()
        {
            found += 1;
        }
    }
    assert_eq!(
        found, 0,
        "expected DimensionNameIdTable in zero real worlds"
    );
}

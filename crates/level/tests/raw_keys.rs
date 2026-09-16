//! Evidence for the Phase 6 "expose undecoded string keys as raw NBT" task
//! (see `crates/level/TODO.md` and `src/raw_keys.rs`'s module docs): a full
//! key scan of every fixture world plus the 1.26 seed world
//! (`tests/level.tar.gz`), and byte-identical decode/encode round trips for
//! every real record the scan found.

mod support;

use bedrock_level::db::Database;
use bedrock_level::raw_keys;
use bedrock_shared::world::dimension::Dimension;
use std::collections::HashMap;

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Every `(world, db)` this crate's fixture corpus can offer: the thirteen
/// imported fixtures plus the separately-shipped 1.26 seed world. Mirrors
/// `phase4_string_keys.rs`'s helper of the same name.
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

/// The five bare-string keys present in every one of the 14 corpus worlds,
/// per `src/raw_keys.rs`'s module-doc scan table.
const ALWAYS_PRESENT_KEYS: [&str; 5] = [
    raw_keys::SCOREBOARD,
    raw_keys::BIOME_DATA,
    raw_keys::MOB_EVENTS,
    raw_keys::SCHEDULER_WT,
    raw_keys::AUTONOMOUS_ENTITIES,
];

/// `scoreboard`/`BiomeData`/`mobevents`/`schedulerWT`/`AutonomousEntities`
/// are present in all 14 worlds -- confirmed by a full-database key scan,
/// not just a `get` at the expected name (a wrong constant would silently
/// read "absent" rather than fail loudly). Every real record found decodes
/// and re-encodes byte-identically.
#[test]
fn decode_then_encode_reproduces_every_always_present_key_in_every_world() {
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        for key in ALWAYS_PRESENT_KEYS {
            let bytes = db
                .get(key.as_bytes())
                .unwrap()
                .unwrap_or_else(|| panic!("{name}: expected {key} to be present"));
            let record = raw_keys::decode(&bytes)
                .unwrap_or_else(|e| panic!("{name}: {key}: failed to decode: {e}"));
            let re_encoded = raw_keys::encode(&record)
                .unwrap_or_else(|e| panic!("{name}: {key}: failed to encode: {e}"));
            assert_eq!(
                &re_encoded, &*bytes,
                "{name}: {key}: re-encoded record is not byte-identical"
            );
            total += 1;
        }
    }
    assert_eq!(total, 14 * ALWAYS_PRESENT_KEYS.len() as u32);
}

/// `read`/`write` round-trip the same way as the lower-level
/// `decode`/`encode` pair, exercised against one real record per
/// always-present key in the 1.26 seed world.
#[test]
fn read_then_write_round_trips_real_records() {
    let (_tmp, db) = open_1_26();
    for key in ALWAYS_PRESENT_KEYS {
        let original = db.get(key.as_bytes()).unwrap().unwrap();
        let record = raw_keys::read(&db, key.as_bytes()).unwrap().unwrap();
        raw_keys::write(&db, key.as_bytes(), &record).unwrap();
        let after = db.get(key.as_bytes()).unwrap().unwrap();
        assert_eq!(
            &*after, &*original,
            "{key}: write did not reproduce the original bytes"
        );
    }
}

/// The three dimension keys (`Overworld`/`Nether`/`TheEnd`) -- present in
/// 14/3/5 of the 14 worlds respectively, per the scan table in
/// `src/raw_keys.rs`. Confirms [`raw_keys::dimension_key`] names the right
/// on-disk key and that every real record found round-trips
/// byte-identically.
#[test]
fn decode_then_encode_reproduces_every_real_dimension_key() {
    let expected = HashMap::from([
        (Dimension::Overworld, 14u32),
        (Dimension::Nether, 3u32),
        (Dimension::End, 5u32),
    ]);
    let mut found: HashMap<Dimension, u32> = HashMap::new();

    for (name, _tmp, db) in all_worlds() {
        for dimension in [Dimension::Overworld, Dimension::Nether, Dimension::End] {
            let key = raw_keys::dimension_key(dimension).unwrap();
            let Some(bytes) = db.get(&key).unwrap() else {
                continue;
            };
            *found.entry(dimension).or_insert(0) += 1;
            let record = raw_keys::decode(&bytes)
                .unwrap_or_else(|e| panic!("{name}: {dimension:?} dimension key: {e}"));
            assert_eq!(
                raw_keys::encode(&record).unwrap(),
                &*bytes,
                "{name}: {dimension:?} dimension key: re-encoded record is not byte-identical"
            );
        }
    }

    for (dimension, count) in &expected {
        assert_eq!(
            found.get(dimension).copied().unwrap_or(0),
            *count,
            "{dimension:?}"
        );
    }
}

/// `game_flatworldlayers`, `VILLAGE_*`/`mVillages`, `structuretemplate_*`,
/// and `tickingarea_*` are absent from the whole corpus -- a standing
/// assertion so a corpus that starts carrying one of them is noticed
/// (matching `phase4_string_keys.rs`'s convention for `BiomeIdsTable`/
/// `DimensionNameIdTable`).
#[test]
fn unattested_keys_are_absent_from_the_whole_corpus() {
    let mut flatworldlayers = 0u32;
    let mut village = 0u32;
    let mut mvillages = 0u32;
    let mut structuretemplate = 0u32;
    let mut tickingarea = 0u32;

    for (_name, _tmp, db) in all_worlds() {
        if db
            .get(raw_keys::GAME_FLATWORLDLAYERS.as_bytes())
            .unwrap()
            .is_some()
        {
            flatworldlayers += 1;
        }
        if db.get(raw_keys::VILLAGES.as_bytes()).unwrap().is_some() {
            mvillages += 1;
        }
        village += raw_keys::village_keys(&db).unwrap().len() as u32;
        structuretemplate += raw_keys::structure_template_keys(&db).unwrap().len() as u32;
        tickingarea += raw_keys::ticking_area_keys(&db).unwrap().len() as u32;
    }

    assert_eq!(flatworldlayers, 0, "game_flatworldlayers");
    assert_eq!(village, 0, "VILLAGE_*");
    assert_eq!(mvillages, 0, "mVillages");
    assert_eq!(structuretemplate, 0, "structuretemplate_*");
    assert_eq!(tickingarea, 0, "tickingarea_*");
}

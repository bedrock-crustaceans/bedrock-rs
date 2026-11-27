//! Corpus evidence for the Phase 8 stretch task "structured decode where
//! the layout is stable enough to model" (`TODO.md`, `src/flat_records.rs`):
//! every typed field decodes and re-encodes byte-identically against every
//! real record the corpus scan found, the exact per-key presence counts are
//! pinned, and a mutated record re-encodes with the change while still
//! decoding back.

pub mod support;

use bedrock_level::db::Database;
use bedrock_level::flat_records;
use bedrock_level::raw_keys;
use bedrock_shared::world::dimension::Dimension;

fn open_1_26() -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("tmp");
    let tar_gz = std::fs::File::open("tests/level.tar.gz").unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar).unpack(tmp.path()).unwrap();
    let db_path = tmp.path().join("debug/db");
    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Opens the checked-in fixture named `name`, the same lookup
/// `checksums.rs` and several other test files use.
fn open_fixture(name: &str) -> (tempfile::TempDir, Database) {
    support::fixtures()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("{name} fixture is part of the checked-in fixture set"))
        .open()
}

/// Every `(world, db)` this crate's fixture corpus can offer: the thirteen
/// imported fixtures plus the separately-shipped 1.26 seed world. Mirrors
/// `raw_keys.rs`'s helper of the same name.
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

const WORLD_COUNT: u32 = 14;

// ---------------------------------------------------------------------
// mobevents -- 14/14
// ---------------------------------------------------------------------

#[test]
fn mob_events_decode_then_encode_reproduces_every_real_record() {
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db
            .get(raw_keys::MOB_EVENTS.as_bytes())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: expected mobevents to be present"));
        let record = flat_records::decode_mob_events(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
        assert_eq!(
            flat_records::encode_mob_events(&record).unwrap(),
            &*bytes,
            "{name}: re-encoded mobevents is not byte-identical"
        );
        // Every observed value in the corpus is `1` -- see the module docs.
        assert_eq!(record.events_enabled(), Some(true), "{name}");
        assert_eq!(record.pillager_patrols_event(), Some(true), "{name}");
        assert_eq!(record.wandering_trader_event(), Some(true), "{name}");
        total += 1;
    }
    assert_eq!(total, WORLD_COUNT);
}

#[test]
fn mob_events_ender_dragon_event_absent_only_in_v1_12() {
    for (name, _tmp, db) in all_worlds() {
        let bytes = db.get(raw_keys::MOB_EVENTS.as_bytes()).unwrap().unwrap();
        let record = flat_records::decode_mob_events(&bytes).unwrap();
        if name == "v1_12" {
            assert_eq!(record.ender_dragon_event(), None, "{name}");
        } else {
            assert_eq!(record.ender_dragon_event(), Some(true), "{name}");
        }
    }
}

#[test]
fn mob_events_mutation_re_encodes_with_the_change_and_decodes_back() {
    let (_tmp, db) = open_1_26();
    let bytes = db.get(raw_keys::MOB_EVENTS.as_bytes()).unwrap().unwrap();
    let mut record = flat_records::decode_mob_events(&bytes).unwrap();
    let nbtx::Value::Compound(compound) = record.value_mut() else {
        panic!("expected a compound");
    };
    compound.insert("events_enabled".into(), nbtx::Value::Byte(0));

    let re_encoded = flat_records::encode_mob_events(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_mob_events(&re_encoded).unwrap();
    assert_eq!(re_decoded.events_enabled(), Some(false));
}

// ---------------------------------------------------------------------
// schedulerWT -- 14/14
// ---------------------------------------------------------------------

#[test]
fn scheduler_wt_decode_then_encode_reproduces_every_real_record() {
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db
            .get(raw_keys::SCHEDULER_WT.as_bytes())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: expected schedulerWT to be present"));
        let record = flat_records::decode_scheduler_wt(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
        assert_eq!(
            flat_records::encode_scheduler_wt(&record).unwrap(),
            &*bytes,
            "{name}: re-encoded schedulerWT is not byte-identical"
        );
        assert!(record.days_since_last_wt_spawn().is_some(), "{name}");
        assert_eq!(record.is_spawning_wt(), Some(false), "{name}");
        assert!(record.next_wt_spawn_check_tick().is_some(), "{name}");
        total += 1;
    }
    assert_eq!(total, WORLD_COUNT);
}

#[test]
fn scheduler_wt_existing_wt_present_only_in_the_three_named_worlds() {
    const WITH_EXISTING_WT: [&str; 3] = ["v1_18", "v1_18_30", "v1_19_30"];
    let mut found = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db.get(raw_keys::SCHEDULER_WT.as_bytes()).unwrap().unwrap();
        let record = flat_records::decode_scheduler_wt(&bytes).unwrap();
        if WITH_EXISTING_WT.contains(&name.as_str()) {
            assert!(record.existing_wt().is_some(), "{name}");
            found += 1;
        } else {
            assert_eq!(record.existing_wt(), None, "{name}");
        }
    }
    assert_eq!(found, 3);
}

#[test]
fn scheduler_wt_mutation_re_encodes_with_the_change_and_decodes_back() {
    let (_tmp, db) = open_1_26();
    let bytes = db.get(raw_keys::SCHEDULER_WT.as_bytes()).unwrap().unwrap();
    let mut record = flat_records::decode_scheduler_wt(&bytes).unwrap();
    let nbtx::Value::Compound(compound) = record.value_mut() else {
        panic!("expected a compound");
    };
    compound.insert("daysSinceLastWTSpawn".into(), nbtx::Value::Int(99));

    let re_encoded = flat_records::encode_scheduler_wt(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_scheduler_wt(&re_encoded).unwrap();
    assert_eq!(re_decoded.days_since_last_wt_spawn(), Some(99));
}

// ---------------------------------------------------------------------
// scoreboard -- 14/14
// ---------------------------------------------------------------------

#[test]
fn scoreboard_decode_then_encode_reproduces_every_real_record() {
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db
            .get(raw_keys::SCOREBOARD.as_bytes())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: expected scoreboard to be present"));
        let record = flat_records::decode_scoreboard(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
        assert_eq!(
            flat_records::encode_scoreboard(&record).unwrap(),
            &*bytes,
            "{name}: re-encoded scoreboard is not byte-identical"
        );
        assert!(record.last_unique_id().is_some(), "{name}");
        assert_eq!(record.criteria(), Some(&nbtx::ValueList::End), "{name}");
        assert_eq!(
            record.display_objectives(),
            Some(&nbtx::ValueList::End),
            "{name}"
        );
        assert_eq!(record.entries(), Some(&nbtx::ValueList::End), "{name}");
        assert_eq!(record.objectives(), Some(&nbtx::ValueList::End), "{name}");
        total += 1;
    }
    assert_eq!(total, WORLD_COUNT);
}

#[test]
fn scoreboard_mutation_re_encodes_with_the_change_and_decodes_back() {
    let (_tmp, db) = open_1_26();
    let bytes = db.get(raw_keys::SCOREBOARD.as_bytes()).unwrap().unwrap();
    let mut record = flat_records::decode_scoreboard(&bytes).unwrap();
    let nbtx::Value::Compound(compound) = record.value_mut() else {
        panic!("expected a compound");
    };
    compound.insert("LastUniqueID".into(), nbtx::Value::Long(999));

    let re_encoded = flat_records::encode_scoreboard(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_scoreboard(&re_encoded).unwrap();
    assert_eq!(re_decoded.last_unique_id(), Some(999));
}

// ---------------------------------------------------------------------
// BiomeData -- 14/14
// ---------------------------------------------------------------------

#[test]
fn biome_data_decode_then_encode_reproduces_every_real_record() {
    let mut total = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db
            .get(raw_keys::BIOME_DATA.as_bytes())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: expected BiomeData to be present"));
        let record = flat_records::decode_biome_data(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
        assert_eq!(
            flat_records::encode_biome_data(&record).unwrap(),
            &*bytes,
            "{name}: re-encoded BiomeData is not byte-identical"
        );
        assert!(record.entries().is_some_and(|e| !e.is_empty()), "{name}");
        total += 1;
    }
    assert_eq!(total, WORLD_COUNT);
}

/// The 1.26 seed world keeps `Short` ids on every entry; every other corpus
/// world keeps `Byte` ids on every entry -- no world mixes the two tags
/// within its own record.
#[test]
fn biome_data_ids_are_short_only_in_the_1_26_seed_world() {
    let mut byte_worlds = 0u32;
    let mut short_worlds = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap();
        let record = flat_records::decode_biome_data(&bytes).unwrap();
        let entries = record.entries().unwrap();
        let all_byte = entries
            .iter()
            .all(|e| matches!(e.id, flat_records::BiomeId::Byte(_)));
        let all_short = entries
            .iter()
            .all(|e| matches!(e.id, flat_records::BiomeId::Short(_)));
        if name == "1_26_seed" {
            assert!(all_short, "{name}: expected every id to be Short");
            short_worlds += 1;
        } else {
            assert!(all_byte, "{name}: expected every id to be Byte");
            byte_worlds += 1;
        }
    }
    assert_eq!(byte_worlds, 13);
    assert_eq!(short_worlds, 1);
}

/// `foliageSnow` is present on every entry in the `v1_18` world and absent
/// everywhere else -- see the module docs' note that no trigger for this is
/// discernible from the corpus.
#[test]
fn biome_data_foliage_snow_present_only_in_v1_18() {
    let mut with_foliage_snow = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap();
        let record = flat_records::decode_biome_data(&bytes).unwrap();
        let entries = record.entries().unwrap();
        let any_foliage_snow = entries.iter().any(|e| e.foliage_snow.is_some());
        if name == "v1_18" {
            assert!(
                entries.iter().all(|e| e.foliage_snow.is_some()),
                "{name}: expected every entry to carry foliageSnow"
            );
            with_foliage_snow += 1;
        } else {
            assert!(!any_foliage_snow, "{name}");
        }
    }
    assert_eq!(with_foliage_snow, 1);
}

#[test]
fn biome_data_mutation_re_encodes_with_the_change_and_decodes_back() {
    let (_tmp, db) = open_1_26();
    let bytes = db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap();
    let mut record = flat_records::decode_biome_data(&bytes).unwrap();
    let before = record.entries().unwrap();
    let nbtx::Value::Compound(compound) = record.value_mut() else {
        panic!("expected a compound");
    };
    let nbtx::Value::List(nbtx::ValueList::Compound(entries)) =
        compound.get_mut("list".as_bytes()).unwrap()
    else {
        panic!("expected a list of compounds");
    };
    let first = &mut entries[0];
    first.insert("snowAccumulation".into(), nbtx::Value::Float(9.5));

    let re_encoded = flat_records::encode_biome_data(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_biome_data(&re_encoded).unwrap();
    let after = re_decoded.entries().unwrap();
    assert_eq!(after[0].snow_accumulation, 9.5);
    assert_ne!(after[0].snow_accumulation, before[0].snow_accumulation);
}

/// The mutation test above only exercises the `Short`-id `1_26_seed` world.
/// `v1_18` is a `Byte`-id world (see
/// `biome_data_ids_are_short_only_in_the_1_26_seed_world`); mutating an
/// unrelated field there and re-encoding must not touch the `id` tag --
/// [`flat_records::BiomeId`] re-emits whatever tag it read, and this pins
/// that a mutation elsewhere in the record does not widen it.
#[test]
fn biome_data_mutation_on_a_byte_world_keeps_the_byte_id_tag() {
    let (_tmp, db) = open_fixture("v1_18");
    let bytes = db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap();
    let mut record = flat_records::decode_biome_data(&bytes).unwrap();
    let before = record.entries().unwrap();
    assert!(
        before
            .iter()
            .all(|e| matches!(e.id, flat_records::BiomeId::Byte(_))),
        "v1_18: expected every id to be Byte before mutation"
    );

    let nbtx::Value::Compound(compound) = record.value_mut() else {
        panic!("expected a compound");
    };
    let nbtx::Value::List(nbtx::ValueList::Compound(entries)) =
        compound.get_mut("list".as_bytes()).unwrap()
    else {
        panic!("expected a list of compounds");
    };
    let first = &mut entries[0];
    first.insert("snowAccumulation".into(), nbtx::Value::Float(2.0));

    let re_encoded = flat_records::encode_biome_data(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_biome_data(&re_encoded).unwrap();
    let after = re_decoded.entries().unwrap();
    assert_eq!(after[0].snow_accumulation, 2.0);
    assert!(
        matches!(after[0].id, flat_records::BiomeId::Byte(_)),
        "v1_18: expected id to stay Byte after an unrelated mutation"
    );
    assert_eq!(after[0].id, before[0].id);
}

/// Corroborates the "1.26 uses `Short` ids" test above against the
/// `BiomeId::value` convenience directly, on one real entry.
#[test]
fn biome_id_value_widens_both_tags_to_i32() {
    assert_eq!(flat_records::BiomeId::Byte(10).value(), 10);
    assert_eq!(flat_records::BiomeId::Short(10).value(), 10);
    assert_eq!(flat_records::BiomeId::Byte(-116).value(), -116);
}

// ---------------------------------------------------------------------
// AutonomousEntities -- 14/14
// ---------------------------------------------------------------------

#[test]
fn autonomous_entities_decode_then_encode_reproduces_every_real_record() {
    let mut total = 0u32;
    let mut empty = 0u32;
    let mut non_empty = 0u32;
    for (name, _tmp, db) in all_worlds() {
        let bytes = db
            .get(raw_keys::AUTONOMOUS_ENTITIES.as_bytes())
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: expected AutonomousEntities to be present"));
        let record = flat_records::decode_autonomous_entities(&bytes)
            .unwrap_or_else(|e| panic!("{name}: failed to decode: {e}"));
        assert_eq!(
            flat_records::encode_autonomous_entities(&record).unwrap(),
            &*bytes,
            "{name}: re-encoded AutonomousEntities is not byte-identical"
        );
        let list = record.entity_list().unwrap();
        if list.is_empty() {
            empty += 1;
        } else {
            assert_eq!(name, "v1_18", "unexpected non-empty AutonomousEntityList");
            non_empty += 1;
        }
        total += 1;
    }
    assert_eq!(total, WORLD_COUNT);
    assert_eq!(empty, 13);
    assert_eq!(non_empty, 1);
}

// ---------------------------------------------------------------------
// Overworld / Nether / TheEnd
// ---------------------------------------------------------------------

#[test]
fn dimension_record_decode_then_encode_reproduces_every_real_record() {
    let mut overworld = 0u32;
    let mut nether = 0u32;
    let mut the_end = 0u32;

    for (name, _tmp, db) in all_worlds() {
        for (key, counter) in [
            (
                raw_keys::dimension_key_name(Dimension::Overworld).unwrap(),
                &mut overworld,
            ),
            (
                raw_keys::dimension_key_name(Dimension::Nether).unwrap(),
                &mut nether,
            ),
            (
                raw_keys::dimension_key_name(Dimension::End).unwrap(),
                &mut the_end,
            ),
        ] {
            let Some(bytes) = db.get(key.as_bytes()).unwrap() else {
                continue;
            };
            *counter += 1;
            let record = flat_records::decode_dimension_record(&bytes)
                .unwrap_or_else(|e| panic!("{name}: {key}: failed to decode: {e}"));
            assert_eq!(
                flat_records::encode_dimension_record(&record).unwrap(),
                &*bytes,
                "{name}: {key}: re-encoded record is not byte-identical"
            );
            assert_eq!(
                record.limbo_entities(),
                Some(&nbtx::ValueList::End),
                "{name}: {key}: LimboEntities"
            );
        }
    }

    assert_eq!(overworld, 14);
    assert_eq!(nether, 3);
    assert_eq!(the_end, 5);
}

#[test]
fn dimension_record_read_matches_dimension_key_presence() {
    let mut overworld = 0u32;
    let mut nether = 0u32;
    let mut the_end = 0u32;

    for (name, _tmp, db) in all_worlds() {
        for (dimension, counter) in [
            (Dimension::Overworld, &mut overworld),
            (Dimension::Nether, &mut nether),
            (Dimension::End, &mut the_end),
        ] {
            let record = flat_records::read_dimension_record(&db, dimension)
                .unwrap_or_else(|e| panic!("{name}: {dimension:?}: {e}"));
            if record.is_some() {
                *counter += 1;
            }
        }
    }

    assert_eq!(overworld, 14);
    assert_eq!(nether, 3);
    assert_eq!(the_end, 5);
}

/// `TheEnd`'s `DragonFight` is present on every one of the 5 real records;
/// `Overworld`/`Nether` never carry one.
#[test]
fn dimension_record_dragon_fight_present_only_on_the_end() {
    let mut the_end_with_fight = 0u32;
    for (name, _tmp, db) in all_worlds() {
        for dimension in [Dimension::Overworld, Dimension::Nether, Dimension::End] {
            let key = raw_keys::dimension_key_name(dimension).unwrap();
            let Some(bytes) = db.get(key.as_bytes()).unwrap() else {
                continue;
            };
            let record = flat_records::decode_dimension_record(&bytes).unwrap();
            match dimension {
                Dimension::End => {
                    assert!(record.dragon_fight().is_some(), "{name}: {key}");
                    the_end_with_fight += 1;
                }
                _ => assert!(record.dragon_fight().is_none(), "{name}: {key}"),
            }
        }
    }
    assert_eq!(the_end_with_fight, 5);
}

/// `ExitPortalLocation` is present in exactly the 2 of 5 `TheEnd` records
/// whose `DragonSpawned` is also true in this corpus -- an observed
/// correlation (n=5), not a proven causal rule; see the module docs.
#[test]
fn dimension_record_exit_portal_location_present_iff_dragon_spawned_in_this_corpus() {
    let mut spawned_with_exit = 0u32;
    let mut not_spawned_without_exit = 0u32;
    let mut total = 0u32;

    let the_end_key = raw_keys::dimension_key(Dimension::End).unwrap();
    for (name, _tmp, db) in all_worlds() {
        let Some(bytes) = db.get(&the_end_key).unwrap() else {
            continue;
        };
        let record = flat_records::decode_dimension_record(&bytes).unwrap();
        let fight = record
            .dragon_fight()
            .unwrap_or_else(|| panic!("{name}: expected a DragonFight"));
        total += 1;
        if fight.spawned {
            assert!(
                fight.exit_portal_location.is_some(),
                "{name}: spawned but no ExitPortalLocation"
            );
            spawned_with_exit += 1;
        } else {
            assert!(
                fight.exit_portal_location.is_none(),
                "{name}: not spawned but has ExitPortalLocation"
            );
            not_spawned_without_exit += 1;
        }
    }

    assert_eq!(total, 5);
    assert_eq!(spawned_with_exit, 2);
    assert_eq!(not_spawned_without_exit, 3);
}

#[test]
fn dimension_record_mutation_re_encodes_with_the_change_and_decodes_back() {
    let (_tmp, db) = open_1_26();
    let bytes = db
        .get(raw_keys::dimension_key(Dimension::End).unwrap())
        .unwrap()
        .unwrap();
    let mut record = flat_records::decode_dimension_record(&bytes).unwrap();
    let was_killed = record.dragon_fight().unwrap().killed;

    let nbtx::Value::Compound(root) = record.value_mut() else {
        panic!("expected a compound");
    };
    let nbtx::Value::Compound(data) = root.get_mut("data".as_bytes()).unwrap() else {
        panic!("expected a compound");
    };
    let nbtx::Value::Compound(dragon_fight) = data.get_mut("DragonFight".as_bytes()).unwrap()
    else {
        panic!("expected a compound");
    };
    dragon_fight.insert(
        "DragonKilled".into(),
        nbtx::Value::Byte(i8::from(!was_killed)),
    );

    let re_encoded = flat_records::encode_dimension_record(&record).unwrap();
    assert_ne!(re_encoded, *bytes);
    let re_decoded = flat_records::decode_dimension_record(&re_encoded).unwrap();
    assert_eq!(re_decoded.dragon_fight().unwrap().killed, !was_killed);
}

// ---------------------------------------------------------------------
// read/write round trips through a real `Database`
// ---------------------------------------------------------------------

#[test]
fn read_then_write_round_trips_real_records() {
    let (_tmp, db) = open_1_26();

    let original = db.get(raw_keys::MOB_EVENTS.as_bytes()).unwrap().unwrap();
    let record = flat_records::read_mob_events(&db).unwrap().unwrap();
    flat_records::write_mob_events(&db, &record).unwrap();
    assert_eq!(
        &*db.get(raw_keys::MOB_EVENTS.as_bytes()).unwrap().unwrap(),
        &*original
    );

    let original = db.get(raw_keys::SCHEDULER_WT.as_bytes()).unwrap().unwrap();
    let record = flat_records::read_scheduler_wt(&db).unwrap().unwrap();
    flat_records::write_scheduler_wt(&db, &record).unwrap();
    assert_eq!(
        &*db.get(raw_keys::SCHEDULER_WT.as_bytes()).unwrap().unwrap(),
        &*original
    );

    let original = db.get(raw_keys::SCOREBOARD.as_bytes()).unwrap().unwrap();
    let record = flat_records::read_scoreboard(&db).unwrap().unwrap();
    flat_records::write_scoreboard(&db, &record).unwrap();
    assert_eq!(
        &*db.get(raw_keys::SCOREBOARD.as_bytes()).unwrap().unwrap(),
        &*original
    );

    let original = db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap();
    let record = flat_records::read_biome_data(&db).unwrap().unwrap();
    flat_records::write_biome_data(&db, &record).unwrap();
    assert_eq!(
        &*db.get(raw_keys::BIOME_DATA.as_bytes()).unwrap().unwrap(),
        &*original
    );

    let original = db
        .get(raw_keys::AUTONOMOUS_ENTITIES.as_bytes())
        .unwrap()
        .unwrap();
    let record = flat_records::read_autonomous_entities(&db)
        .unwrap()
        .unwrap();
    flat_records::write_autonomous_entities(&db, &record).unwrap();
    assert_eq!(
        &*db.get(raw_keys::AUTONOMOUS_ENTITIES.as_bytes())
            .unwrap()
            .unwrap(),
        &*original
    );

    let the_end_key = raw_keys::dimension_key(Dimension::End).unwrap();
    let original = db.get(&the_end_key).unwrap().unwrap();
    let record = flat_records::read_dimension_record(&db, Dimension::End)
        .unwrap()
        .unwrap();
    flat_records::write_dimension_record(&db, Dimension::End, &record).unwrap();
    assert_eq!(&*db.get(&the_end_key).unwrap().unwrap(), &*original);
}

#[test]
fn write_dimension_record_fails_for_a_dimension_with_no_key_name() {
    let (_tmp, db) = open_1_26();
    let record = flat_records::DimensionRecord::new(nbtx::Value::Compound(nbtx::Compound::new()));
    assert!(flat_records::write_dimension_record(&db, Dimension::Other(1000), &record).is_err());
}

#[test]
fn read_dimension_record_is_none_for_a_dimension_with_no_key_name() {
    let (_tmp, db) = open_1_26();
    assert_eq!(
        flat_records::read_dimension_record(&db, Dimension::Other(1000)).unwrap(),
        None
    );
}

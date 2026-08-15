use std::fs::File;
use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::traits::CursorExt;
use bedrock_vanilla_data::block_entities::BlockEntity;

use flate2::read::GzDecoder;
use tar::Archive;

fn extract_test_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    // The world fixture is shared with the level crate rather than duplicated here.
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../level/tests/level.tar.gz");
    let tar_gz = File::open(fixture).expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

/// Opens the fixture database, returning the `TempDir` alongside it.
///
/// The `TempDir` unlinks its contents when dropped, and `Database` keeps no
/// reference back to it, so the caller has to hold onto it for as long as the
/// `Database` is in use — dropping it early silently truncates whatever the
/// database can still see to leftover file-descriptor state.
fn open_test_db() -> (tempfile::TempDir, Database) {
    let tmp = extract_test_dir();
    let db_path = tmp.path().join("debug/db");

    (tmp, Database::open(db_path.to_str().unwrap()).unwrap())
}

/// Every block-entity id known to fail against the real test-world fixture,
/// why, and exactly how many of its records currently decode successfully
/// despite that.
///
/// None of these are tag-width or bool-decode issues — nothing here is a
/// `lenient_width` candidate — each needs its own field-by-field fix, tracked
/// in `crates/level/TODO.md`.
///
/// Most of these ids are a gap only some of their records hit (a required
/// field a minority of records omit, say), so most of their own records
/// decode fine — the second element pins exactly how many. Pinning a full
/// count, not just "this id fails sometimes", matters because a tolerance can
/// regress from "some records fail" to "all records fail" without changing
/// which ids show up as failing at all — e.g. if `ShulkerBox::findable`'s
/// `lenient_width` were dropped, `ShulkerBox` would stay in this list (it
/// already fails 100% of its records for the unrelated `facing` bug) with no
/// other signal that a second, real tolerance silently vanished.
///
/// Kept as an explicit allowlist rather than an `#[ignore]` on the whole test
/// so the 4169 records (89.5%) that already decode correctly stay covered: a
/// decode failure for an id *not* on this list is a real regression, and any
/// drift in a listed id's exact ok-count — including to zero, or up to its
/// full total — is a stale entry to update or investigate.
const KNOWN_FAILING_IDS: &[(&str, usize)] = &[
    // `SculkSensor::vibration_listener` is required but absent on these
    // records; `CalibratedSculkSensor` and `SculkSensor` share the struct.
    ("CalibratedSculkSensor", 180),
    ("SculkSensor", 64),
    // `ShulkerBox::facing` is typed `f32`; real records write a face-index
    // `Byte`. A modeling bug, not a tag-width gap — `lenient_width` would
    // accept the tag but not resolve it to the right value.
    ("ShulkerBox", 0),
    // `SignText::filtered_text` is required but absent on these records.
    ("Sign", 413),
    // `DecoratedPot::item` is required but absent on these records.
    ("DecoratedPot", 45),
    // `JigsawBlock::placement_priority` is required but absent.
    ("JigsawBlock", 1),
    // `Dispenser::loot_table` is required but absent; `Dropper` and
    // `Dispenser` share the struct. `Chest::loot_table` already models the
    // same key as `Option<String>`, so this is likely just a missed `Option`.
    ("Dropper", 0),
    ("Dispenser", 44),
    // `Furnace::stored_xp` is required but absent; `BlastFurnace` shares the
    // struct.
    ("BlastFurnace", 0),
    // `StructureBlock::last_touched_player_id` is required but absent.
    ("StructureBlock", 1),
    // No struct registered for these ids at all yet.
    ("DaylightDetector", 0),
    ("EnderChest", 0),
    ("SculkShrieker", 0),
    ("Smoker", 0),
    ("SporeBlossom", 0),
];

/// The `id` string a raw block-entity record's `id` key holds, or `"?"` if
/// the record is not a compound or the key is missing/not a string — either
/// of which is itself an unexpected shape the assertion below will catch.
fn record_id(value: &nbtx::Value) -> &str {
    let nbtx::Value::Compound(entries) = value else {
        return "?";
    };
    match entries.get(bstr::BStr::new("id")) {
        Some(nbtx::Value::String(s)) => std::str::from_utf8(s).unwrap_or("?"),
        _ => "?",
    }
}

/// Decodes every block-entity record in the real test-world fixture and
/// checks two things against [`KNOWN_FAILING_IDS`]: that no id outside the
/// list produced a failure, and that every listed id's exact ok-count still
/// matches — not just that it failed *at all*.
///
/// The exact-count check is what makes this a real regression guard rather
/// than an easily-satisfied one: an id whose ok-count silently drops (partial
/// failure becoming total failure, most dangerously) or rises (a fix landed
/// but the entry was not updated) fails the assertion either way, even though
/// the *set* of failing ids never changed.
#[test]
fn block_entity_decode_failures_match_known_gaps() {
    let (_tmp, db) = open_test_db();
    let mut keys = db.keys().unwrap();

    let mut total = 0usize;
    let mut ok = 0usize;
    let mut ok_by_id: std::collections::HashMap<String, usize> = Default::default();
    let mut failing_ids: std::collections::BTreeSet<String> = Default::default();

    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut key_buf) else {
            continue;
        };

        if let KeyVariant::BlockEntity = key.data {
            let mut cursor = Cursor::new(kv.value());

            while cursor.has_remaining() {
                let raw: nbtx::Value = nbtx::from_le_bytes(&mut cursor)
                    .expect("a block-entity record's raw NBT structure should always parse");
                total += 1;

                let id = record_id(&raw).to_string();
                match BlockEntity::from_value(raw) {
                    Ok(_) => {
                        ok += 1;
                        *ok_by_id.entry(id).or_insert(0) += 1;
                    }
                    Err(_) => {
                        failing_ids.insert(id);
                    }
                }
            }
        }
    }

    println!("total records: {total}, ok: {ok}, known-failing: {}", total - ok);

    let known: std::collections::BTreeSet<&str> =
        KNOWN_FAILING_IDS.iter().map(|(id, _)| *id).collect();
    let regressions: Vec<_> = failing_ids
        .iter()
        .filter(|id| !known.contains(id.as_str()))
        .collect();
    assert!(
        regressions.is_empty(),
        "id(s) outside KNOWN_FAILING_IDS failed to decode - a real regression: {regressions:?}"
    );

    let mut drifted = Vec::new();
    for (id, expected_ok) in KNOWN_FAILING_IDS {
        let actual_ok = ok_by_id.get(*id).copied().unwrap_or(0);
        if actual_ok != *expected_ok {
            drifted.push(format!("{id}: expected {expected_ok} ok, got {actual_ok}"));
        }
    }
    assert!(
        drifted.is_empty(),
        "KNOWN_FAILING_IDS ok-count(s) drifted from the fixture - a fix landed and the entry \
         needs updating, or a tolerance regressed:\n{}",
        drifted.join("\n")
    );

    // A sanity floor on top of the per-id checks above: if the fixture or the
    // decode path changes in a way that stops touching real records
    // altogether, this catches it even though no single id looks wrong.
    assert!(total > 4000, "expected the full fixture, got {total} records");
}

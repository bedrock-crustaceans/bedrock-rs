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
/// and why. None of these are tag-width or bool-decode issues — nothing here
/// is a `lenient_width` candidate — each needs its own field-by-field fix,
/// tracked in `crates/level/TODO.md`.
///
/// Kept as an explicit allowlist rather than an `#[ignore]` on the whole test
/// so the 4169 records (89.5%) that already decode correctly stay covered:
/// a decode failure for an id *not* on this list is a real regression, and an
/// id on this list that stops failing is a stale entry to remove.
const KNOWN_FAILING_IDS: &[&str] = &[
    // `SculkSensor::vibration_listener` is required but absent on these
    // records; `CalibratedSculkSensor` and `SculkSensor` share the struct.
    "CalibratedSculkSensor",
    "SculkSensor",
    // `ShulkerBox::facing` is typed `f32`; real records write a face-index
    // `Byte`. A modeling bug, not a tag-width gap — `lenient_width` would
    // accept the tag but not resolve it to the right value.
    "ShulkerBox",
    // `SignText::filtered_text` is required but absent on these records.
    "Sign",
    // `DecoratedPot::item` is required but absent on these records.
    "DecoratedPot",
    // `JigsawBlock::placement_priority` is required but absent.
    "JigsawBlock",
    // `Dispenser::loot_table` is required but absent; `Dropper` and
    // `Dispenser` share the struct. `Chest::loot_table` already models the
    // same key as `Option<String>`, so this is likely just a missed `Option`.
    "Dropper",
    "Dispenser",
    // `Furnace::stored_xp` is required but absent; `BlastFurnace` shares the
    // struct.
    "BlastFurnace",
    // `StructureBlock::last_touched_player_id` is required but absent.
    "StructureBlock",
    // No struct registered for these ids at all yet.
    "DaylightDetector",
    "EnderChest",
    "SculkShrieker",
    "Smoker",
    "SporeBlossom",
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
/// checks the *set* of ids that produced at least one decode failure against
/// [`KNOWN_FAILING_IDS`].
///
/// This is a set comparison, not "every record with a listed id must fail":
/// several of these ids (`Sign`, `Dispenser`, `DecoratedPot`, `JigsawBlock`,
/// `StructureBlock`, `CalibratedSculkSensor`, `SculkSensor`, `BlastFurnace`)
/// are missing-field gaps that only some records happen to hit, so most of
/// their records decode fine — that's expected and fine. What is not
/// expected: a *new* id showing up in the failure set (a real regression), or
/// a listed id no longer producing *any* failure (a stale allowlist entry,
/// most likely because it just got fixed).
#[test]
fn block_entity_decode_failures_match_known_gaps() {
    let (_tmp, db) = open_test_db();
    let mut keys = db.keys().unwrap();

    let mut total = 0usize;
    let mut ok = 0usize;
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
                    Ok(_) => ok += 1,
                    Err(_) => {
                        failing_ids.insert(id);
                    }
                }
            }
        }
    }

    println!("total records: {total}, ok: {ok}, known-failing: {}", total - ok);

    let known: std::collections::BTreeSet<String> =
        KNOWN_FAILING_IDS.iter().map(|s| s.to_string()).collect();

    let regressions: Vec<_> = failing_ids.difference(&known).collect();
    let stale: Vec<_> = known.difference(&failing_ids).collect();

    assert!(
        regressions.is_empty(),
        "id(s) outside KNOWN_FAILING_IDS failed to decode - a real regression: {regressions:?}"
    );
    assert!(
        stale.is_empty(),
        "id(s) in KNOWN_FAILING_IDS produced no failures - stale allowlist entries, remove: \
         {stale:?}"
    );
    // A sanity floor on top of the per-id checks above: if the fixture or the
    // decode path changes in a way that stops touching real records
    // altogether, this catches it even though no single id looks wrong.
    assert!(total > 4000, "expected the full fixture, got {total} records");
}

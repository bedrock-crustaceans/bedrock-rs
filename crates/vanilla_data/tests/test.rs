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
/// Empty: every id the fixture carries now decodes, across all 4655 real
/// block-entity records. The gaps that used to live here — five missing
/// ids (`DaylightDetector`, `EnderChest`, `SculkShrieker`, `Smoker`,
/// `SporeBlossom`), seven required-but-sometimes-absent fields, and
/// `ShulkerBox::facing`'s wrong type — are fixed; see `crates/level/TODO.md`
/// for what each one was.
///
/// Kept as an explicit allowlist rather than an `#[ignore]` on the whole test
/// so every record stays covered: a decode failure for any id is a real
/// regression, and any drift in a listed id's exact ok-count — including a
/// tolerance silently regressing from "some records fail" to "all records
/// fail" without the failing *set* changing at all — is caught the same way.
const KNOWN_FAILING_IDS: &[(&str, usize)] = &[];

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

    println!(
        "total records: {total}, ok: {ok}, known-failing: {}",
        total - ok
    );

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
    assert!(
        total > 4000,
        "expected the full fixture, got {total} records"
    );
}

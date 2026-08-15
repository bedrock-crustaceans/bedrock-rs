//! Exercises the imported version-diverse test worlds through the fixture
//! harness in `support`, as opposed to `test.rs`'s single hardcoded world.

mod support;

use std::collections::BTreeSet;
use std::io::Cursor;

use bedrock_level::key::Key;

/// Every fixture opens through the pure-Rust backend, yields at least one
/// key, and its on-disk header storage version matches what the index
/// records for it. This is also the smoke test the index numbers were
/// generated against: `support::fixtures()` and the scan that built
/// `index.json` both go through the same `Database`/`Key` path.
#[test]
fn every_fixture_opens_and_matches_index() {
    for fixture in support::fixtures() {
        let (tmp, db) = fixture.open();

        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));
        let mut seen = 0usize;
        let mut chunk_versions_seen: BTreeSet<u8> = BTreeSet::new();
        for kv in &mut keys {
            seen += 1;
            let mut cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut cursor) else {
                continue;
            };
            if key.data == bedrock_level::key::KeyVariant::ChunkVersion
                && let Some(&v) = kv.value().as_ref().first()
            {
                chunk_versions_seen.insert(v);
            }
        }
        assert!(seen > 0, "{}: database opened but is empty", fixture.name);

        // A fixture whose only chunk-version record is the legacy `0x76`
        // fallback (which the index also accounts for) is not double-checked
        // here since this loop only reads `0x2c`; the index's
        // `chunk_versions` is still the authoritative set.
        for v in &chunk_versions_seen {
            assert!(
                fixture.chunk_versions.contains(v),
                "{}: chunk version {v} present on disk but missing from the index",
                fixture.name
            );
        }

        let level_dat = std::fs::read(fixture.level_dat_path(&tmp))
            .unwrap_or_else(|e| panic!("{}: failed to read level.dat: {e}", fixture.name));
        let header_storage_version =
            i32::from_le_bytes(level_dat[0..4].try_into().unwrap_or_else(|e| {
                panic!("{}: level.dat shorter than 4 bytes: {e}", fixture.name)
            }));
        assert_eq!(
            header_storage_version, fixture.header_storage_version,
            "{}: level.dat header storage version does not match the index",
            fixture.name
        );
    }
}

/// The imported set actually covers the format transitions it was brought
/// in for: both biome encodings, both sides of the negative-Y boundary, and
/// nothing pre-paletted (no fixture predates 1.12).
#[test]
fn fixtures_cover_expected_capabilities() {
    let all = support::fixtures();
    assert!(!all.is_empty());
    assert!(all.iter().any(|f| f.capabilities.biomes_2d));
    assert!(all.iter().any(|f| f.capabilities.biomes_3d));
    assert!(all.iter().any(|f| f.capabilities.negative_y));
    assert!(all.iter().any(|f| !f.capabilities.negative_y));
    assert!(all.iter().all(|f| f.capabilities.paletted));
}

/// `LevelSettings` was modeled against the single pre-existing fixture (a
/// 1.26 world) and does not yet parse every payload shape older worlds
/// write under the crate's strict decode. Tracked here, not hidden: a fix
/// that widens coverage has to shrink this list rather than pass silently
/// with no signal that anything changed.
#[test]
fn level_dat_strict_decode_known_gaps() {
    use bedrock_level::settings::LevelSettings;

    let known_failures: BTreeSet<&str> = [
        "v1_12",
        "v1_16",
        "v1_17_20",
        "v1_17_20_caves_and_cliffs",
        "v1_17_40",
        "v1_17_40_caves_and_cliffs",
        "v1_17_40_superflat",
        "v1_18",
        "v1_18_30",
        "v1_18_superflat",
        "v1_18_updated_superflat",
        "v1_19_30",
        "v1_20_81",
    ]
    .into_iter()
    .collect();

    let mut actual_failures = BTreeSet::new();
    for fixture in support::fixtures() {
        let (tmp, _db) = fixture.open();
        let data = std::fs::read(fixture.level_dat_path(&tmp))
            .unwrap_or_else(|e| panic!("{}: failed to read level.dat: {e}", fixture.name));
        if LevelSettings::read(data.as_slice()).is_err() {
            actual_failures.insert(fixture.name.clone());
        }
    }

    let actual_failures: BTreeSet<&str> = actual_failures.iter().map(String::as_str).collect();
    assert_eq!(
        actual_failures, known_failures,
        "the set of fixtures LevelSettings::read fails on has changed; \
         update known_failures to match"
    );
}

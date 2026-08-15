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

/// Every fixture's `level.dat` parses under the crate's strict decode
/// (`deny-unknown-fields`, the default feature). `LevelSettings` was
/// originally modeled against a single 1.26 world; this pins that the model
/// now covers the field set every imported world actually writes, across
/// the 1.12-to-1.20.81 range these fixtures cover.
#[test]
fn every_fixture_level_dat_parses_strictly() {
    use bedrock_level::settings::LevelSettings;

    for fixture in support::fixtures() {
        let (tmp, _db) = fixture.open();
        let data = std::fs::read(fixture.level_dat_path(&tmp))
            .unwrap_or_else(|e| panic!("{}: failed to read level.dat: {e}", fixture.name));
        LevelSettings::read(data.as_slice())
            .unwrap_or_else(|e| panic!("{}: LevelSettings::read failed: {e}", fixture.name));
    }
}

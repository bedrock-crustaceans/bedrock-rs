//! `chunk_version` dispatch against real fixture worlds: every chunk version
//! the fixture corpus actually contains resolves to a [`FormatProfile`]
//! consistent with what that fixture's own key scan found, and the
//! `0x2c`/`0x76` preference the corpus is checked for one real,
//! converter-stamped disagreement between the two.

mod support;

use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;

use bedrock_level::chunk_version::{BiomeForm, DecodeVersion, FormatProfile};
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

/// Every chunk version a fixture's `index.json` entry lists resolves to a
/// [`BiomeForm`] that agrees with that same fixture's `biomes_2d`/
/// `biomes_3d` capability flags -- both sourced independently, from a real
/// key scan of the on-disk world rather than from each other, so agreement
/// here is a genuine cross-check rather than the table checking itself.
///
/// A fixture can carry more than one chunk version (an updated world keeps
/// old chunks alongside newly-touched ones at the current version), so the
/// check is against the *set* of biome forms the fixture's chunk versions
/// resolve to, not a single value.
#[test]
fn fixture_chunk_versions_resolve_to_the_observed_biome_form() {
    let fixtures = support::fixtures();
    assert!(
        fixtures.iter().any(|f| f.capabilities.biomes_2d),
        "expected at least one biomes_2d fixture to cross-check against"
    );
    assert!(
        fixtures.iter().any(|f| f.capabilities.biomes_3d),
        "expected at least one biomes_3d fixture to cross-check against"
    );

    for fixture in &fixtures {
        assert!(
            !fixture.chunk_versions.is_empty(),
            "{}: index.json lists no chunk versions to check",
            fixture.name
        );

        let forms: BTreeSet<BiomeForm> = fixture
            .chunk_versions
            .iter()
            .map(|&version| FormatProfile::for_version(version).biomes)
            .collect();

        assert_eq!(
            forms.contains(&BiomeForm::Columns2d),
            fixture.capabilities.biomes_2d,
            "{}: chunk versions {:?} resolve to biome forms {:?}, disagreeing with \
             capabilities.biomes_2d = {}",
            fixture.name,
            fixture.chunk_versions,
            forms,
            fixture.capabilities.biomes_2d,
        );
        assert_eq!(
            forms.contains(&BiomeForm::Volumes3d),
            fixture.capabilities.biomes_3d,
            "{}: chunk versions {:?} resolve to biome forms {:?}, disagreeing with \
             capabilities.biomes_3d = {}",
            fixture.name,
            fixture.chunk_versions,
            forms,
            fixture.capabilities.biomes_3d,
        );
    }
}

/// The fixture-observed versions the task specifically calls out
/// (15, 21, 22, 25, 31, 39, 40) each land on the correct side of the
/// 2D/3D biome boundary. Redundant with the sweep above once it passes, but
/// pinned individually so a future change to the fact-boundary table trips
/// an assertion naming the exact version that regressed.
#[test]
fn specifically_observed_versions_land_on_the_expected_side_of_the_biome_boundary() {
    let two_d = [15u8, 21, 22];
    let three_d = [25u8, 31, 39, 40];

    for version in two_d {
        assert_eq!(
            FormatProfile::for_version(version).biomes,
            BiomeForm::Columns2d,
            "chunk version {version} expected 2D biomes"
        );
    }
    for version in three_d {
        assert_eq!(
            FormatProfile::for_version(version).biomes,
            BiomeForm::Volumes3d,
            "chunk version {version} expected 3D biomes"
        );
    }
}

/// Scans a fixture's raw database directly for every chunk's `0x2c`
/// (`ChunkVersion`) and `0x76` (`LegacyVersion`) byte, keyed by chunk
/// position. A from-scratch scan rather than a call into `ChunkRecords`,
/// since this is checking the raw on-disk values `DecodeVersion::select`
/// consumes, not re-deriving them through the code under test.
fn scan_version_keys(db: &Database) -> HashMap<ChunkPosition, (Option<u8>, Option<u8>)> {
    let mut out: HashMap<ChunkPosition, (Option<u8>, Option<u8>)> = HashMap::new();
    let mut keys = db.keys().expect("failed to iterate database");

    for kv in &mut keys {
        let key_bytes = kv.key();
        let mut cursor = Cursor::new(key_bytes.as_ref());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if key.dimension != Dimension::Overworld {
            continue;
        }
        let value_bytes = kv.value();
        let Some(&byte) = value_bytes.first() else {
            continue;
        };

        match key.data {
            KeyVariant::ChunkVersion => out.entry(key.chunk).or_default().0 = Some(byte),
            KeyVariant::LegacyVersion => out.entry(key.chunk).or_default().1 = Some(byte),
            _ => {}
        }
    }

    out
}

/// A converter-written world stamps a fixed value into `0x76` on every
/// chunk, independent of the chunk's real version, which is why
/// `DecodeVersion::select` must prefer `0x2c` rather than treat the two
/// interchangeably. This fixture is the real, on-disk example: every one of
/// its chunks carries `0x2c = 40` (its actual, paletted-3D-biome format)
/// alongside `0x76 = 7` (a stamp far too old to be real -- chunk version 7
/// is pre-palette, non-paletted-subchunk terrain, which this world's
/// content flatly is not).
#[test]
fn converter_stamped_0x76_disagrees_with_0x2c_on_a_real_fixture() {
    let fixtures = support::fixtures();
    let fixture = fixtures
        .iter()
        .find(|f| f.name == "v1_19_30")
        .expect("expected the v1_19_30 fixture to be present");

    let (_tmp, db) = fixture.open();
    let versions = scan_version_keys(&db);

    assert!(
        !versions.is_empty(),
        "{}: found no 0x2c/0x76 records to check",
        fixture.name
    );

    let mut disagreements = 0usize;
    for (chunk_version_0x2c, legacy_version_0x76) in versions.values() {
        let (Some(chunk_version_0x2c), Some(legacy_version_0x76)) =
            (chunk_version_0x2c, legacy_version_0x76)
        else {
            continue;
        };
        if chunk_version_0x2c != legacy_version_0x76 {
            disagreements += 1;
        }

        // Whatever this chunk's raw bytes are, resolution must always pick
        // 0x2c over 0x76.
        let resolved = DecodeVersion::select(Some(*chunk_version_0x2c), Some(*legacy_version_0x76));
        assert_eq!(resolved, DecodeVersion::Found(*chunk_version_0x2c));
    }

    assert!(
        disagreements > 0,
        "{}: expected at least one chunk where 0x2c and 0x76 disagree \
         (the converter-stamp case), found none",
        fixture.name
    );

    // Pin the exact real values this fixture carries, so a future change to
    // the fixture tarball is noticed rather than the assertion above
    // silently degrading to a different disagreement.
    let sample = versions
        .values()
        .find(|(a, b)| matches!((a, b), (Some(a), Some(b)) if a != b))
        .expect("already established at least one disagreement exists");
    assert_eq!(*sample, (Some(40), Some(7)));
}

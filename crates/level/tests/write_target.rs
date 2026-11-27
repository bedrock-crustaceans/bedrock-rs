//! The Phase 5 "write target: default to newest, allow pinning" acceptance tests.
//!
//! Exercises [`Chunk::encode_as`]'s structural retargeting against the imported
//! version-diverse fixtures (selected by [`support::fixtures`], never hardcoded), plus the
//! derived-default and family-grouping guarantees [`bedrock_level::chunk_version`] makes.

pub mod support;

use std::collections::HashMap;

use bedrock_level::chunk::ChunkRecords;
use bedrock_level::chunk_version::{
    DEFAULT_WRITE_VERSION, FormatProfile, KNOWN_CHUNK_VERSION_CEILING,
};
use bedrock_level::db::Database;
use bedrock_level::key::{KeyVariant, subchunk_index_to_key};
use bedrock_level::level::Level;
use bedrock_level::level_chunk::{Chunk, RetargetOutcome, StructuralShortfall, Terrain};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;
use flate2::read::GzDecoder;
use tar::Archive;

/// Mirrors `level_test.rs`'s own copy: `Level::open` needs an unopened directory, so a
/// fixture's tarball is extracted without also opening a `Database` on it (see that file's
/// doc comment on `extract_fixture` for why the two handles cannot coexist).
fn extract_fixture(fixture: &support::Fixture) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz_path = format!("tests/fixtures/{}", fixture.tarball);
    let tar_gz = std::fs::File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
    let tar = GzDecoder::new(tar_gz);
    Archive::new(tar)
        .unpack(tmp.path())
        .unwrap_or_else(|e| panic!("{tar_gz_path}: failed to unpack: {e}"));
    tmp
}

fn open_fixture_level(fixture: &support::Fixture) -> (tempfile::TempDir, Level) {
    let tmp = extract_fixture(fixture);
    let level = Level::open(tmp.path().join("world"))
        .unwrap_or_else(|e| panic!("{}: Level::open failed: {e}", fixture.name));
    (tmp, level)
}

/// A fresh, empty `Database` to write an encode into and read it straight back, the same
/// pattern `level_test.rs`'s round-trip test uses.
fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("db");
    let db = Database::open(db_path.to_str().unwrap()).unwrap();
    (dir, db)
}

/// The first chunk position in `level` (any dimension) that actually carries terrain --
/// `Terrain::SubChunks` with at least one subchunk, or `Terrain::Legacy` with a record --
/// skipping generation-progress stubs that have nothing for retargeting to move. `None` if
/// the fixture has no such chunk at all (not expected for any imported fixture, but a real
/// outcome this helper reports rather than panicking on, since a bulk-selection helper
/// should not assume every fixture's shape).
fn first_chunk_with_terrain(level: &Level) -> Option<(Dimension, ChunkPosition, Chunk)> {
    for &dimension in level.dimensions() {
        for position in level.chunk_positions(dimension) {
            let Some(chunk) = level.chunk(dimension, position).ok().flatten() else {
                continue;
            };
            let has_terrain = match chunk.terrain() {
                Terrain::Legacy(legacy) => legacy.is_some(),
                Terrain::SubChunks(subchunks) => !subchunks.is_empty(),
            };
            if has_terrain {
                return Some((dimension, position, chunk));
            }
        }
    }
    None
}

/// Encodes `records` into a fresh database and decodes it straight back into a `Chunk`, to
/// check what a target actually round-trips to rather than only inspecting the
/// `StructuralWriteReport`.
fn round_trip(records: &ChunkRecords, position: ChunkPosition, dimension: Dimension) -> Chunk {
    let (_dir, db) = fresh_db();
    records.write(&db).expect("write encoded records");
    let read_back = ChunkRecords::read(&db, position, dimension)
        .expect("read back")
        .expect("round-tripped chunk is missing entirely");
    Chunk::decode(read_back, &db).expect("decode round-tripped chunk")
}

/// A target guaranteed to sit in a different chunk-version family than `family`: one past
/// the family's own ceiling when there is room (the common "modernize an old chunk" shape),
/// otherwise one below the family's floor (only the unbounded last family, `25..=255`,
/// has no room above).
fn out_of_family_target(family: std::ops::RangeInclusive<u8>) -> u8 {
    match family.end().checked_add(1) {
        Some(above) => above,
        None => family.start().saturating_sub(1),
    }
}

#[test]
fn every_fixture_encodes_byte_identical_at_its_own_version() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);
        let Some((dimension, position, chunk)) = first_chunk_with_terrain(&level) else {
            continue;
        };

        let original = ChunkRecords::read(level.raw(), position, dimension)
            .unwrap()
            .expect("position came from the level's own index");

        let own_version = chunk.chunk_version();
        let (records, report) = chunk
            .encode_as(position, dimension, own_version)
            .unwrap_or_else(|e| panic!("{}: encode_as(own version) failed: {e}", fixture.name));

        assert_eq!(report.requested, own_version, "{}", fixture.name);
        assert_eq!(report.achieved, own_version, "{}", fixture.name);
        assert_eq!(
            report.retarget,
            RetargetOutcome::Unretargeted,
            "{}: {report:?}",
            fixture.name
        );
        assert_eq!(report.shortfall, None, "{}: {report:?}", fixture.name);

        let original_map: HashMap<_, Vec<u8>> = original
            .iter()
            .map(|(key, bytes)| (key.data, bytes.to_vec()))
            .collect();
        let round_tripped_map: HashMap<_, Vec<u8>> = records
            .iter()
            .map(|(key, bytes)| (key.data, bytes.to_vec()))
            .collect();
        assert_eq!(
            original_map, round_tripped_map,
            "{}: encode_as at the chunk's own version must reproduce the original records \
             byte for byte -- this is the pre-existing Chunk::encode_at regression",
            fixture.name
        );

        checked += 1;
    }

    assert!(checked > 0, "no fixture contributed a chunk with terrain");
}

#[test]
fn every_fixture_retargets_to_its_family_ceiling_and_decodes_strictly() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);
        let Some((dimension, position, chunk)) = first_chunk_with_terrain(&level) else {
            continue;
        };

        let family = FormatProfile::version_family(chunk.chunk_version());
        // The newest *confirmed* version in the family, not the family's literal (possibly
        // open-ended, up to 255) upper bound -- see `chunk_version.rs`'s
        // `KNOWN_CHUNK_VERSION_CEILING` doc comment for why writing past it would be an
        // extrapolation rather than a real target.
        let target = (*family.end()).min(KNOWN_CHUNK_VERSION_CEILING);

        let (records, report) = chunk
            .encode_as(position, dimension, target)
            .unwrap_or_else(|e| panic!("{}: encode_as(family ceiling) failed: {e}", fixture.name));

        assert_eq!(report.requested, target, "{}", fixture.name);
        assert_eq!(report.achieved, target, "{}: {report:?}", fixture.name);
        assert_eq!(report.shortfall, None, "{}: {report:?}", fixture.name);
        assert_eq!(
            report.retarget.moved(),
            target != chunk.chunk_version(),
            "{}: {report:?}",
            fixture.name
        );

        let round_tripped = round_trip(&records, position, dimension);
        assert_eq!(
            round_tripped.chunk_version(),
            target,
            "{}: DecodeVersion::Found(target) expected",
            fixture.name
        );
        assert_eq!(
            round_tripped.terrain(),
            chunk.terrain(),
            "{}: terrain must be semantically identical after an in-family retarget",
            fixture.name
        );
        assert_eq!(
            round_tripped.biomes(),
            chunk.biomes(),
            "{}: biomes must be semantically identical after an in-family retarget",
            fixture.name
        );

        checked += 1;
    }

    assert!(checked > 0, "no fixture contributed a chunk with terrain");
}

#[test]
fn every_fixture_reports_a_shortfall_and_stamps_the_achieved_version_across_families() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);
        let Some((dimension, position, chunk)) = first_chunk_with_terrain(&level) else {
            continue;
        };

        let family = FormatProfile::version_family(chunk.chunk_version());
        let target = out_of_family_target(family.clone());
        assert!(
            !family.contains(&target),
            "{}: test bug, chosen target {target} is inside {family:?}",
            fixture.name
        );

        let (records, report) = chunk
            .encode_as(position, dimension, target)
            .unwrap_or_else(|e| panic!("{}: encode_as(cross-family) failed: {e}", fixture.name));

        assert_eq!(report.requested, target, "{}", fixture.name);

        if family == (8..=24) {
            // 8-24 targeted one version past its own ceiling lands on 25 -- the one
            // cross-family conversion this crate actually performs: promoting 2D `0x2d`
            // biomes to 3D `0x2b` volumes (see the Phase 5 "Cross-family retargeting: 2D ->
            // 3D biomes" task in TODO.md). This is not a refusal: `achieved` reaches
            // `target` directly, `shortfall` stays `None`, and the biome record on disk
            // changes kind. Every other family below still refuses (asserted in the `else`
            // branch), since the reverse (3D to 2D) is lossy and non-paletted/legacy-terrain
            // conversions need a provider this crate does not invoke here.
            assert_eq!(
                target, 25,
                "{}: test bug, expected the promotion target to be 25",
                fixture.name
            );
            assert_eq!(report.achieved, target, "{}: {report:?}", fixture.name);
            assert_eq!(report.shortfall, None, "{}: {report:?}", fixture.name);
            assert_eq!(
                report.retarget,
                RetargetOutcome::Promoted,
                "{}: {report:?}",
                fixture.name
            );

            assert!(
                records
                    .iter()
                    .any(|(key, _)| key.data == KeyVariant::Biome3d),
                "{}: a promoted chunk must carry 0x2b",
                fixture.name
            );
            assert!(
                !records
                    .iter()
                    .any(|(key, _)| key.data == KeyVariant::HeightMap),
                "{}: a promoted chunk must not carry 0x2d",
                fixture.name
            );

            let stamp = records
                .iter()
                .find(|(key, _)| key.data == KeyVariant::ChunkVersion)
                .map(|(_, bytes)| bytes[0]);
            assert_eq!(
                stamp,
                Some(25),
                "{}: the 0x2c stamp must equal the promoted (achieved) version",
                fixture.name
            );
        } else {
            assert_eq!(
                report.shortfall,
                Some(StructuralShortfall::DifferentFamily),
                "{}: {report:?}",
                fixture.name
            );
            // Above the family, this clamps down to the family's own ceiling. Below the
            // family, it does NOT clamp up to the family floor -- that would manufacture a
            // version nobody asked for (and, for the 25..=255 family, could silently walk
            // the chunk into the 25-28 offset window as a side effect). Below-family
            // instead leaves the chunk at its own current version, unretargeted. Either way
            // the achieved version is never the requested target itself, since that would
            // overstate what the write actually did (architecture decision 6).
            let expected_achieved = if target > *family.end() {
                *family.end()
            } else {
                chunk.chunk_version()
            };
            assert_eq!(
                report.achieved, expected_achieved,
                "{}: {report:?}",
                fixture.name
            );
            assert_ne!(report.achieved, target, "{}: {report:?}", fixture.name);
            // `retarget.moved()` tracks whether the stamp actually moved off the chunk's
            // own decode version -- not whether a shortfall occurred. The two usually
            // coincide for a cross-family request, but not when the chunk already sat
            // exactly at its own family's edge (clamping then reproduces its own version).
            assert_eq!(
                report.retarget.moved(),
                report.achieved != chunk.chunk_version(),
                "{}: {report:?}",
                fixture.name
            );

            // The stamp written on disk is the achieved version, never the requested one.
            let stamp = records
                .iter()
                .find(|(key, _)| key.data == KeyVariant::ChunkVersion)
                .map(|(_, bytes)| bytes[0]);
            assert_eq!(
                stamp,
                Some(report.achieved),
                "{}: the 0x2c stamp must equal the achieved version, not the requested target",
                fixture.name
            );
        }

        checked += 1;
    }

    assert!(checked > 0, "no fixture contributed a chunk with terrain");
}

/// Pins the 25-28 (offset) versus 29+ (no offset) subchunk key-index window against a real
/// negative-Y 3D-biome fixture chunk: retargeting from its own version (39/40) down into the
/// window (27) and back up again must change the on-disk subchunk key form each way while
/// leaving the decoded terrain and biomes exactly as they started.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one chunk retargeted down into the window and back, with the key form and the decoded contents checked at each step"
)]
fn retargeting_a_negative_y_chunk_through_the_offset_window_round_trips_semantically() {
    let target_in_window = 27u8;

    let candidate = support::fixtures()
        .into_iter()
        .find(|f| f.capabilities.negative_y && f.chunk_versions.iter().any(|&v| v >= 39));
    let Some(fixture) = candidate else {
        panic!("no fixture with capabilities.negative_y and a chunk_version >= 39 found");
    };

    let (_tmp, level) = open_fixture_level(&fixture);

    // Prefer a chunk that actually has a negative subchunk index, so the offset math is
    // exercised on a real negative value rather than only on non-negative ones.
    let mut chosen: Option<(Dimension, ChunkPosition, Chunk)> = None;
    'search: for &dimension in level.dimensions() {
        for position in level.chunk_positions(dimension) {
            let Some(chunk) = level.chunk(dimension, position).ok().flatten() else {
                continue;
            };
            if let Terrain::SubChunks(subchunks) = chunk.terrain()
                && subchunks.keys().any(|&index| index < 0)
            {
                chosen = Some((dimension, position, chunk));
                break 'search;
            }
        }
    }
    let (dimension, position, chunk) = chosen
        .or_else(|| first_chunk_with_terrain(&level))
        .unwrap_or_else(|| panic!("{}: no chunk with terrain found", fixture.name));

    let own_version = chunk.chunk_version();
    let family = FormatProfile::version_family(own_version);
    assert!(
        family.contains(&target_in_window),
        "{}: chunk's own family {family:?} must contain the offset window for this test to \
         be an in-family retarget",
        fixture.name
    );

    // Down into the window.
    let (down_records, down_report) = chunk
        .encode_as(position, dimension, target_in_window)
        .unwrap_or_else(|e| panic!("{}: encode_as(27) failed: {e}", fixture.name));
    assert_eq!(down_report.achieved, target_in_window);
    assert_eq!(down_report.shortfall, None, "{down_report:?}");

    let Terrain::SubChunks(subchunks) = chunk.terrain() else {
        panic!("{}: expected subchunk terrain", fixture.name);
    };
    for &index in subchunks.keys() {
        let expected_key_index = subchunk_index_to_key(index, target_in_window, dimension);
        let actual_key_index = down_records
            .iter()
            .find_map(|(key, _)| match key.data {
                KeyVariant::SubChunk { index: key_index }
                    if key.subchunk_absolute_index(target_in_window) == Some(index) =>
                {
                    Some(key_index)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{}: subchunk {index} missing after retarget", fixture.name));
        assert_eq!(
            actual_key_index, expected_key_index,
            "{}: subchunk {index}'s key must carry the +4 offset inside the 25-28 window",
            fixture.name
        );
        // And it must differ from the un-offset form whenever the offset is nonzero for
        // this index (every real absolute index in this corpus is far from i8's extremes,
        // so saturation never makes the two forms coincide).
        assert_ne!(
            actual_key_index, index,
            "{}: subchunk {index}",
            fixture.name
        );
    }

    let down = round_trip(&down_records, position, dimension);
    assert_eq!(down.chunk_version(), target_in_window);
    assert_eq!(down.terrain(), chunk.terrain(), "{}", fixture.name);
    assert_eq!(down.biomes(), chunk.biomes(), "{}", fixture.name);

    // Back up again, from the retargeted (version-27) chunk.
    let (up_records, up_report) = down
        .encode_as(position, dimension, own_version)
        .unwrap_or_else(|e| panic!("{}: encode_as(back up) failed: {e}", fixture.name));
    assert_eq!(up_report.achieved, own_version);
    assert_eq!(up_report.shortfall, None, "{up_report:?}");

    let Terrain::SubChunks(down_subchunks) = down.terrain() else {
        panic!("{}: expected subchunk terrain", fixture.name);
    };
    for &index in down_subchunks.keys() {
        let expected_key_index = subchunk_index_to_key(index, own_version, dimension);
        let actual_key_index = up_records
            .iter()
            .find_map(|(key, _)| match key.data {
                KeyVariant::SubChunk { index: key_index }
                    if key.subchunk_absolute_index(own_version) == Some(index) =>
                {
                    Some(key_index)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{}: subchunk {index} missing after retarget", fixture.name));
        assert_eq!(
            actual_key_index, expected_key_index,
            "{}: subchunk {index}'s key must lose the offset back outside the window",
            fixture.name
        );
    }

    let up = round_trip(&up_records, position, dimension);
    assert_eq!(up.chunk_version(), own_version);
    assert_eq!(
        up.terrain(),
        chunk.terrain(),
        "{}: full round trip",
        fixture.name
    );
    assert_eq!(
        up.biomes(),
        chunk.biomes(),
        "{}: full round trip",
        fixture.name
    );
}

/// Pins the BLOCKER fix: a chunk with a subchunk deeper than the 25-28 offset window can
/// represent (absolute index below `-4`) must refuse a retarget into that window rather
/// than silently writing a wrong, sign-flipped key byte and reporting success.
///
/// Selected by *actually scanning* `index.json`'s recorded `subchunk_height_range` for the
/// fixture with the lowest real Overworld floor -- not by fixture name or file order, which
/// is exactly what let the original bug slip past the existing offset-window test (it
/// matches fixtures in file order and stops at the first one satisfying `negative_y`, which
/// is `v1_18`, floor `-64` / index `-4`, representable; it never reaches the fixture that
/// actually breaks the window, whichever one that turns out to be as fixtures are added or
/// removed).
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "selects the deepest fixture chunk by scan, then checks the refusal, the version stamp and the exact key byte"
)]
fn retargeting_into_the_offset_window_is_refused_for_a_chunk_too_deep_to_represent() {
    let fixture = support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.negative_y)
        .min_by_key(|f| {
            f.subchunk_height_range
                .get("overworld")
                .map(|[min_y, _]| *min_y)
        })
        .expect("at least one negative_y fixture");
    let floor = fixture
        .subchunk_height_range
        .get("overworld")
        .unwrap_or_else(|| {
            panic!(
                "{}: no recorded overworld subchunk_height_range",
                fixture.name
            )
        })[0];
    let expected_min_index = i8::try_from(floor.div_euclid(16))
        .unwrap_or_else(|_| panic!("{}: floor {floor} out of i8 range", fixture.name));
    assert!(
        expected_min_index < bedrock_level::key::SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_FLOOR,
        "{}: expected the corpus's deepest fixture to have a floor below the window's -4 \
         representable floor; got index {expected_min_index} (block floor {floor}) -- if this \
         fires, no fixture in the corpus exercises the blocker this test pins any more",
        fixture.name
    );

    let (_tmp, level) = open_fixture_level(&fixture);
    let mut chosen: Option<(Dimension, ChunkPosition, Chunk)> = None;
    'search: for &dimension in level.dimensions() {
        if dimension != Dimension::Overworld {
            // The offset window (and this fixture's recorded floor) only ever applies to
            // the Overworld -- see `subchunk_key_index_is_offset`.
            continue;
        }
        for position in level.chunk_positions(dimension) {
            let Some(chunk) = level.chunk(dimension, position).ok().flatten() else {
                continue;
            };
            if let Terrain::SubChunks(subchunks) = chunk.terrain()
                && subchunks.keys().next() == Some(&expected_min_index)
            {
                chosen = Some((dimension, position, chunk));
                break 'search;
            }
        }
    }
    let (dimension, position, chunk) = chosen.unwrap_or_else(|| {
        panic!(
            "{}: no Overworld chunk found with a subchunk at the recorded minimum index {expected_min_index}",
            fixture.name
        )
    });

    let own_version = chunk.chunk_version();
    let target_in_window = 27u8;
    let family = FormatProfile::version_family(own_version);
    assert!(
        family.contains(&target_in_window),
        "{}: expected the chunk's own family to contain the offset window",
        fixture.name
    );

    let (records, report) = chunk
        .encode_as(position, dimension, target_in_window)
        .unwrap_or_else(|e| panic!("{}: encode_as(27) failed: {e}", fixture.name));

    assert_eq!(report.requested, target_in_window, "{}", fixture.name);
    assert_eq!(report.achieved, own_version, "{}: {report:?}", fixture.name);
    assert_eq!(
        report.retarget,
        RetargetOutcome::Unretargeted,
        "{}: {report:?}",
        fixture.name
    );
    assert_eq!(
        report.shortfall,
        Some(StructuralShortfall::DepthUnrepresentableInOffsetWindow),
        "{}: {report:?}",
        fixture.name
    );

    // The stamp on disk stays at the chunk's own version -- never 27, which would overstate
    // what was actually converted (decision 6).
    let stamp = records
        .iter()
        .find(|(key, _)| key.data == KeyVariant::ChunkVersion)
        .map(|(_, bytes)| bytes[0]);
    assert_eq!(
        stamp,
        Some(own_version),
        "{}: the 0x2c stamp must stay at the chunk's own version",
        fixture.name
    );

    // The deepest subchunk's key byte must be exactly what its own (unchanged) version
    // produces -- never the window-shifted form the original bug wrote. The un-offset form
    // is itself negative here (that is an ordinary, correct negative-Y key outside the
    // window); the window-shifted form would ALSO be negative, but a different, wrong
    // value, so this checks the exact byte rather than merely its sign.
    let expected_key_index = subchunk_index_to_key(expected_min_index, own_version, dimension);
    let window_shifted_key_index = expected_min_index.saturating_add(4);
    assert!(
        window_shifted_key_index < 0,
        "test sanity: expected the refused window-shifted byte to itself be negative"
    );
    let actual_key_index = records
        .iter()
        .find_map(|(key, _)| match key.data {
            KeyVariant::SubChunk { index: key_index }
                if key.subchunk_absolute_index(own_version) == Some(expected_min_index) =>
            {
                Some(key_index)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{}: deepest subchunk missing after encode", fixture.name));
    assert_eq!(
        actual_key_index, expected_key_index,
        "{}: the deepest subchunk's key must reflect the achieved (unchanged) version",
        fixture.name
    );
    assert_ne!(
        actual_key_index, window_shifted_key_index,
        "{}: the deepest subchunk's key must not carry the refused offset-window shift",
        fixture.name
    );
}

/// The derived write-target default is the newest chunk version this crate's profile table
/// actually confirms -- not a hardcoded literal repeated here. `DEFAULT_WRITE_VERSION` and
/// `KNOWN_CHUNK_VERSION_CEILING` are two names tied together by a `const _: () = assert!`
/// beside `INTRODUCED_IN` in `chunk_version.rs` (which also has its own table-length
/// assertion); this test additionally pins that a freshly opened `Level` actually uses that
/// derived value, so the tie is checked end to end from the public API rather than only
/// inside `chunk_version.rs`'s own module.
#[test]
fn default_write_version_is_the_derived_newest_confirmed_version() {
    assert_eq!(DEFAULT_WRITE_VERSION, KNOWN_CHUNK_VERSION_CEILING);

    let fixture = support::fixtures()
        .into_iter()
        .next()
        .expect("at least one fixture");
    let (_tmp, level) = open_fixture_level(&fixture);
    assert_eq!(level.write_version(), DEFAULT_WRITE_VERSION);
}

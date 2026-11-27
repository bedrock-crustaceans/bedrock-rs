//! Cross-family retargeting: 2D (`0x2d`) to 3D (`0x2b`) biome promotion.
//!
//! Exercises [`Chunk::encode_as`]'s promotion path (source chunk version in the 8-24 family,
//! target >= 25) against the imported fixtures, selected by capability
//! ([`support::Fixture::capabilities`]) rather than by name.

mod support;

use bedrock_level::biome::{Biomes3d, COLUMNS};
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::chunk_version::{DEFAULT_WRITE_VERSION, fragment_count_for};
use bedrock_level::db::Database;
use bedrock_level::key::{KeyVariant, subchunk_index_to_key};
use bedrock_level::level::Level;
use bedrock_level::level_chunk::{
    Chunk, ChunkBiomes, RetargetOutcome, StructuralShortfall, Terrain,
};
use bedrock_level::types::{BlockPosition, ChunkPosition};
use bedrock_shared::world::dimension::Dimension;
use flate2::read::GzDecoder;
use tar::Archive;

/// Mirrors `write_target.rs`'s own copy: `Level::open` needs an unopened directory.
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

fn fresh_db() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("db");
    let db = Database::open(db_path.to_str().unwrap()).unwrap();
    (dir, db)
}

fn round_trip(records: ChunkRecords, position: ChunkPosition, dimension: Dimension) -> Chunk {
    let (_dir, db) = fresh_db();
    records.write(&db).expect("write encoded records");
    let read_back = ChunkRecords::read(&db, position, dimension)
        .expect("read back")
        .expect("round-tripped chunk is missing entirely");
    Chunk::decode(read_back, &db).expect("decode round-tripped chunk")
}

/// The first chunk (any dimension) that carries `Terrain::SubChunks` with at least one
/// subchunk *and* a `0x2d` [`ChunkBiomes::Columns`] record -- the shape a promotion needs.
/// `None` if the fixture has no such chunk (not expected for any 2D fixture this crate
/// imports, but reported rather than assumed).
fn first_promotable_chunk(level: &Level) -> Option<(Dimension, ChunkPosition, Chunk)> {
    for &dimension in level.dimensions() {
        for position in level.chunk_positions(dimension) {
            let Some(chunk) = level.chunk(dimension, position).ok().flatten() else {
                continue;
            };
            let has_subchunks = matches!(chunk.terrain(), Terrain::SubChunks(s) if !s.is_empty());
            let has_columns = matches!(chunk.biomes(), Some(ChunkBiomes::Columns(_)));
            if has_subchunks && has_columns {
                return Some((dimension, position, chunk));
            }
        }
    }
    None
}

/// The first chunk carrying `Terrain::SubChunks` with at least one subchunk, regardless of
/// its biome record -- used for the 3D-fixture-targeted-at-8-24 case, which does not need a
/// promotable chunk (it exercises the ordinary family-clamp refusal).
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

/// Every fixture whose capabilities are 2D-biomes-only (`biomes_2d && !biomes_3d`) -- the
/// promotion source family, 8-24, never mixed with a 3D record in the same fixture.
fn pure_2d_fixtures() -> Vec<support::Fixture> {
    support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.biomes_2d && !f.capabilities.biomes_3d)
        .collect()
}

/// Asserts that `volumes`' `biome_at` agrees with `source`'s per-column ids at every column,
/// for every subchunk index `0..fragment_count` -- the promotion is lossless upward, so a
/// promoted volume must answer every subchunk exactly as the source's flat 2D record would
/// have for that whole-height column.
fn assert_promoted_biomes_match_source(
    volumes: &Biomes3d,
    source: &bedrock_level::biome::Biomes2d,
    fragment_count: usize,
    context: &str,
) {
    for subchunk_index in 0..fragment_count {
        for z in 0..16usize {
            for x in 0..16usize {
                let expected = u32::from(source.id_at(x, z).unwrap());
                let actual = volumes
                    .biome_at(subchunk_index, BlockPosition(x as u8, 0, z as u8))
                    .unwrap_or_else(|| {
                        panic!("{context}: biome_at({subchunk_index}, {x}, {z}) returned None")
                    });
                assert_eq!(
                    actual, expected,
                    "{context}: subchunk {subchunk_index} column ({x}, {z})"
                );
            }
        }
    }
}

fn assert_heightmap_bytes_match(
    source: &bedrock_level::height_map::HeightMap,
    promoted: &Biomes3d,
    context: &str,
) {
    for i in 0..COLUMNS {
        let expected = source.heights()[i] as u16;
        let actual = promoted.heightmap()[i];
        assert_eq!(actual, expected, "{context}: heightmap column {i}");
    }
}

#[test]
fn every_2d_fixture_promotes_to_the_newest_version_and_decodes_strictly() {
    let fixtures = pure_2d_fixtures();
    assert!(
        !fixtures.is_empty(),
        "expected at least one pure-2D fixture"
    );
    let mut checked = 0usize;

    for fixture in &fixtures {
        let (_tmp, level) = open_fixture_level(fixture);
        let Some((dimension, position, chunk)) = first_promotable_chunk(&level) else {
            continue;
        };

        let Some(ChunkBiomes::Columns(source_heightmap)) = chunk.biomes() else {
            panic!("{}: expected Columns biomes", fixture.name);
        };
        let source_biomes_2d = source_heightmap.biomes().clone();
        let source_heightmap = source_heightmap.clone();

        let target = DEFAULT_WRITE_VERSION;
        let (records, report) = chunk
            .encode_as(position, dimension, target)
            .unwrap_or_else(|e| panic!("{}: encode_as(newest) failed: {e}", fixture.name));

        assert_eq!(report.requested, target, "{}", fixture.name);
        assert_eq!(report.achieved, target, "{}: {report:?}", fixture.name);
        assert_eq!(
            report.retarget,
            RetargetOutcome::Promoted,
            "{}: {report:?}",
            fixture.name
        );
        assert_eq!(report.shortfall, None, "{}: {report:?}", fixture.name);

        // `0x2d` absent, `0x2b` present, in the encoded records themselves.
        assert!(
            !records
                .iter()
                .any(|(key, _)| key.data == KeyVariant::HeightMap),
            "{}: 0x2d must be absent after promotion",
            fixture.name
        );
        assert!(
            records
                .iter()
                .any(|(key, _)| key.data == KeyVariant::Biome3d),
            "{}: 0x2b must be present after promotion",
            fixture.name
        );

        let decoded = round_trip(records, position, dimension);
        assert_eq!(
            decoded.chunk_version(),
            target,
            "{}: must decode strictly at the achieved version",
            fixture.name
        );
        let Some(ChunkBiomes::Volumes(volumes)) = decoded.biomes() else {
            panic!(
                "{}: expected Volumes biomes after promotion, got {:?}",
                fixture.name,
                decoded.biomes()
            );
        };

        let fragment_count = fragment_count_for(target).get();
        assert_eq!(
            volumes.fragments().len(),
            fragment_count,
            "{}: fragment count must match fragment_count_for(target)",
            fixture.name
        );
        assert_promoted_biomes_match_source(
            volumes,
            &source_biomes_2d,
            fragment_count,
            &fixture.name,
        );
        assert_heightmap_bytes_match(&source_heightmap, volumes, &fixture.name);

        checked += 1;
    }

    assert!(
        checked > 0,
        "no pure-2D fixture contributed a promotable chunk"
    );
}

#[test]
fn every_2d_fixture_promotes_into_the_offset_window_with_correct_subchunk_keys() {
    let target_in_window = 27u8;
    let fixtures = pure_2d_fixtures();
    let mut checked = 0usize;

    for fixture in &fixtures {
        let (_tmp, level) = open_fixture_level(fixture);
        let Some((dimension, position, chunk)) = first_promotable_chunk(&level) else {
            continue;
        };

        let (records, report) = chunk
            .encode_as(position, dimension, target_in_window)
            .unwrap_or_else(|e| panic!("{}: encode_as(27) failed: {e}", fixture.name));

        // No 2D-biomes fixture in this corpus has a subchunk deep enough to trip the
        // offset-window depth refusal (that only happens for negative-Y worlds, which this
        // family predates) -- assert that explicitly rather than silently branching on it.
        assert_eq!(
            report.shortfall, None,
            "{}: expected a clean promotion into the offset window, got {report:?}",
            fixture.name
        );
        assert_eq!(
            report.retarget,
            RetargetOutcome::Promoted,
            "{}: {report:?}",
            fixture.name
        );
        assert_eq!(report.achieved, target_in_window, "{}", fixture.name);

        let Terrain::SubChunks(subchunks) = chunk.terrain() else {
            panic!("{}: expected subchunk terrain", fixture.name);
        };
        for &index in subchunks.keys() {
            let expected_key_index = subchunk_index_to_key(index, target_in_window, dimension);
            let actual_key_index = records
                .iter()
                .find_map(|(key, _)| match key.data {
                    KeyVariant::SubChunk { index: key_index }
                        if key.subchunk_absolute_index(target_in_window) == Some(index) =>
                    {
                        Some(key_index)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| {
                    panic!("{}: subchunk {index} missing after promotion", fixture.name)
                });
            assert_eq!(
                actual_key_index, expected_key_index,
                "{}: subchunk {index}'s key must carry the +4 offset inside the 25-28 window",
                fixture.name
            );
        }

        let decoded = round_trip(records, position, dimension);
        assert_eq!(
            decoded.chunk_version(),
            target_in_window,
            "{}",
            fixture.name
        );
        assert!(
            matches!(decoded.biomes(), Some(ChunkBiomes::Volumes(_))),
            "{}: expected Volumes biomes after promotion into the window",
            fixture.name
        );

        checked += 1;
    }

    assert!(
        checked > 0,
        "no pure-2D fixture contributed a promotable chunk"
    );
}

#[test]
fn a_3d_fixture_targeted_at_8_24_still_reports_different_family() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.capabilities.biomes_3d)
        .expect("at least one biomes_3d fixture");

    let (_tmp, level) = open_fixture_level(&fixture);
    let Some((dimension, position, chunk)) = first_chunk_with_terrain(&level) else {
        panic!("{}: no chunk with terrain found", fixture.name);
    };

    let own_version = chunk.chunk_version();
    assert!(
        own_version >= 25,
        "{}: expected a 3D-biome fixture's chunk to sit at version 25+, got {own_version}",
        fixture.name
    );

    let target = 15u8; // Squarely inside the 8-24 family -- below the 3D chunk's own family.
    let (records, report) = chunk
        .encode_as(position, dimension, target)
        .unwrap_or_else(|e| panic!("{}: encode_as(15) failed: {e}", fixture.name));

    assert_eq!(report.requested, target, "{}", fixture.name);
    assert_eq!(report.achieved, own_version, "{}: {report:?}", fixture.name);
    assert_eq!(
        report.retarget,
        RetargetOutcome::Unretargeted,
        "{}: {report:?}",
        fixture.name
    );
    assert_eq!(
        report.shortfall,
        Some(StructuralShortfall::DifferentFamily),
        "{}: {report:?}",
        fixture.name
    );

    let stamp = records
        .iter()
        .find(|(key, _)| key.data == KeyVariant::ChunkVersion)
        .map(|(_, bytes)| bytes[0]);
    assert_eq!(stamp, Some(own_version), "{}", fixture.name);
}

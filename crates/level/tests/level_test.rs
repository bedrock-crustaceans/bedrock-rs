//! Exercises the level layer (`Level`, `Chunk`, `DimensionRef`) against the
//! imported version-diverse fixtures, as opposed to the lower-level tests
//! that drive `Database`/`ChunkRecords`/`Key` directly.

mod support;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Cursor;

use bedrock_level::Greedy;
use bedrock_level::biome::Biomes3d;
use bedrock_level::chunk::ChunkRecords;
use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::level::Level;
use bedrock_level::level_chunk::{Chunk, ChunkBlockPosition, Terrain};
use bedrock_level::types::{BlockPosition, ChunkPosition};
use bedrock_shared::world::dimension::Dimension;
use flate2::read::GzDecoder;
use tar::Archive;

/// Unpacks `fixture`'s tarball into a fresh `TempDir`, mirroring
/// `support::Fixture::open` but *without* opening a `Database` on it --
/// `Level::open` does that itself, and `rusty-leveldb` holds an exclusive
/// lock on the store directory for as long as any handle on it is alive, so
/// a test that wants a `Level` must not also hold `Fixture::open`'s own
/// `Database` handle open at the same time. Where a test also needs direct
/// `Database` access, it goes through `Level::raw` on the same handle
/// `Level::open` already holds, rather than opening a second one.
fn extract_fixture(fixture: &support::Fixture) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz_path = format!("tests/fixtures/{}", fixture.tarball);
    let tar_gz = File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
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

/// Maps `index.json`'s lowercase dimension-name strings to `Dimension`.
/// Kept separate from any mapping inside `bedrock_level` itself so the
/// dimension checks below are not simply comparing the crate against its
/// own internal table.
fn dimension_from_fixture_name(name: &str) -> Option<Dimension> {
    match name {
        "overworld" => Some(Dimension::Overworld),
        "nether" => Some(Dimension::Nether),
        "end" => Some(Dimension::End),
        _ => None,
    }
}

/// A from-scratch key scan, independent of `Level::open`'s own index-building
/// scan, so the position-index checks below are cross-checked against
/// something other than the code under test.
fn scan_positions_independently(db: &Database) -> HashMap<Dimension, HashSet<ChunkPosition>> {
    let mut out: HashMap<Dimension, HashSet<ChunkPosition>> = HashMap::new();
    let mut keys = db.keys().expect("failed to iterate keys");
    for kv in &mut keys {
        let mut cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut cursor) else {
            continue;
        };
        if !key.data.is_chunk_scoped() {
            continue;
        }
        out.entry(key.dimension).or_default().insert(key.chunk);
    }
    out
}

/// Every fixture opens through `Level::open`, and the dimensions it reports
/// are a superset of what `index.json`'s independently recorded dimension
/// list names (a superset, not an exact match, because `Level::dimensions`
/// also unions in dimensions the metadata dictionary names -- see its doc
/// comment -- which `index.json`'s `dimensions` field, built from a plain
/// key scan, does not account for).
#[test]
fn every_fixture_opens_and_dimensions_match_index() {
    let fixtures = support::fixtures();
    assert!(!fixtures.is_empty());

    for fixture in fixtures {
        let (_tmp, level) = open_fixture_level(&fixture);

        let expected: HashSet<Dimension> = fixture
            .dimensions
            .iter()
            .filter_map(|d| dimension_from_fixture_name(d))
            .collect();
        let actual: HashSet<Dimension> = level.dimensions().iter().copied().collect();

        assert!(
            !actual.is_empty(),
            "{}: Level::dimensions reported nothing",
            fixture.name
        );
        assert!(
            expected.is_subset(&actual),
            "{}: index dimensions {expected:?} not all present in Level::dimensions {actual:?}",
            fixture.name
        );
    }
}

/// `Level::chunk_positions` matches an independent key scan exactly, for
/// every dimension of every fixture -- not just non-empty, but the same
/// set, guarding against a vacuous pass where both sides happened to be
/// empty for the same wrong reason.
#[test]
fn chunk_positions_match_an_independent_scan() {
    let mut total_checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);
        let independent = scan_positions_independently(level.raw());

        for &dimension in level.dimensions() {
            let from_level: HashSet<ChunkPosition> = level.chunk_positions(dimension).collect();
            let from_scan = independent.get(&dimension).cloned().unwrap_or_default();
            assert_eq!(
                from_level, from_scan,
                "{}: dimension {dimension:?} chunk-position mismatch against an independent scan",
                fixture.name
            );
            total_checked += from_level.len();
        }
    }

    assert!(
        total_checked > 0,
        "no chunk positions were checked across any fixture"
    );
}

/// A bounded sample of each dimension's chunk positions decodes without
/// error, checked through the bulk iterator (`Level::chunks`) and
/// spot-checked once per dimension through the id-parameter API
/// (`Level::chunk`).
///
/// Bounded rather than exhaustive over every position in every fixture, and
/// derived from a *single* `Level::chunks` iteration rather than looping
/// `Level::chunk` per position, for a real cost reason worth recording:
/// `Level::chunk` costs one full database scan *per call* (documented on
/// `ChunkRecords::read`, which it delegates to), so calling it once per
/// sampled position -- the first shape this test had -- repeats that scan
/// 20 times per dimension for no benefit `Level::chunks`' own single scan
/// doesn't already give. `Level::chunks(dim).take(N)` pays for one scan (the
/// same one `Level::chunk` would pay per call) and then decodes only the
/// first `N` of the groups that scan already materialized, which is what
/// keeps this bounded rather than exhaustive without reintroducing the
/// O(sample size) scan cost. [`chunk_positions_match_an_independent_scan`]
/// already establishes that every position the index reports genuinely
/// exists on disk; what this test adds is that decoding one actually
/// succeeds.
#[test]
fn every_chunk_decodes_without_error() {
    const SAMPLE_PER_DIMENSION: usize = 20;
    let mut total_decoded = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);

        for &dimension in level.dimensions() {
            let has_positions = level.chunk_positions(dimension).next().is_some();

            let sample: Vec<(ChunkPosition, Chunk)> = level
                .chunks(dimension)
                .unwrap_or_else(|e| panic!("{}: chunks({dimension:?}) failed: {e}", fixture.name))
                .take(SAMPLE_PER_DIMENSION)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|e| {
                    panic!(
                        "{}: chunks({dimension:?}) yielded an error: {e}",
                        fixture.name
                    )
                });

            assert!(
                !has_positions || !sample.is_empty(),
                "{}: chunks({dimension:?}) yielded nothing where chunk_positions had entries",
                fixture.name
            );
            for (position, chunk) in &sample {
                assert_eq!(chunk.position(), *position);
                assert_eq!(chunk.dimension(), dimension);
                assert!(
                    level.chunk_positions(dimension).any(|p| p == *position),
                    "{}: chunks({dimension:?}) yielded {position:?}, which chunk_positions does not know about",
                    fixture.name
                );
            }

            // Exercise the id-parameter API too, but only once per
            // dimension: unlike the bulk path above, each call here costs
            // its own full scan (see this test's doc comment), so doing it
            // for the whole sample would reintroduce exactly the cost this
            // test is now structured to avoid.
            if let Some((position, _)) = sample.first() {
                let chunk = level
                    .chunk(dimension, *position)
                    .unwrap_or_else(|e| {
                        panic!(
                            "{}: chunk({dimension:?}, {position:?}) errored: {e}",
                            fixture.name
                        )
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: chunk({dimension:?}, {position:?}) unexpectedly absent",
                            fixture.name
                        )
                    });
                assert_eq!(chunk.position(), *position);
                assert_eq!(chunk.dimension(), dimension);
            }

            total_decoded += sample.len();
        }
    }

    assert!(total_decoded > 0, "decoded zero chunks across all fixtures");
}

/// Reading a chunk, encoding it unmodified, and writing it into a *fresh*
/// database reproduces every record byte for byte -- the decision-7
/// guarantee at the level layer. Checked for one chunk per fixture (the
/// first position in its first dimension), across every fixture, rather
/// than every chunk, since the per-record round trip this exercises is
/// separately pinned exhaustively for `SubChunk`, `HeightMap`, and
/// `LegacyTerrain` (each carries its own byte-identity tests) and for
/// `Biomes3d` (`tests/biome3d.rs`, added alongside this one after this test
/// caught a real gap: `Biomes3d`'s prior only fixture-driven check compared
/// decode->encode->decode, which cannot detect a header bit with no effect
/// on the decoded value -- see `Biomes3d::to_disk`'s doc comment); this
/// test is about the level layer's own plumbing (`Chunk::decode` and
/// `Chunk::encode`), not re-proving format decode correctness on its own.
///
/// Deliberately calls [`Chunk::encode`] and [`ChunkRecords::write`]
/// directly rather than [`bedrock_level::level::Level::put_chunk`]: this
/// test's fresh database has no chunk at `position` at all, so it is
/// exactly the same-coordinates fast path `put_chunk` would already take
/// (see its own doc comment), and going around it keeps this test's
/// coverage exact rather than incidentally depending on `put_chunk`'s
/// coordinate-matching logic too. [`put_chunk_can_introduce_a_brand_new_chunk`]
/// and [`put_chunk_moves_a_chunk_to_a_different_occupied_position`] cover
/// `put_chunk` itself, including the one case this test's direct-write
/// shape cannot exercise (clearing a *different* position's stale
/// records).
#[test]
fn read_encode_write_round_trip_preserves_records_byte_for_byte() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);

        let Some(&dimension) = level.dimensions().first() else {
            continue;
        };
        let Some(position) = level.chunk_positions(dimension).next() else {
            continue;
        };

        let original = ChunkRecords::read(level.raw(), position, dimension)
            .unwrap()
            .expect("position came from the level's own index");
        let chunk = level
            .chunk(dimension, position)
            .unwrap()
            .expect("position came from the level's own index");

        let fresh_dir = tempfile::tempdir().unwrap();
        let fresh_db_path = fresh_dir.path().join("db");
        let fresh_db = Database::open(fresh_db_path.to_str().unwrap()).unwrap();

        chunk
            .encode()
            .unwrap_or_else(|e| panic!("{}: Chunk::encode failed: {e}", fixture.name))
            .write(&fresh_db)
            .unwrap_or_else(|e| panic!("{}: writing the encoded chunk failed: {e}", fixture.name));

        let round_tripped = ChunkRecords::read(&fresh_db, position, dimension)
            .unwrap()
            .unwrap_or_else(|| panic!("{}: round-tripped chunk is missing entirely", fixture.name));

        let original_map: HashMap<_, Vec<u8>> = original
            .iter()
            .map(|(key, bytes)| (key.data, bytes.to_vec()))
            .collect();
        let round_tripped_map: HashMap<_, Vec<u8>> = round_tripped
            .iter()
            .map(|(key, bytes)| (key.data, bytes.to_vec()))
            .collect();

        if original_map != round_tripped_map {
            let mut problems = Vec::new();
            for (key, bytes) in &original_map {
                match round_tripped_map.get(key) {
                    None => problems.push(format!("{key:?}: present originally, missing after round trip")),
                    // `Biome3d` is the one documented exception to strict byte
                    // identity -- see `tests/biome3d.rs`'s
                    // `decodes_and_round_trips_every_record` for the full
                    // account (a packed-word padding-bit finding, not a bug):
                    // a mismatch here is only a problem if the two byte
                    // strings decode to different values.
                    Some(rt) if rt != bytes && *key == KeyVariant::Biome3d => {
                        let original_decoded =
                            Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(bytes.as_slice()));
                        let round_tripped_decoded =
                            Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(rt.as_slice()));
                        if !matches!(
                            (&original_decoded, &round_tripped_decoded),
                            (Ok(a), Ok(b)) if a == b
                        ) {
                            problems.push(format!(
                                "{key:?}: byte mismatch that also decodes differently (or fails to decode)"
                            ));
                        }
                    }
                    Some(rt) if rt != bytes => problems.push(format!(
                        "{key:?}: {} bytes originally, {} bytes after round trip (first differing byte at {:?})",
                        bytes.len(),
                        rt.len(),
                        bytes.iter().zip(rt.iter()).position(|(a, b)| a != b)
                    )),
                    _ => {}
                }
            }
            for key in round_tripped_map.keys() {
                if !original_map.contains_key(key) {
                    problems.push(format!(
                        "{key:?}: absent originally, present after round trip"
                    ));
                }
            }
            assert!(
                problems.is_empty(),
                "{}: chunk {position:?}/{dimension:?} did not round-trip byte-identically through a fresh database:\n{}",
                fixture.name,
                problems.join("\n")
            );
        }
        checked += 1;
    }

    assert!(checked > 0, "round-trip check ran against zero fixtures");
}

/// `Chunk::block`'s whole-column position math resolves to the same entry
/// as scanning the owning subchunk's layer 0 directly -- a known block at a
/// known (derived, not hardcoded) position, across several populated
/// subchunks so the check is not vacuous.
#[test]
fn block_accessor_matches_a_direct_subchunk_scan() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let (_tmp, level) = open_fixture_level(&fixture);
    let dimension = Dimension::Overworld;

    let mut checked = 0usize;
    'positions: for position in level.chunk_positions(dimension) {
        let chunk = level.chunk(dimension, position).unwrap().unwrap();
        let Terrain::SubChunks(subchunks) = chunk.terrain() else {
            continue;
        };
        for (&index, subchunk) in subchunks {
            let Some(layer) = subchunk.get_layer(0) else {
                continue;
            };
            if layer.is_empty() {
                continue;
            }
            let expected = layer.get(BlockPosition(0, 0, 0));
            let y = i32::from(index) * 16;
            let actual = chunk.block(ChunkBlockPosition::new(0, y, 0));
            assert_eq!(
                actual, expected,
                "{}: chunk {position:?} block() disagreed with a direct layer scan at y={y}",
                fixture.name
            );
            checked += 1;
            if checked >= 5 {
                break 'positions;
            }
        }
    }

    assert!(
        checked > 0,
        "no populated subchunk layer found to spot-check block() against"
    );
}

/// `v1_20_81` is a synthetically generated integration-test world whose
/// Overworld and Nether both carry subchunks outside any range real
/// gameplay could produce -- see `tests/height_range.rs`'s
/// `known_observed_exceeds_nominal` for the full account (independently
/// re-derived there from a direct key scan). `Level::bounds` resolves
/// through the same `height_range::resolve` this crate's other height-range
/// tests already pin, so it inherits the same documented exception rather
/// than this test rediscovering it as a failure.
fn known_observed_exceeds_bounds(fixture_name: &str, dimension: Dimension) -> bool {
    matches!(
        (fixture_name, dimension),
        ("v1_20_81", Dimension::Overworld | Dimension::Nether)
    )
}

/// `Level::bounds` resolves to a range containing every subchunk the
/// fixture's index actually observed on disk, for every dimension the
/// index recorded a range for -- except the one documented exception (see
/// [`known_observed_exceeds_bounds`]), where it must *not* contain it, so a
/// fix that quietly widened the exception away would also be caught here.
#[test]
fn bounds_contains_the_recorded_subchunk_range() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);

        for (dim_name, [min, max]) in &fixture.subchunk_height_range {
            let Some(dimension) = dimension_from_fixture_name(dim_name) else {
                continue;
            };
            let bounds = level
                .bounds(dimension)
                .unwrap_or_else(|| panic!("{}: bounds({dimension:?}) is None", fixture.name));
            let contains = bounds.min <= *min && *max <= bounds.max;

            if known_observed_exceeds_bounds(&fixture.name, dimension) {
                assert!(
                    !contains,
                    "{}: {dimension:?} observed range no longer exceeds resolved bounds -- \
                     update known_observed_exceeds_bounds",
                    fixture.name
                );
            } else {
                assert!(
                    contains,
                    "{}: resolved bounds {}..{} do not contain the observed subchunk range {min}..{max} for {dim_name}",
                    fixture.name, bounds.min, bounds.max
                );
            }
            checked += 1;
        }
    }

    assert!(checked > 0, "bounds check ran against zero dimensions");
}

/// `DimensionRef` is a pure convenience over the id-parameter API: every
/// call it offers must agree exactly with the matching `Level` method.
#[test]
fn dimension_ref_matches_the_id_parameter_api() {
    let mut checked = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, level) = open_fixture_level(&fixture);

        for &dimension in level.dimensions() {
            let view = level.dimension(dimension);
            assert_eq!(view.dimension(), dimension);
            assert_eq!(view.bounds(), level.bounds(dimension));

            let via_ref: Vec<ChunkPosition> = view.chunk_positions().collect();
            let via_id: Vec<ChunkPosition> = level.chunk_positions(dimension).collect();
            assert_eq!(
                via_ref, via_id,
                "{}: DimensionRef::chunk_positions diverges from Level::chunk_positions for {dimension:?}",
                fixture.name
            );

            if let Some(position) = via_id.first().copied() {
                let via_ref_chunk = view.chunk(position).unwrap();
                let via_id_chunk = level.chunk(dimension, position).unwrap();
                assert_eq!(via_ref_chunk.is_some(), via_id_chunk.is_some());
                assert_eq!(
                    via_ref_chunk.as_ref().map(Chunk::position),
                    via_id_chunk.as_ref().map(Chunk::position),
                );
            }
            checked += 1;
        }
    }

    assert!(
        checked > 0,
        "DimensionRef parity check ran against zero dimensions"
    );
}

/// A chunk that never existed on disk (`Chunk::new`) still encodes to a
/// well-formed record group -- at minimum a version stamp -- and
/// `Level::put_chunk` can place it at a position the level had nothing at
/// before, updating the position index.
#[test]
fn put_chunk_can_introduce_a_brand_new_chunk() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let mut level = Level::open_rw(tmp.path().join("world")).unwrap();

    let dimension = Dimension::Overworld;
    let new_position = ChunkPosition(100_000, 100_000);
    assert!(level.chunk(dimension, new_position).unwrap().is_none());
    assert!(!level.chunk_positions(dimension).any(|p| p == new_position));

    let chunk = Chunk::new(new_position, dimension, level.write_version());
    let _report = level.put_chunk(dimension, new_position, &chunk).unwrap();

    assert!(level.chunk_positions(dimension).any(|p| p == new_position));
    let read_back = level.chunk(dimension, new_position).unwrap();
    assert!(read_back.is_some());
}

/// `Level::put_chunk` writing a chunk to a position *different* from its
/// own, where that target position already holds different real content,
/// clears every record the target had that the moved chunk's encode does
/// not cover and leaves the target byte-exact to what a direct
/// `chunk.encode_at(target, dim)` would produce. This is the cross-position
/// path that `read_encode_write_round_trip_preserves_records_byte_for_byte`
/// cannot exercise (its fresh database has nothing at the target to begin
/// with, so there is no stale record to prove got cleared) -- see
/// `Level::put_chunk`'s doc comment for the one case this guarantee does
/// not cover (a chunk from a *different* `Level` moved to coordinates this
/// level already occupies).
#[test]
fn put_chunk_moves_a_chunk_to_a_different_occupied_position() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_updated_superflat")
        .expect("v1_18_updated_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let mut level = Level::open_rw(tmp.path().join("world")).unwrap();
    let dimension = Dimension::Overworld;

    let positions: Vec<ChunkPosition> = level.chunk_positions(dimension).collect();
    assert!(
        positions.len() >= 2,
        "{}: need at least two real chunks to exercise a cross-position move",
        fixture.name
    );
    let source_position = positions[0];
    let source_chunk = level.chunk(dimension, source_position).unwrap().unwrap();

    // Search for a target among the real positions whose on-disk key set
    // actually differs from what the moved chunk would leave there --
    // without this, "stale records get cleared" has nothing to prove.
    let mut chosen: Option<(ChunkPosition, ChunkRecords, HashMap<KeyVariant, Vec<u8>>)> = None;
    for &candidate in &positions[1..] {
        let expected = source_chunk.encode_at(candidate, dimension).unwrap();
        let expected_map: HashMap<KeyVariant, Vec<u8>> = expected
            .iter()
            .map(|(k, bytes)| (k.data, bytes.to_vec()))
            .collect();
        let existing = ChunkRecords::read(level.raw(), candidate, dimension)
            .unwrap()
            .expect("candidate came from the level's own index");
        let existing_map: HashMap<KeyVariant, Vec<u8>> = existing
            .iter()
            .map(|(k, bytes)| (k.data, bytes.to_vec()))
            .collect();
        if existing_map != expected_map {
            chosen = Some((candidate, existing, expected_map));
            break;
        }
    }
    let (target_position, target_before, expected_after_map) = chosen.unwrap_or_else(|| {
        panic!(
            "{}: no target among {} candidate positions differs from the moved chunk -- \
             fixture lacks the variety this test needs",
            fixture.name,
            positions.len() - 1
        )
    });
    let target_before_keys: HashSet<KeyVariant> =
        target_before.iter().map(|(k, _)| k.data).collect();
    let expected_after_keys: HashSet<KeyVariant> = expected_after_map.keys().copied().collect();

    // Pinned at the chunk's own version via `put_chunk_at`: this test is about
    // stale-record clearing on a cross-position move, not write-target retargeting, and
    // `expected_after_map` above was built with `encode_at` (no retarget) -- `put_chunk`'s
    // own default (the level's write-target version) would diverge from that on purpose.
    let _report = level
        .put_chunk_at(
            dimension,
            target_position,
            &source_chunk,
            source_chunk.chunk_version(),
        )
        .unwrap();

    let actual_after = ChunkRecords::read(level.raw(), target_position, dimension)
        .unwrap()
        .expect("target position still exists after the move");
    let actual_after_map: HashMap<KeyVariant, Vec<u8>> = actual_after
        .iter()
        .map(|(k, bytes)| (k.data, bytes.to_vec()))
        .collect();

    assert_eq!(
        actual_after_map, expected_after_map,
        "{}: put_chunk's result at the target position does not match a direct encode_at",
        fixture.name
    );

    // Every key the target held before that the moved chunk doesn't cover
    // must be gone entirely, not merely left with different bytes.
    for stale_key in target_before_keys.difference(&expected_after_keys) {
        assert!(
            !actual_after_map.contains_key(stale_key),
            "{}: stale record {stale_key:?} survived the move to {target_position:?}",
            fixture.name
        );
    }

    // The source position is untouched by a move to a different one.
    let source_still_there = level.chunk(dimension, source_position).unwrap();
    assert_eq!(
        source_still_there.map(|c| c.position()),
        Some(source_position)
    );
}

/// [`Level::save`] on an `open_rw` handle rewrites `level.dat` (decodable
/// strictly, carrying the new name) and `levelname.txt` (exactly the new
/// name, plain text, nothing else) after [`Level::set_name`] -- the one
/// mutation `save` currently makes meaningful, per `Level::save`'s doc
/// comment.
#[test]
fn save_writes_level_dat_and_levelname_txt() {
    use bedrock_level::settings::LevelSettings;

    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let world_path = tmp.path().join("world");

    let mut level = Level::open_rw(&world_path).unwrap();
    assert_ne!(level.name(), "a renamed test world");
    level.set_name("a renamed test world");
    level.save().unwrap_or_else(|e| panic!("save failed: {e}"));

    let level_dat = std::fs::read(world_path.join("level.dat")).expect("level.dat written");
    let settings = LevelSettings::read(level_dat.as_slice())
        .expect("freshly saved level.dat decodes strictly");
    assert_eq!(settings.level_name, "a renamed test world");

    let levelname =
        std::fs::read_to_string(world_path.join("levelname.txt")).expect("levelname.txt written");
    assert_eq!(levelname, "a renamed test world");

    // Reopening through `Level::open` sees the rename too -- `save`
    // actually reached disk, not just this handle's in-memory settings.
    // `rusty-leveldb` holds an exclusive lock on `db/` for as long as any
    // handle on it is alive (see `extract_fixture`'s doc comment), so the
    // first handle must be dropped before a second one can open the same
    // store.
    drop(level);
    let reopened = Level::open(&world_path).unwrap();
    assert_eq!(reopened.name(), "a renamed test world");
}

/// [`Level::save`] on an `open` (read-only) handle returns an error rather
/// than silently doing nothing, and leaves `level.dat` untouched -- the
/// read-only decision documented on [`Level::open`] and [`Level::save`].
#[test]
fn save_on_read_only_handle_errors_and_does_not_write() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let world_path = tmp.path().join("world");

    let before = std::fs::read(world_path.join("level.dat")).unwrap();
    let levelname_existed_before = world_path.join("levelname.txt").exists();

    let mut level = Level::open(&world_path).unwrap();
    level.set_name("should never reach disk");
    let result = level.save();
    assert!(result.is_err(), "save on a read-only handle should error");

    let after = std::fs::read(world_path.join("level.dat")).unwrap();
    assert_eq!(before, after, "read-only save must not modify level.dat");
    assert_eq!(
        world_path.join("levelname.txt").exists(),
        levelname_existed_before,
        "read-only save must not create levelname.txt"
    );
}

/// [`Level::put_chunk`] on an `open` (read-only) handle returns an error
/// rather than writing straight through to `db/` -- the guard extended
/// alongside [`Level::save`]'s (see [`Level::open`]'s doc comment).
#[test]
fn put_chunk_on_read_only_handle_errors_and_does_not_write() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let world_path = tmp.path().join("world");

    let mut level = Level::open(&world_path).unwrap();
    let dimension = Dimension::Overworld;
    let new_position = ChunkPosition(100_000, 100_000);
    assert!(level.chunk(dimension, new_position).unwrap().is_none());

    let chunk = Chunk::new(new_position, dimension, level.write_version());
    let result = level.put_chunk(dimension, new_position, &chunk);
    assert!(
        result.is_err(),
        "put_chunk on a read-only handle should error"
    );

    assert!(
        level.chunk(dimension, new_position).unwrap().is_none(),
        "read-only put_chunk must not write the chunk"
    );
    assert!(
        !level.chunk_positions(dimension).any(|p| p == new_position),
        "read-only put_chunk must not update the position index"
    );
}

/// [`Level::remove_chunk`] on an `open` (read-only) handle returns an error
/// rather than deleting anything from `db/`.
#[test]
fn remove_chunk_on_read_only_handle_errors_and_does_not_remove() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let world_path = tmp.path().join("world");

    let mut level = Level::open(&world_path).unwrap();
    let dimension = Dimension::Overworld;
    let position = level
        .chunk_positions(dimension)
        .next()
        .expect("v1_18_superflat has at least one overworld chunk");
    assert!(level.chunk(dimension, position).unwrap().is_some());

    let result = level.remove_chunk(dimension, position);
    assert!(
        result.is_err(),
        "remove_chunk on a read-only handle should error"
    );

    assert!(
        level.chunk(dimension, position).unwrap().is_some(),
        "read-only remove_chunk must not delete the chunk"
    );
    assert!(
        level.chunk_positions(dimension).any(|p| p == position),
        "read-only remove_chunk must not update the position index"
    );
}

/// [`Level::chunks`]' per-item error names the chunk that failed:
/// [`bedrock_level::error::ChunkError`] carries the position and dimension
/// alongside the underlying decode error. Fixtures carry no malformed
/// chunks (they are all real, game-written data), so this synthesizes one
/// directly: a `0x30` `LegacyTerrain` record too short to hold the established
/// 49,152-byte column, which `Chunk::decode` cannot decode at all --
/// wired in through a raw `db/` write and a fresh position-index scan
/// (`Level::open`), rather than through any public chunk-writing API.
#[test]
fn chunks_bulk_error_names_the_failing_chunk_position_and_dimension() {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");
    let tmp = extract_fixture(&fixture);
    let world_path = tmp.path().join("world");

    let dimension = Dimension::Overworld;
    let bad_position = ChunkPosition(424_242, 424_242);

    {
        let level = Level::open_rw(&world_path).unwrap();
        assert!(
            level.chunk(dimension, bad_position).unwrap().is_none(),
            "bad_position must not collide with a real chunk from the fixture"
        );

        let bad_key = Key {
            chunk: bad_position,
            dimension,
            data: KeyVariant::LegacyTerrain,
        };
        let mut key_bytes = Vec::new();
        bad_key.serialize(&mut key_bytes).unwrap();
        // Far short of `legacy_terrain::ESTABLISHED_LEN`: `LegacyTerrain::from_disk`
        // errors reading the id array before it gets anywhere near the tail.
        level.raw().insert(&key_bytes, [0u8; 4]).unwrap();
        // `rusty-leveldb` holds an exclusive lock on `db/` for as long as any
        // handle is alive (see `extract_fixture`'s doc comment); the position
        // index is only built at open time, so this handle must be dropped and
        // a fresh one opened for the new key to be picked up by the scan.
    }

    let level = Level::open(&world_path).unwrap();
    assert!(
        level.chunk_positions(dimension).any(|p| p == bad_position),
        "the synthesized malformed record should surface through the position index like any other"
    );

    let results: Vec<_> = level.chunks(dimension).unwrap().collect();
    let mut ok_count = 0;
    let mut bad_error = None;
    for result in results {
        match result {
            Ok((position, _)) => {
                assert_ne!(
                    position, bad_position,
                    "the malformed record must not decode successfully"
                );
                ok_count += 1;
            }
            Err(err) => {
                assert!(
                    bad_error.is_none(),
                    "only the one synthesized chunk should fail to decode"
                );
                bad_error = Some(err);
            }
        }
    }

    assert!(
        ok_count > 0,
        "the fixture's real chunks should still decode"
    );
    let err = bad_error.expect("the malformed chunk should surface as an Err");
    assert_eq!(err.position, bad_position);
    assert_eq!(err.dimension, dimension);
    let message = err.to_string();
    assert!(
        message.contains("424242"),
        "ChunkError's Display should name the failing position: {message}"
    );
}

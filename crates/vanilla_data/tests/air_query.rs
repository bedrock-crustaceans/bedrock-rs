//! Corpus survey and behavior tests for `ContentProvider::is_air` /
//! `VanillaContent`'s implementation of it.
//!
//! The survey (`v1_12_legacy_palette_air_survey`) scans every real
//! [`PaletteEntry::Legacy`] entry in the corpus's one pre-flattening fixture (`v1_12`) for
//! what air actually looks like there, since `VanillaContent::is_air` can only classify a
//! legacy/numeric entry as far as the corpus establishes -- see `is_air`'s doc comment on
//! `bedrock_level::provider::ContentProvider` and `VanillaContent`'s implementation for
//! what this survey found and why the legacy/numeric arms both answer `None`.

use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::provider::ContentProvider;
use bedrock_level::subchunk::{PaletteEntry, SubChunk};
use bedrock_vanilla_data::VanillaContent;

/// Extracts `tests/fixtures/<name>.tar.gz` (from the sibling `bedrock_level` crate --
/// `bedrock_vanilla_data` has no fixture set of its own) into a fresh `TempDir` and opens
/// its database. Mirrors `bedrock_level`'s own `tests/support/mod.rs::Fixture::open`,
/// duplicated rather than shared because that module is not a public part of
/// `bedrock_level`'s API and this crate has no dependency that would let it reach into
/// another crate's `tests/` directory.
fn open_fixture_db(name: &str) -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let tar_gz_path = format!("../level/tests/fixtures/{name}.tar.gz");
    let tar_gz = std::fs::File::open(&tar_gz_path).unwrap_or_else(|e| panic!("{tar_gz_path}: {e}"));
    let tar = flate2::read::GzDecoder::new(tar_gz);
    tar::Archive::new(tar)
        .unpack(tmp.path())
        .unwrap_or_else(|e| panic!("{tar_gz_path}: failed to unpack: {e}"));

    let db_path = tmp.path().join("world/db");
    let db = Database::open(db_path.to_str().unwrap())
        .unwrap_or_else(|e| panic!("{name}: failed to open database: {e}"));
    (tmp, db)
}

/// Surveys every real [`PaletteEntry::Legacy`] entry in `v1_12` (the corpus's one
/// pre-flattening, non-paletted-content fixture -- see `crates/level/TODO.md`'s note that
/// a fixture flagged `paletted` can still predate flattening) for what air looks like on
/// disk there: which `(name, val)` pairs actually occur, and specifically whether
/// `"minecraft:air"` (or a legacy `"air"`-family name) appears, and at which `val`s.
///
/// This is the evidence `VanillaContent::is_air`'s legacy arm is checked against. Finding:
/// `v1_12` is the corpus's *only* pre-flattening fixture, so any air-name/val pair found
/// here is single-sourced -- true of the whole legacy palette table already (see
/// `VanillaContent`'s crate-root doc comment: there is no official dump of the legacy
/// table at any version) -- and `VanillaContent::is_air` therefore answers `None` for
/// every `Legacy`/`Numeric` entry rather than hardcode a fact this crate cannot check
/// against a second source. The counts below are printed, not asserted into a pass/fail
/// shape tied to a specific number, so this test documents the finding without pinning a
/// magic constant a future corpus change would silently falsify.
#[test]
fn v1_12_legacy_palette_air_survey() {
    let (_tmp, db) = open_fixture_db("v1_12");
    let mut keys = db.keys().unwrap();

    let mut total_legacy_entries: usize = 0;
    let mut total_numeric_entries: usize = 0;
    let mut total_modern_entries: usize = 0;
    let mut air_name_vals: std::collections::BTreeSet<(String, i16)> =
        std::collections::BTreeSet::new();
    let mut distinct_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for kv in &mut keys {
        let mut key_cursor = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut key_cursor) else {
            continue;
        };
        if !matches!(key.data, KeyVariant::SubChunk { .. }) {
            continue;
        }

        let bytes: Vec<u8> = kv.value().into();
        let Ok(subchunk) = SubChunk::from_disk_greedy(&mut Cursor::new(bytes.as_slice())) else {
            continue;
        };

        let mut layer_index = 0;
        while let Some(layer) = subchunk.get_layer(layer_index) {
            for entry in layer.palette() {
                match entry {
                    PaletteEntry::Legacy { name, val } => {
                        total_legacy_entries += 1;
                        distinct_names.insert(name.clone());
                        if name.contains("air") && !name.contains("stairs") {
                            air_name_vals.insert((name.clone(), *val));
                        }
                    }
                    PaletteEntry::Numeric { .. } => total_numeric_entries += 1,
                    PaletteEntry::Modern(_) => total_modern_entries += 1,
                }
            }
            layer_index += 1;
        }
    }

    println!(
        "v1_12 palette entry totals: legacy={total_legacy_entries} numeric={total_numeric_entries} \
         modern={total_modern_entries}"
    );
    println!("v1_12 distinct legacy names: {}", distinct_names.len());
    println!("v1_12 air-named legacy (name, val) pairs: {air_name_vals:?}");

    assert!(
        total_legacy_entries > 0,
        "expected at least one Legacy palette entry in v1_12"
    );
}

#[test]
fn vanilla_content_classifies_modern_air_by_name() {
    let provider = VanillaContent;
    let air = bedrock_level::subchunk::BlockDef {
        name: "minecraft:air".to_string(),
        version: None,
        states: Default::default(),
    };
    let stone = bedrock_level::subchunk::BlockDef {
        name: "minecraft:stone".to_string(),
        version: None,
        states: Default::default(),
    };

    assert_eq!(
        provider.is_air(&PaletteEntry::Modern(air)),
        Some(true),
        "minecraft:air is the fixed vanilla air block name at every pinned generated tag \
         (see this file's module docs)"
    );
    assert_eq!(provider.is_air(&PaletteEntry::Modern(stone)), Some(false));
}

#[test]
fn vanilla_content_declines_to_classify_legacy_and_numeric_entries() {
    let provider = VanillaContent;

    assert_eq!(
        provider.is_air(&PaletteEntry::Legacy {
            name: "minecraft:air".to_string(),
            val: 0,
        }),
        None,
        "the legacy table has no second source to check an air-name/val guess against \
         (see this file's survey test), so this arm honestly reports \"cannot classify\""
    );
    assert_eq!(
        provider.is_air(&PaletteEntry::Numeric { id: 0, data: 0 }),
        None
    );
}

/// Report-only survey (no code in `level_chunk.rs`; see the Phase 5 task brief) comparing
/// candidate heightmap definitions against each fixture's actually-stored heightmap,
/// column by column, for three `biomes_3d` fixtures and one `biomes_2d` fixture. Prints
/// match percentages for each candidate definition so a later wave can pick one on
/// evidence rather than a guess. Uses `PaletteEntry::name() == Some("minecraft:air")`
/// directly (a format-level name comparison, not `ContentProvider::is_air`) since this is
/// a one-off report over Modern (post-flattening) terrain, not a production classification
/// path -- the fixtures surveyed here (`v1_18_30`, `v1_19_30`, `v1_20_81` for 3D;
/// `v1_17_40_superflat` for 2D) are all post-flattening, so every block layer entry
/// visited is `PaletteEntry::Modern`.
///
/// This survey's own "highest non-air +1" definition is a deliberately naive baseline,
/// not a claim about what the game actually computes -- the low match rates below
/// (asserted, not just printed, so nobody later mistakes it for the real definition) are
/// the point. One read-only reference implementation consulted while building this uses
/// "highest block that is non-air **and** (lit or a slab)" -- a light-propagation and
/// block-shape heuristic, not a pure air/non-air classification. That definition needs
/// per-block light data and slab/half-block state, neither of which
/// `ContentProvider::is_air` (or any hook this task added) can supply: `is_air` answers
/// one yes/no/unknown question about a palette entry, with no notion of light level or
/// block shape. A future heightmap recompute built on `is_air` alone is therefore not
/// capable of reproducing this reference definition; it would need a materially richer
/// provider query first. Stated here as provenance for the next wave, not attributed to
/// the reference project by name (see this repo's convention on citing references in
/// code/tests).
#[test]
fn heightmap_definition_survey() {
    use bedrock_level::level::Level;
    use bedrock_level::level_chunk::{ChunkBlockPosition, Terrain};
    use bedrock_level::subchunk::PaletteEntry;

    fn is_air_name(entry: &PaletteEntry) -> Option<bool> {
        entry.name().map(|n| n == "minecraft:air")
    }

    /// Highest absolute block Y with a non-air block-layer-0 entry, plus one, scanning
    /// top-down; `None` if the column has no resolvable non-air block at all (every
    /// column visited is air, or every entry present is unresolvable).
    fn highest_non_air_plus_one(
        chunk: &bedrock_level::level_chunk::Chunk,
        x: u8,
        z: u8,
    ) -> Option<i32> {
        let Terrain::SubChunks(subchunks) = chunk.terrain() else {
            return None;
        };
        for (&index, _) in subchunks.iter().rev() {
            for local_y in (0..16i32).rev() {
                let y = i32::from(index) * 16 + local_y;
                let Some(entry) = chunk.block(ChunkBlockPosition::new(x, y, z)) else {
                    continue;
                };
                match is_air_name(entry) {
                    Some(false) => return Some(y + 1),
                    Some(true) => continue,
                    None => continue,
                }
            }
        }
        None
    }

    let fixture_names = ["v1_18_30", "v1_19_30", "v1_20_81", "v1_17_40_superflat"];

    for fixture in fixture_names {
        let tmp = tempfile::tempdir().unwrap();
        let tar_gz_path = format!("../level/tests/fixtures/{fixture}.tar.gz");
        let tar_gz = std::fs::File::open(&tar_gz_path).unwrap();
        let tar = flate2::read::GzDecoder::new(tar_gz);
        tar::Archive::new(tar).unpack(tmp.path()).unwrap();

        let level = Level::open(tmp.path().join("world")).unwrap();

        // Picks the Overworld out of `level.dimensions()` by its `Debug` text rather than
        // naming `bedrock_shared::world::dimension::Dimension` directly -- this crate has
        // no direct dependency on `bedrock_shared` (only `bedrock_level` does), so the
        // type name itself is not nameable here; relying on `Level::dimensions()`'s
        // already-public return type and inference avoids adding one just for this
        // report-only survey.
        let dimension = *level
            .dimensions()
            .iter()
            .find(|d| format!("{d:?}") == "Overworld")
            .expect("expected an Overworld dimension in every fixture");

        let mut columns_checked = 0usize;
        let mut match_exact = 0usize;
        let mut match_minus_min_y = 0usize;
        let mut unresolvable = 0usize;

        let min_y = level.bounds(dimension).map_or(0, |r| r.min);

        let mut chunks_visited = 0usize;
        for position in level.chunk_positions(dimension) {
            if chunks_visited >= 20 {
                break;
            }
            let Ok(Some(chunk)) = level.chunk(dimension, position) else {
                continue;
            };
            let Some(stored) = chunk.heightmap() else {
                continue;
            };
            chunks_visited += 1;

            for z in 0..16u8 {
                for x in 0..16u8 {
                    columns_checked += 1;
                    let idx = bedrock_level::biome::column_index(x as usize, z as usize).unwrap();
                    let stored_h = stored[idx];

                    match highest_non_air_plus_one(&chunk, x, z) {
                        Some(candidate) => {
                            if candidate == stored_h {
                                match_exact += 1;
                            }
                            if candidate - min_y == stored_h {
                                match_minus_min_y += 1;
                            }
                        }
                        None => unresolvable += 1,
                    }
                }
            }
        }

        let pct = |n: usize| {
            if columns_checked == 0 {
                0.0
            } else {
                100.0 * n as f64 / columns_checked as f64
            }
        };
        println!(
            "{fixture}: columns_checked={columns_checked} chunks_visited={chunks_visited} \
             unresolvable={unresolvable} ({:.1}%) \
             \"highest non-air +1, absolute\" match={match_exact} ({:.1}%) \
             \"highest non-air +1, minus dimension min_y\" match={match_minus_min_y} ({:.1}%)",
            pct(unresolvable),
            pct(match_exact),
            pct(match_minus_min_y),
        );

        assert!(columns_checked > 0, "{fixture}: no columns were checked");
        assert_eq!(
            unresolvable, 0,
            "{fixture}: every column should resolve a highest-non-air block in these \
             post-flattening fixtures"
        );

        let best_match = match_exact.max(match_minus_min_y);
        if fixture == "v1_17_40_superflat" {
            // The 2D, flat fixture: every column is trivially uniform, so even this
            // naive definition matches exactly.
            assert_eq!(
                best_match, columns_checked,
                "{fixture}: expected the naive definition to match every column on flat \
                 terrain"
            );
        } else {
            // The 3D fixtures: pin that the naive "highest non-air" definition stays
            // well short of matching the stored heightmap, so nobody later mistakes it
            // for the real definition -- see this test's module doc for what the real
            // one likely needs instead.
            assert!(
                (best_match as f64) < 0.70 * columns_checked as f64,
                "{fixture}: naive \"highest non-air\" definition matched {best_match}/\
                 {columns_checked} columns ({:.1}%), no longer clearly short of correct",
                pct(best_match)
            );
        }
    }
}

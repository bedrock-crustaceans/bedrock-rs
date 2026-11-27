//! Exercises the automatic content-upgrade pass ([`Chunk::upgrade_content`], driven by
//! [`Level::chunk`]/[`Level::chunks`] whenever a provider is attached) against a real
//! fixture, alongside the in-crate unit tests in `level_chunk.rs` that exercise the same
//! pass against synthetic data (legacy/numeric resolution, `NoEntry`/`Malformed` handling,
//! the neighbour pass). This file's job is the two things only a real fixture can pin: the
//! no-provider path stays byte-identical, and an attached provider's pass touches only the
//! entries it actually claims.

pub mod support;

use std::fs::File;

use bedrock_level::level::Level;
use bedrock_level::level_chunk::Terrain;
use bedrock_level::provider::{
    ContentProvider, ContextResolution, RelativeOffset, StateUpgrade, VersionedBlockDef,
};
use bedrock_level::subchunk::{BlockDef, PaletteEntry};
use flate2::read::GzDecoder;
use tar::Archive;

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

/// A fixture with paletted subchunks (so its entries are `PaletteEntry::Modern`, not the
/// pre-flattening forms the unit tests already cover) -- any such fixture does, this just
/// picks the first one `support::fixtures()` lists.
fn paletted_fixture() -> support::Fixture {
    support::fixtures()
        .into_iter()
        .find(|f| f.capabilities.paletted)
        .expect("at least one fixture has paletted subchunks")
}

/// Claims exactly one block name (`minecraft:air`), bumping it straight to whatever
/// `target` it is asked for with no rule applied -- the same degenerate "confirm and
/// restamp" shape `bedrock_vanilla_data::VanillaContent` uses for `source == target`,
/// stated here as a fixed rule instead so the test does not depend on that crate. Every
/// other name is `NoEntry`, unchanged.
///
/// Also resolves a pre-flattening air entry to the modern name (`{name, val}`'s `name` is
/// already `"minecraft:air"` on every era this format has ever used it under, and the
/// classic numeric id for air is `0`), so this provider gives the same outcome for air no
/// matter which era wrote the fixture chunk under test -- a fixture's palette entries can
/// be `PaletteEntry::Legacy`/`PaletteEntry::Numeric` even under this crate's "paletted"
/// capability flag, since that flag describes the subchunk's on-disk *layout*, not whether
/// flattening had happened yet (see Phase 1's legacy-palette-entry task).
struct AirBumpProvider;

fn air_stub() -> BlockDef {
    BlockDef {
        name: "minecraft:air".to_string(),
        version: None,
        states: indexmap::IndexMap::default(),
    }
}

impl ContentProvider for AirBumpProvider {
    fn legacy_block_state(&self, name: &str, _val: i16) -> Option<VersionedBlockDef> {
        (name == "minecraft:air").then(|| VersionedBlockDef {
            block: air_stub(),
            version: 1,
        })
    }

    fn numeric_block_state(&self, id: u8, _data: u8) -> Option<VersionedBlockDef> {
        (id == 0).then(|| VersionedBlockDef {
            block: air_stub(),
            version: 1,
        })
    }

    fn upgrade_block_state(&self, state: BlockDef, _source: i32, target: i32) -> StateUpgrade {
        if state.name == "minecraft:air" {
            StateUpgrade::Upgraded(VersionedBlockDef {
                block: state,
                version: target,
            })
        } else {
            StateUpgrade::NoEntry(state)
        }
    }

    fn biome_name(&self, _id: i32) -> Option<&str> {
        None
    }
}

/// Every layer-0/layer-1 palette entry across every subchunk of `chunk`, as
/// `(subchunk index, layer index, entry)` triples -- the same granularity
/// `EntryLocation` reports at, used here to line two decodes of the same chunk up
/// entry-for-entry.
fn palette_entries(chunk: &bedrock_level::level_chunk::Chunk) -> Vec<(i8, usize, PaletteEntry)> {
    let mut out = Vec::new();
    let Terrain::SubChunks(subchunks) = chunk.terrain() else {
        return out;
    };
    for (&index, subchunk) in subchunks {
        let mut layer_index = 0usize;
        while let Some(layer) = subchunk.get_layer(layer_index) {
            for entry in layer.palette() {
                out.push((index, layer_index, entry.clone()));
            }
            layer_index += 1;
        }
    }
    out
}

/// The first chunk position in `dimension` whose decoded terrain carries at least one
/// `minecraft:air` palette entry -- not just the first position the level's scan happens
/// to list, which can as easily be an administrative stub (a `digp`/finalized-state-only
/// chunk with no terrain at all) as a real one. Every test in this file needs a chunk with
/// actual paletted content to say anything about the pass at all.
fn chunk_position_with_air(
    level: &Level,
    dimension: bedrock_shared::world::dimension::Dimension,
) -> bedrock_level::types::ChunkPosition {
    for position in level.chunk_positions(dimension) {
        let Some(chunk) = level.chunk(dimension, position).expect("chunk read") else {
            continue;
        };
        if palette_entries(&chunk)
            .iter()
            .any(|(_, _, entry)| entry.name() == Some("minecraft:air"))
        {
            return position;
        }
    }
    panic!("no chunk in {dimension:?} has a minecraft:air palette entry");
}

/// The no-provider path is unaffected by this pass existing: a chunk read through a
/// `Level` with no provider attached decodes exactly as before, and carries no upgrade
/// report at all.
#[test]
fn no_provider_leaves_a_chunk_untouched() {
    let fixture = paletted_fixture();
    let tmp = extract_fixture(&fixture);
    let level = Level::open(tmp.path().join("world")).expect("Level::open");

    let dimension = level
        .dimensions()
        .first()
        .copied()
        .expect("fixture has at least one dimension");
    let position = chunk_position_with_air(&level, dimension);

    let chunk = level
        .chunk(dimension, position)
        .expect("chunk read")
        .expect("chunk present");

    assert!(chunk.content_upgrade().is_none());
    assert_eq!(chunk.achieved_block_state_version(), None);
}

/// A provider attached via `Level::with_provider` runs automatically on every
/// `Level::chunk` read (architecture decision 4), and its pass touches exactly the
/// entries it claims: every `minecraft:air` entry is restamped to the pass's target, and
/// every other entry is byte-for-byte identical to the same chunk decoded with no
/// provider at all.
#[test]
fn provider_touches_only_the_entries_it_claims() {
    let fixture = paletted_fixture();

    let plain_tmp = extract_fixture(&fixture);
    let plain_level = Level::open(plain_tmp.path().join("world")).expect("Level::open");

    let upgraded_tmp = extract_fixture(&fixture);
    let upgraded_level = Level::open(upgraded_tmp.path().join("world"))
        .expect("Level::open")
        .with_provider(AirBumpProvider);

    let dimension = plain_level
        .dimensions()
        .first()
        .copied()
        .expect("fixture has at least one dimension");
    let position = chunk_position_with_air(&plain_level, dimension);

    let plain_chunk = plain_level
        .chunk(dimension, position)
        .expect("chunk read")
        .expect("chunk present");
    let upgraded_chunk = upgraded_level
        .chunk(dimension, position)
        .expect("chunk read")
        .expect("chunk present");

    let target = upgraded_level.block_state_write_target();
    let report = upgraded_chunk
        .content_upgrade()
        .expect("a provider is attached, so the pass ran");

    let plain_entries = palette_entries(&plain_chunk);
    let upgraded_entries = palette_entries(&upgraded_chunk);
    assert_eq!(
        plain_entries.len(),
        upgraded_entries.len(),
        "the pass must not add or remove palette entries, only convert their content"
    );

    let mut saw_air = false;
    for (plain, upgraded) in plain_entries.iter().zip(upgraded_entries.iter()) {
        let (subchunk, layer, plain_entry) = plain;
        let (_, _, upgraded_entry) = upgraded;

        if plain_entry.name() == Some("minecraft:air") {
            saw_air = true;
            let block = upgraded_entry
                .as_block()
                .expect("an air entry stays Modern after the pass");
            assert_eq!(block.version, Some(target));
            assert_eq!(block.name, "minecraft:air");
        } else {
            assert_eq!(
                plain_entry, upgraded_entry,
                "subchunk {subchunk} layer {layer}: an entry the provider does not claim \
                 must be byte-for-byte unchanged"
            );
        }
    }
    assert!(
        saw_air,
        "fixture chunk is expected to contain at least one minecraft:air palette entry"
    );

    // The chunk-wide achieved version is the minimum over every visited entry, so it
    // cannot exceed the target the (mostly unclaimed) entries were even asked to reach.
    let achieved = report
        .achieved_block_state_version()
        .expect("at least one entry was visited");
    assert!(achieved <= target);
}

/// Editing nothing and writing an upgraded chunk back through `Level::put_chunk` persists
/// the upgraded palette: reopening without a provider sees the restamped `minecraft:air`
/// entries the first `Level::chunk` call produced.
#[test]
fn put_chunk_after_upgrade_persists_the_upgraded_palette() {
    let fixture = paletted_fixture();
    let tmp = extract_fixture(&fixture);

    let dimension;
    let position;
    let target;
    {
        let mut level = Level::open_rw(tmp.path().join("world"))
            .expect("Level::open_rw")
            .with_provider(AirBumpProvider);

        dimension = level
            .dimensions()
            .first()
            .copied()
            .expect("fixture has at least one dimension");
        position = chunk_position_with_air(&level, dimension);
        target = level.block_state_write_target();

        let chunk = level
            .chunk(dimension, position)
            .expect("chunk read")
            .expect("chunk present");
        assert!(chunk.content_upgrade().is_some());

        let _report = level
            .put_chunk(dimension, position, &chunk)
            .expect("put_chunk");
    }

    // Reopen with no provider at all -- what comes back must be what was actually
    // written, not another live run of the pass.
    let level = Level::open(tmp.path().join("world")).expect("reopen");
    let chunk = level
        .chunk(dimension, position)
        .expect("chunk read")
        .expect("chunk present");
    assert!(chunk.content_upgrade().is_none());

    let entries = palette_entries(&chunk);
    let air_entries: Vec<_> = entries
        .iter()
        .filter(|(_, _, entry)| entry.name() == Some("minecraft:air"))
        .collect();
    assert!(
        !air_entries.is_empty(),
        "fixture chunk is expected to contain at least one minecraft:air palette entry"
    );
    for (_, _, entry) in air_entries {
        let block = entry.as_block().expect("air entry stays Modern");
        assert_eq!(block.version, Some(target));
    }
}

/// A provider that flags one block and resolves it from a same-chunk neighbour runs
/// through the full `Level`-driven pass, not just the unit-level `Chunk::upgrade_content`
/// call -- this is the "exercise the pass" requirement's real-`Level` half; the
/// synthetic-data half (asserting exactly which position got which outcome) lives with
/// `Chunk::upgrade_content`'s own unit tests in `level_chunk.rs`, which do not need a real
/// fixture to set up a controlled neighbour arrangement.
struct NeighbourFlaggingProvider;

impl ContentProvider for NeighbourFlaggingProvider {
    /// A generic pass-through rather than an air-specific rule: a real legacy `{name,
    /// val}` entry's `name` is already the modern dotted identifier (Phase 1 confirmed
    /// this against the whole corpus), so carrying it straight into `Modern` with empty
    /// `states` is enough to let `needs_neighbours`'s name check below see
    /// `"minecraft:air"` regardless of which era wrote this fixture chunk.
    fn legacy_block_state(&self, name: &str, _val: i16) -> Option<VersionedBlockDef> {
        Some(VersionedBlockDef {
            block: BlockDef {
                name: name.to_string(),
                version: None,
                states: indexmap::IndexMap::default(),
            },
            version: 1,
        })
    }

    fn numeric_block_state(&self, _id: u8, _data: u8) -> Option<VersionedBlockDef> {
        None
    }

    fn upgrade_block_state(&self, state: BlockDef, _source: i32, target: i32) -> StateUpgrade {
        StateUpgrade::Upgraded(VersionedBlockDef {
            block: state,
            version: target,
        })
    }

    fn needs_neighbours(&self, state: &BlockDef) -> bool {
        state.name == "minecraft:air"
    }

    fn resolve_with_neighbours<'n>(
        &self,
        state: BlockDef,
        neighbour: &mut dyn FnMut(RelativeOffset) -> Option<&'n BlockDef>,
    ) -> ContextResolution {
        match neighbour(RelativeOffset::new(0, 1, 0)) {
            Some(_) => ContextResolution::Resolved(state),
            None => ContextResolution::NoEntry(state),
        }
    }

    fn biome_name(&self, _id: i32) -> Option<&str> {
        None
    }
}

#[test]
fn neighbour_pass_runs_through_a_real_level_without_panicking() {
    let fixture = paletted_fixture();
    let tmp = extract_fixture(&fixture);
    let level = Level::open(tmp.path().join("world"))
        .expect("Level::open")
        .with_provider(NeighbourFlaggingProvider);

    let dimension = level
        .dimensions()
        .first()
        .copied()
        .expect("fixture has at least one dimension");
    let position = chunk_position_with_air(&level, dimension);

    let chunk = level
        .chunk(dimension, position)
        .expect("chunk read")
        .expect("chunk present");
    let report = chunk
        .content_upgrade()
        .expect("a provider is attached, so the pass ran");

    // Every `minecraft:air` position was flagged and revisited -- the neighbour pass
    // reports something for each one, `Resolved` since `minecraft:air` blocks are common
    // enough that at least one has a same-chunk block above it.
    assert!(report.neighbour_resolutions().next().is_some());
}

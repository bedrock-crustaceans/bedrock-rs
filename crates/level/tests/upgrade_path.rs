//! Phase 7's upgrade-path tests, scoped to what this crate actually has: no
//! content-upgrade chain exists yet (`level_chunk`'s module docs -- a
//! version-42 chunk's numeric palette entry stays numeric, a legacy
//! `{name, val}` entry stays legacy), so nothing here can assert that an
//! upgraded chunk's *content* (the actual block a position holds) matches
//! what the same position holds in an equivalent modern world. What can be
//! asserted honestly is structural: does the upgraded record carry the
//! shape a modern chunk of its kind carries -- the same subchunk set, the
//! same per-layer palette size, the same biome storage form.
//!
//! The corpus's one fixture that is itself the product of an upgrade,
//! `v1_18_updated_superflat` (a 1.17.40 superflat world last opened by
//! 1.18), makes this concrete rather than hypothetical: 65 of its 72
//! Overworld chunks were never touched by the upgrade and still carry
//! chunk version 22 (2D biomes, the pre-negative-Y height range), and the
//! other 7 were rewritten at version 39 (3D biomes) the moment 1.18 first
//! loaded them. That split, discovered by direct inspection while writing
//! this test (not asserted anywhere before it), is exactly the trio the
//! TODO's task asks for:
//!
//! - `v1_17_40_superflat` -- the pre-upgrade world, chunk version 22
//!   throughout.
//! - `v1_18_updated_superflat` -- the same world after 1.18 touched a
//!   fraction of its chunks. Same chunk positions as the fixture above,
//!   65/72 still version 22, 7/72 now version 39.
//! - `v1_18_superflat` -- an unrelated, freshly generated 1.18 superflat
//!   world (different chunk positions entirely -- 1.18 world generation
//!   does not produce the same chunk coordinates as 1.17.40's), useful only
//!   as "what a chunk version 1.18 writes from scratch looks like".
//!
//! Two comparisons follow from that shape: the *untouched* 65 chunks
//! against their own pre-upgrade selves (an exact, byte-identical check --
//! stronger than the TODO asks for, and worth pinning because it is not
//! obvious a chunk the upgrade never visits stays untouched at the raw
//! byte level rather than merely at the decoded level), and the *touched*
//! 7 chunks against `v1_18_superflat`'s structural shape (weaker, since the
//! positions differ, but still real: same subchunk count, same palette
//! size, same biome form).
//!
//! What neither comparison can do, and this file has no test for: confirm
//! that an upgraded chunk's blocks are the *same* blocks a fresh world
//! would place at the equivalent position, only that the record shapes
//! match. That needs architecture decision 9's content-mapping provider and
//! the upgrade-chain machinery `level_chunk`'s module docs describe as not
//! yet existing -- revisit once both land.

mod support;

use std::collections::{BTreeMap, HashMap, HashSet};

use bedrock_level::chunk::ChunkRecords;
use bedrock_level::key::KeyVariant;
use bedrock_level::level_chunk::{Chunk, ChunkBiomes, Terrain};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;

/// Per-subchunk shape: its vertical index paired with each layer's palette
/// length, in layer order. Two subchunks with the same shape are not
/// necessarily the same blocks (a palette of 4 entries says nothing about
/// which 4) -- this is deliberately the weakest fact that is still
/// meaningful, since content equivalence is exactly what this test cannot
/// assert. See the module docs.
fn subchunk_shapes(chunk: &Chunk) -> Vec<(i8, Vec<usize>)> {
    let Terrain::SubChunks(subchunks) = chunk.terrain() else {
        return Vec::new();
    };
    subchunks
        .iter()
        .map(|(&index, subchunk)| {
            let mut lens = Vec::new();
            let mut i = 0;
            while let Some(layer) = subchunk.get_layer(i) {
                lens.push(layer.palette().len());
                i += 1;
            }
            (index, lens)
        })
        .collect()
}

fn biome_form(chunk: &Chunk) -> &'static str {
    match chunk.biomes() {
        Some(ChunkBiomes::Columns(_)) => "2d",
        Some(ChunkBiomes::Volumes(_)) => "3d",
        None => "none",
    }
}

fn chunk_version_of(records: &ChunkRecords) -> Option<u8> {
    records
        .get(KeyVariant::ChunkVersion)
        .or_else(|| records.get(KeyVariant::LegacyVersion))
        .and_then(|bytes| bytes.first().copied())
}

/// The chunk positions the pre-upgrade world and the post-upgrade world
/// share are exactly the same set, and within that set, every chunk the
/// upgrade left at its original version (22) is byte-for-byte identical to
/// the pre-upgrade record -- the strongest honest claim this crate can make
/// without a content-upgrade chain: an untouched record really is
/// untouched, all the way down to the bytes, not merely "decodes the same".
///
/// Pinned counts (65 untouched / 7 upgraded out of 72) are exact rather
/// than `>0` so a change in either fixture's content is noticed rather than
/// silently absorbed by a loosened assertion.
#[test]
fn untouched_chunks_are_byte_identical_across_the_upgrade() {
    let pre = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_17_40_superflat")
        .expect("v1_17_40_superflat fixture is present");
    let post = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_updated_superflat")
        .expect("v1_18_updated_superflat fixture is present");

    let (_tmp_pre, db_pre) = pre.open();
    let (_tmp_post, db_post) = post.open();
    let pre_records = ChunkRecords::read_all(&db_pre).unwrap();
    let post_records = ChunkRecords::read_all(&db_post).unwrap();

    let pre_ids: HashSet<(ChunkPosition, Dimension)> = pre_records.keys().copied().collect();
    let post_ids: HashSet<(ChunkPosition, Dimension)> = post_records.keys().copied().collect();
    assert_eq!(
        pre_ids, post_ids,
        "the pre- and post-upgrade worlds should carry exactly the same chunk positions -- \
         the upgrade adds or removes chunks, only rewrites some of them"
    );

    let mut untouched = 0usize;
    let mut upgraded = 0usize;
    let mut other = Vec::new();

    for (id, pre_group) in &pre_records {
        let post_group = &post_records[id];
        let post_version = chunk_version_of(post_group);
        match post_version {
            Some(22) => {
                untouched += 1;
                let pre_map: HashMap<KeyVariant, &[u8]> =
                    pre_group.iter().map(|(k, b)| (k.data, b)).collect();
                let post_map: HashMap<KeyVariant, &[u8]> =
                    post_group.iter().map(|(k, b)| (k.data, b)).collect();
                assert_eq!(
                    pre_map, post_map,
                    "{id:?}: still chunk version 22 in the post-upgrade world, but its record \
                     group is not byte-identical to the pre-upgrade world"
                );
            }
            Some(39) => upgraded += 1,
            other_version => other.push((*id, other_version)),
        }
    }

    assert!(
        other.is_empty(),
        "chunk(s) at a version other than 22 or 39: {other:?}"
    );
    assert_eq!(
        (untouched, upgraded),
        (65, 7),
        "the untouched/upgraded split changed shape -- update this test's pinned counts and \
         its module doc comment together"
    );
}

/// The 7 chunks the upgrade rewrote share `v1_18_superflat`'s structural
/// shape -- one populated subchunk, one block layer, the same palette size,
/// 3D biomes -- even though they sit at different chunk positions and (per
/// the fixture index) different absolute heights: the upgraded chunks stay
/// in their original chunk's 0..16 height range (subchunk index 0) rather
/// than being moved into 1.18's new -64..320 range that a freshly generated
/// world uses (subchunk index -4 for the equivalent flat-terrain layer).
/// That divergence is real and expected, not a bug this test is hiding --
/// it is exactly the fact `v1_18_updated_superflat`'s `subchunk_height_range`
/// in `index.json` (`0..16`) already records against `v1_18_superflat`'s
/// (`-64..-48`), and it is stated here rather than asserted away.
///
/// What this test does *not* and cannot claim: that the upgraded chunk's
/// palette holds the *same blocks* as the freshly generated world's, only
/// that it holds the same *number* of distinct entries in the same shape.
/// Confirming the blocks themselves match needs the content-upgrade chain
/// (architecture decision 9's provider), which does not exist yet -- see
/// this file's module docs.
#[test]
fn upgraded_chunks_match_a_fresh_worlds_structural_shape() {
    let post = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_updated_superflat")
        .expect("v1_18_updated_superflat fixture is present");
    let fresh = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_superflat")
        .expect("v1_18_superflat fixture is present");

    let (_tmp_post, db_post) = post.open();
    let (_tmp_fresh, db_fresh) = fresh.open();
    let post_records = ChunkRecords::read_all(&db_post).unwrap();
    let fresh_records = ChunkRecords::read_all(&db_fresh).unwrap();

    let mut upgraded_shapes: Vec<Vec<(i8, Vec<usize>)>> = Vec::new();
    let mut upgraded_biome_forms: Vec<&'static str> = Vec::new();
    for (id, records) in &post_records {
        if chunk_version_of(records) != Some(39) {
            continue;
        }
        let chunk = Chunk::decode(records.clone(), &db_post)
            .unwrap_or_else(|e| panic!("{id:?}: Chunk::decode failed: {e}"));
        upgraded_shapes.push(subchunk_shapes(&chunk));
        upgraded_biome_forms.push(biome_form(&chunk));
    }

    let mut fresh_shapes: Vec<Vec<(i8, Vec<usize>)>> = Vec::new();
    let mut fresh_biome_forms: Vec<&'static str> = Vec::new();
    for (id, records) in &fresh_records {
        let chunk = Chunk::decode(records.clone(), &db_fresh)
            .unwrap_or_else(|e| panic!("{id:?}: Chunk::decode failed: {e}"));
        fresh_shapes.push(subchunk_shapes(&chunk));
        fresh_biome_forms.push(biome_form(&chunk));
    }

    assert_eq!(upgraded_shapes.len(), 7, "expected 7 upgraded chunks");
    assert_eq!(fresh_shapes.len(), 6, "expected 6 freshly generated chunks");

    // Every biome record is 3D on both sides.
    assert!(upgraded_biome_forms.iter().all(|&f| f == "3d"));
    assert!(fresh_biome_forms.iter().all(|&f| f == "3d"));

    // Every chunk on both sides has exactly one populated subchunk, one
    // block layer, holding the same palette size -- the flat terrain's
    // single slab of bedrock/dirt/grass-equivalent blocks, whatever their
    // exact names turn out to be once content mapping exists.
    for shapes in upgraded_shapes.iter().chain(fresh_shapes.iter()) {
        assert_eq!(
            shapes.len(),
            1,
            "expected exactly one populated subchunk, found {shapes:?}"
        );
        let (_, layers) = &shapes[0];
        assert_eq!(
            layers,
            &vec![4usize],
            "expected exactly one block layer with a 4-entry palette, found {layers:?}"
        );
    }

    // The honest divergence: absolute placement differs by exactly the
    // height-range gap `index.json` already records (updated_superflat
    // stayed at 0..16, subchunk index 0; a fresh 1.18 world uses -64..320,
    // and this flat terrain's equivalent slab lands at subchunk index -4).
    let upgraded_indices: BTreeMap<i8, usize> =
        upgraded_shapes.iter().fold(BTreeMap::new(), |mut acc, s| {
            *acc.entry(s[0].0).or_insert(0) += 1;
            acc
        });
    let fresh_indices: BTreeMap<i8, usize> =
        fresh_shapes.iter().fold(BTreeMap::new(), |mut acc, s| {
            *acc.entry(s[0].0).or_insert(0) += 1;
            acc
        });
    assert_eq!(
        upgraded_indices,
        BTreeMap::from([(0i8, 7usize)]),
        "expected every upgraded chunk's populated subchunk at index 0 (its unmoved 0..16 \
         height range)"
    );
    assert_eq!(
        fresh_indices,
        BTreeMap::from([(-4i8, 6usize)]),
        "expected every freshly generated chunk's populated subchunk at index -4 (a fresh \
         1.18 world's -64..320 height range)"
    );
}

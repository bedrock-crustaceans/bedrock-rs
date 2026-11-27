//! Pins `VanillaContent::biome_name` against the real generated biome tables -- see
//! `biome.rs`'s module doc comment for why coverage only begins at `v1.21.130.3`
//! (`xtask/dumps/bedrock-samples/COVERAGE.md`'s biome section has the survey).
#![cfg(feature = "generated-palettes")]

use bedrock_level::provider::ContentProvider;
use bedrock_vanilla_data::VanillaContent;
use bedrock_vanilla_data::biome;

#[test]
fn biome_name_resolves_a_real_id() {
    let provider = VanillaContent;
    // `minecraft:ocean` is id 0 in every vendored biome dump that carries a numeric id.
    assert_eq!(provider.biome_name(0), Some("minecraft:ocean"));
    assert_eq!(provider.biome_name(1), Some("minecraft:plains"));
}

#[test]
fn biome_name_reports_an_unassigned_id_as_none() {
    let provider = VanillaContent;
    assert_eq!(provider.biome_name(-1), None);
    assert_eq!(provider.biome_name(i32::MAX), None);
}

#[test]
fn biomes_for_version_finds_the_earliest_and_latest_pinned_biome_tables() {
    let earliest = biome::biomes_for_version(1, 21, 130).expect("v1.21.130 should be generated");
    assert_eq!(earliest.name(0), Some("minecraft:ocean"));

    let latest = biome::biomes_for_version(1, 26, 40).expect("v1.26.40 should be generated");
    assert_eq!(latest.name(0), Some("minecraft:ocean"));
}

#[test]
fn biomes_for_version_reports_none_for_a_block_only_version() {
    // `v1.20.10` has a generated block palette but no biome dump at all -- confirmed
    // against the upstream tag tree, not assumed. `biomes_for_version` must not paper
    // over that by falling back to a neighbouring table.
    assert!(biome::biomes_for_version(1, 20, 10).is_none());
}

#[test]
fn biomes_for_version_covers_v1_26_50_and_v1_26_60() {
    for (minor, patch) in [(26, 50), (26, 60)] {
        let table = biome::biomes_for_version(1, minor, patch)
            .unwrap_or_else(|| panic!("v1.{minor}.{patch} should be generated"));
        assert_eq!(table.name(0), Some("minecraft:ocean"));
    }
}

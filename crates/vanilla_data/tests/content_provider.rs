//! Pins `VanillaContent`'s honest-no-entry behaviour (architecture decision 9).
//!
//! `legacy_block_state`/`numeric_block_state` stay `None` unconditionally -- no dump
//! this generator can regenerate from covers the pre-flattening era at all (see
//! `lib.rs`'s `VanillaContent` doc comment), so those two are pinned here regardless of
//! the `generated-palettes` feature. `biome_name` and `upgrade_block_state` are
//! different: once real generated tables exist (`generated-palettes` on), both can and
//! do answer real lookups -- see `generated_palette_provider.rs` and
//! `cross_version_upgrade.rs` for those. The two tests below stay scoped to the
//! no-tables-compiled-in build, where "unknown" is not an approximation but the
//! honestly correct answer.

#[cfg(not(feature = "generated-palettes"))]
use bedrock_level::provider::StateUpgrade;
use bedrock_level::provider::{ContentProvider, States};
use bedrock_level::subchunk::BlockDef;
use bedrock_vanilla_data::VanillaContent;

fn state(name: &str, version: Option<i32>) -> BlockDef {
    BlockDef {
        name: name.to_string(),
        version,
        states: States::new(),
    }
}

#[test]
fn legacy_block_state_has_no_entries() {
    let provider = VanillaContent;

    // A representative sweep, not an exhaustive one: the point is that nothing is
    // hand-authored (architecture decision 12), so no `(name, val)` pair -- vanilla or
    // otherwise -- resolves yet.
    assert!(provider.legacy_block_state("minecraft:wood", 0).is_none());
    assert!(provider.legacy_block_state("minecraft:wool", 14).is_none());
    assert!(provider.legacy_block_state("", 0).is_none());
}

#[test]
fn numeric_block_state_has_no_entries() {
    let provider = VanillaContent;

    assert!(provider.numeric_block_state(1, 0).is_none());
    assert!(provider.numeric_block_state(5, 3).is_none());
    assert!(provider.numeric_block_state(0, 0).is_none());
}

#[test]
#[cfg(not(feature = "generated-palettes"))]
fn upgrade_block_state_is_always_no_entry_and_preserves_the_original() {
    let provider = VanillaContent;
    let original = state("minecraft:stone", Some(17_760_256));

    match provider.upgrade_block_state(original.clone(), 17_760_256, 18_090_528) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry (no schema tables yet), got {other:?}"),
    }
}

#[test]
#[cfg(not(feature = "generated-palettes"))]
fn biome_name_has_no_entries() {
    let provider = VanillaContent;

    assert!(provider.biome_name(1).is_none());
    assert!(provider.biome_name(0).is_none());
    assert!(provider.biome_name(-1).is_none());
}

#[test]
fn neighbour_hooks_use_the_trait_default_and_never_flag_anything() {
    let provider = VanillaContent;
    let stairs = state("minecraft:oak_stairs", Some(1));

    // `VanillaContent` does not override `needs_neighbours`/`resolve_with_neighbours`,
    // so it inherits the trait's opt-out default -- pinned here so a future override
    // is a deliberate, visible change rather than a silent behaviour shift.
    assert!(!provider.needs_neighbours(&stairs));
}

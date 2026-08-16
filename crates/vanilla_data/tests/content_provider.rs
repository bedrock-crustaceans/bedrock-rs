//! Pins `VanillaContent`'s honest-no-entry behaviour (architecture decision 9): with
//! no generated tables yet, every lookup reports "unknown" rather than fabricating an
//! answer, and `upgrade_block_state` always answers `NoEntry`, handing the entry back
//! unchanged instead of stamping it as upgraded. This is the contract other crates are
//! expected to rely on until the schema generator (Phase 3's `xtask` track) lands.

use bedrock_level::provider::{ContentProvider, StateUpgrade, States};
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
fn upgrade_block_state_is_always_no_entry_and_preserves_the_original() {
    let provider = VanillaContent;
    let original = state("minecraft:stone", Some(17_760_256));

    match provider.upgrade_block_state(original.clone(), 17_760_256, 18_090_528) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry (no schema tables yet), got {other:?}"),
    }
}

#[test]
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

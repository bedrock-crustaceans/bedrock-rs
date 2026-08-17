//! Pins `VanillaContent`'s one real (non-`NoEntry`) answer under the `generated-palettes`
//! feature: confirming a block name is one the game actually shipped at an exact version,
//! via `upgrade_block_state`'s `source == target` degenerate case. See `lib.rs`'s doc
//! comment on `VanillaContent` for why this is the whole of what the generated data
//! honestly supports today, and `content_provider.rs` for the no-feature baseline this
//! extends.
#![cfg(feature = "generated-palettes")]

use bedrock_level::provider::{ContentProvider, StateUpgrade, States};
use bedrock_level::subchunk::BlockDef;
use bedrock_vanilla_data::VanillaContent;
use bedrock_vanilla_data::palette::{self, PropertyKind};

fn state(name: &str) -> BlockDef {
    BlockDef {
        name: name.to_string(),
        version: None,
        states: States::new(),
    }
}

fn state_with(name: &str, properties: &[(&str, nbtx::Value)]) -> BlockDef {
    let mut states = States::new();
    for (key, value) in properties {
        states.insert(key.to_string(), value.clone());
    }
    BlockDef {
        name: name.to_string(),
        version: None,
        states,
    }
}

/// `v1.20.10.1` is the earliest pinned dump; `(1, 20, 10)` is its game-version key.
const V1_20_10: i32 = BlockDef::pack_version([1, 20, 10, 0]);

#[test]
fn palette_lookup_finds_a_real_block_with_no_properties() {
    let palette = palette::palette_for_version(1, 20, 10).expect("v1.20.10 should be generated");
    let fence = palette
        .block("minecraft:acacia_fence")
        .expect("acacia_fence should be present");
    assert!(fence.properties.is_empty());
}

#[test]
fn palette_lookup_finds_a_real_property_with_its_value_domain() {
    let palette = palette::palette_for_version(1, 20, 10).unwrap();
    let attachment = palette
        .property("attachment")
        .expect("attachment should be a known property");
    assert_eq!(
        attachment.kind,
        PropertyKind::Str(&["hanging", "multiple", "side", "standing"])
    );
}

#[test]
fn palette_lookup_reports_unknown_name_and_unknown_version_as_none() {
    assert!(
        palette::palette_for_version(1, 20, 10)
            .unwrap()
            .block("minecraft:nonexistent")
            .is_none()
    );
    // No dump was pinned for this version.
    assert!(palette::palette_for_version(9, 9, 9).is_none());
}

#[test]
fn upgrade_block_state_confirms_a_known_name_at_a_known_version() {
    let provider = VanillaContent;
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Upgraded(upgraded) => {
            assert_eq!(upgraded.version, V1_20_10);
            // Confirmed, not fabricated: the entry comes back exactly as given, no
            // properties filled in, because the dump carries no defaults to fill them
            // from (see the module docs).
            assert_eq!(upgraded.block, original);
        }
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_no_entry_for_an_unknown_block_name() {
    let provider = VanillaContent;
    let original = state("minecraft:this_block_does_not_exist");

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_no_entry_for_a_version_with_no_generated_table() {
    let provider = VanillaContent;
    let no_such_version = BlockDef::pack_version([9, 9, 9, 0]);
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original.clone(), no_such_version, no_such_version) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_no_entry_across_two_different_versions() {
    let provider = VanillaContent;
    let v1_21_0 = BlockDef::pack_version([1, 21, 0, 0]);
    let original = state("minecraft:acacia_fence");

    // Both versions have real generated tables and both contain this block, but
    // cross-version diffing is not implemented -- see the module docs.
    match provider.upgrade_block_state(original.clone(), V1_20_10, v1_21_0) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_ignores_the_revision_byte_when_matching() {
    let provider = VanillaContent;
    // Same (major, minor, patch) as V1_20_10, different revision -- still one exact
    // `source == target` word, so still resolves against the same generated table.
    let with_revision = BlockDef::pack_version([1, 20, 10, 7]);
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original, with_revision, with_revision) {
        StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.version, with_revision),
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

/// `minecraft:acacia_stairs` at `v1.20.10.1` declares exactly `upside_down_bit` (bool)
/// and `weirdo_direction` (int, domain `0..=3`) -- real facts about the pinned dump,
/// used across the tests below.
#[test]
fn upgrade_block_state_accepts_a_state_that_omits_a_declared_property() {
    let provider = VanillaContent;
    // Only one of the two properties the block declares is present -- not a
    // completeness check, so this is still accepted.
    let original = state_with(
        "minecraft:acacia_stairs",
        &[("upside_down_bit", nbtx::Value::Byte(1))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.block, original),
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_accepts_a_fully_specified_valid_state() {
    let provider = VanillaContent;
    let original = state_with(
        "minecraft:acacia_stairs",
        &[
            ("upside_down_bit", nbtx::Value::Byte(1)),
            ("weirdo_direction", nbtx::Value::Int(2)),
        ],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.block, original),
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_accepts_an_integer_property_under_a_different_tag_width() {
    let provider = VanillaContent;
    // `weirdo_direction` is stored as an `Int` in the dump's own vocabulary, but the
    // format is known to carry the same logical value under different tag widths --
    // this should still validate against the same domain.
    let original = state_with(
        "minecraft:acacia_stairs",
        &[("weirdo_direction", nbtx::Value::Byte(2))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Upgraded(upgraded) => assert_eq!(upgraded.block, original),
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_rejects_a_property_the_block_does_not_declare() {
    let provider = VanillaContent;
    // `minecraft:acacia_fence` declares no properties at all at this version.
    let original = state_with(
        "minecraft:acacia_fence",
        &[("weirdo_direction", nbtx::Value::Int(0))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Malformed(returned, reason) => {
            assert_eq!(returned, original);
            assert!(reason.0.contains("weirdo_direction"), "{reason}");
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_rejects_a_value_outside_the_declared_domain() {
    let provider = VanillaContent;
    // `weirdo_direction`'s real domain is `0..=3`; 99 is not in it.
    let original = state_with(
        "minecraft:acacia_stairs",
        &[("weirdo_direction", nbtx::Value::Int(99))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Malformed(returned, reason) => {
            assert_eq!(returned, original);
            assert!(reason.0.contains("weirdo_direction"), "{reason}");
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn upgrade_block_state_rejects_a_bool_property_given_an_out_of_range_value() {
    let provider = VanillaContent;
    let original = state_with(
        "minecraft:acacia_stairs",
        &[("upside_down_bit", nbtx::Value::Byte(7))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_20_10) {
        StateUpgrade::Malformed(returned, reason) => {
            assert_eq!(returned, original);
            assert!(reason.0.contains("upside_down_bit"), "{reason}");
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

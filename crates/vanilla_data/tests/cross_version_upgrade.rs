//! Exercises `VanillaContent::upgrade_block_state`'s cross-version walk (`source != target`)
//! against real facts from the pinned dumps -- see `lib.rs`'s doc comment on
//! `VanillaContent` for the walk's rules. `generated_palette_provider.rs` covers the
//! `source == target` degenerate case this extends.
#![cfg(feature = "generated-palettes")]

use bedrock_level::provider::{ContentProvider, StateUpgrade, States};
use bedrock_level::subchunk::BlockDef;
use bedrock_vanilla_data::VanillaContent;

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

const V1_20_10: i32 = BlockDef::pack_version([1, 20, 10, 0]);
const V1_21_0: i32 = BlockDef::pack_version([1, 21, 0, 0]);
const V1_21_60: i32 = BlockDef::pack_version([1, 21, 60, 0]);
const V1_26_40: i32 = BlockDef::pack_version([1, 26, 40, 0]);
const V1_26_50: i32 = BlockDef::pack_version([1, 26, 50, 0]);
const V1_26_60: i32 = BlockDef::pack_version([1, 26, 60, 0]);

/// `minecraft:acacia_fence` is a real block that persists, unchanged, across every one
/// of the eight pinned versions -- confirmed directly against the vendored dumps, not
/// assumed. A full-range walk should reach the target exactly.
#[test]
fn walks_the_full_pinned_range_when_the_block_persists_throughout() {
    let provider = VanillaContent;
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original, V1_20_10, V1_26_40) {
        StateUpgrade::Upgraded(upgraded) => {
            assert_eq!(upgraded.version, V1_26_40);
            assert_eq!(upgraded.block.name, "minecraft:acacia_fence");
        }
        other => panic!("expected Upgraded, got {other:?}"),
    }
}

/// `minecraft:chemistry_table` is a real block present at `v1.20.10.1` and `v1.21.0.3`,
/// then removed by `v1.21.60.10` -- confirmed directly against the vendored dumps. Asking
/// to reach `v1.21.60.10` (or further) should report the progress the walk actually made
/// (`v1.21.0.3`), not `NoEntry`: the very first step succeeded, only the second could not
/// be taken. This is architecture decision 6's honest achieved-version report.
#[test]
fn reports_the_furthest_version_actually_reached_when_a_block_is_later_removed() {
    let provider = VanillaContent;
    let original = state_with(
        "minecraft:chemistry_table",
        &[
            (
                "chemistry_table_type",
                nbtx::Value::String("lab_table".into()),
            ),
            ("direction", nbtx::Value::Int(2)),
        ],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_21_60) {
        StateUpgrade::Upgraded(upgraded) => {
            assert_eq!(upgraded.version, V1_21_0);
            assert_eq!(upgraded.block.name, "minecraft:chemistry_table");
            // The properties were still declared, and the values still in domain, at
            // the version actually reached -- carried through unchanged.
            assert_eq!(upgraded.block.states, original.states);
        }
        other => panic!("expected Upgraded (partial progress), got {other:?}"),
    }
}

/// `minecraft:concrete_powder` is a real block present only at `v1.20.10.1` among the
/// pinned versions -- removed by `v1.21.0.3`, the very next step. Asking to upgrade past
/// it should report `NoEntry`: the walk could not take even its first step, matching this
/// trait's "no rule starting from this entry's source version at all" contract.
#[test]
fn no_entry_when_the_block_is_removed_at_the_very_first_step() {
    let provider = VanillaContent;
    let original = state_with(
        "minecraft:concrete_powder",
        &[("color", nbtx::Value::String("red".into()))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_21_0) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

/// A state that does not match its own claimed source version is `Malformed`, even when
/// asking to upgrade across versions -- the walk never starts if the very first check
/// (against `source`) already fails.
#[test]
fn malformed_when_the_source_state_does_not_match_its_own_stamp() {
    let provider = VanillaContent;
    // `minecraft:acacia_fence` declares no properties at `v1.20.10.1`.
    let original = state_with(
        "minecraft:acacia_fence",
        &[("nonexistent", nbtx::Value::Int(0))],
    );

    match provider.upgrade_block_state(original.clone(), V1_20_10, V1_26_40) {
        StateUpgrade::Malformed(returned, reason) => {
            assert_eq!(returned, original);
            assert!(reason.0.contains("nonexistent"), "{reason}");
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// Asking to go backward (`target` behind `source`) has no rule to apply -- this trait
/// only ever upgrades forward.
#[test]
fn no_entry_when_target_is_behind_source() {
    let provider = VanillaContent;
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original.clone(), V1_21_0, V1_20_10) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

/// Neither `source` nor `target` being a pinned version is `NoEntry` -- there is no
/// generated schema data to walk from or to.
#[test]
fn no_entry_when_neither_version_is_pinned() {
    let provider = VanillaContent;
    let unpinned_a = BlockDef::pack_version([9, 9, 9, 0]);
    let unpinned_b = BlockDef::pack_version([9, 9, 10, 0]);
    let original = state("minecraft:acacia_fence");

    match provider.upgrade_block_state(original.clone(), unpinned_a, unpinned_b) {
        StateUpgrade::NoEntry(returned) => assert_eq!(returned, original),
        other => panic!("expected NoEntry, got {other:?}"),
    }
}

/// The ladder continues past `v1.26.40.05`: a block that persists walks on to the newest
/// pinned stable release, one step per adjacent pair of pinned versions.
#[test]
fn walks_on_through_v1_26_50_to_v1_26_60() {
    let provider = VanillaContent;
    for target in [V1_26_50, V1_26_60] {
        match provider.upgrade_block_state(state("minecraft:acacia_fence"), V1_26_40, target) {
            StateUpgrade::Upgraded(upgraded) => {
                assert_eq!(upgraded.version, target);
                assert_eq!(upgraded.block.name, "minecraft:acacia_fence");
            }
            other => panic!("expected Upgraded, got {other:?}"),
        }
    }
}

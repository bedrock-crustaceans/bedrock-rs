//! The names of the game's lists are those 1.26.60 registers. The Drop 4 files of the shipped
//! install name a block, a potion, a level sound event and a particle type that older lists
//! lacked; each resolves. The entity, recipe and feature are the shipped files
//! (`behavior_packs/experimental_y_2026_drop_4`); the two items are written here in JSON (the
//! install ships them as MCB) with the block their `block_placer` names.

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics};

use super::common::link;

const ICE_BALL: &str = include_str!("../fixtures/link/drop_4/entities/ice_ball.json");
const BREW: &str = include_str!("../fixtures/link/drop_4/recipes/brew_awkward_iceball_potion.json");
const CRYSTAL_FEATURE: &str =
    include_str!("../fixtures/link/drop_4/features/ice_crystal_single_block_feature.json");

/// An item `minecraft:<name>` at 1.26.50 that places the block `minecraft:<block>`.
fn placer(name: &str, block: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.50", "minecraft:item": {{"description": {{"identifier": "demo:{name}"}}, "components": {{"minecraft:block_placer": {{"block": "minecraft:{block}"}}}}}}}}"#
    )
}

/// The names the findings of `found` are about: the reference's name, or the text of a finding
/// without one.
fn named(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .map(|finding| match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference { id, .. }) => id.clone(),
            _ => finding.code.to_string(),
        })
        .collect()
}

#[test]
fn the_drop_4_names_of_the_1_26_60_binary_resolve() {
    let crystal = placer("ice_crystal", "ice_crystal");
    let icicle = placer("icicle", "icicle");
    let found = link(
        &[
            ("entities/ice_ball.json", ICE_BALL),
            ("recipes/brew_awkward_iceball_potion.json", BREW),
            (
                "features/ice_crystal_single_block_feature.json",
                CRYSTAL_FEATURE,
            ),
            ("items/ice_crystal.json", &crystal),
            ("items/icicle.json", &icicle),
        ],
        &[],
    );
    let names = named(&found);
    for name in [
        "ice_ball.break",
        "iceballbreak",
        "minecraft:potion_type:freezing",
        "minecraft:ice_crystal",
        "minecraft:icicle",
    ] {
        assert!(
            !names.iter().any(|said| said.contains(name)),
            "{name}: {found:#?}"
        );
    }
    // A name the game does not register is still told: the check runs.
    let found = link(&[("items/x.json", &placer("x", "ice_crystals"))], &[]);
    assert!(
        named(&found)
            .iter()
            .any(|said| said.contains("minecraft:ice_crystals")),
        "{found:#?}"
    );
}

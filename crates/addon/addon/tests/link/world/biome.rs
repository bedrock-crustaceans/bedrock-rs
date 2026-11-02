//! 20 W12 (the biomes a biome turns into), W13 (the biomes it replaces) and W14 (its blocks).
//! A finding is an error: the game does not register such a biome. The pass tells it and
//! leaves the registry as loaded.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::BiomeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with, stacks};
use super::{finding, said, stacks_with_game, texts};

const MOOR: &str = "biomes/moor.json";

/// A biome `id` with the members `components`.
fn biome(id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:biome": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

fn error(code: &'static str, member: &str, kind: &'static str, id: &str) -> super::Said {
    finding(
        code,
        Severity::Error,
        MOOR,
        &format!("minecraft:biome/components/{member}"),
        kind,
        id,
    )
}

// ---- W12 -------------------------------------------------------------------------------------

/// `minecraft:overworld_generation_rules` naming `name` in every form of a transformation: one
/// name, a list of names, a list of `[name, weight]` pairs.
fn transformations(name: &str) -> String {
    format!(
        r#""minecraft:overworld_generation_rules": {{"hills_transformation": "{name}", "mutate_transformation": ["minecraft:plains", "{name}"], "river_transformation": [["minecraft:plains", 1], ["{name}", 2]], "shore_transformation": "{name}"}}"#
    )
}

#[test]
fn w12_a_biome_a_pack_defines_or_the_game_has_resolves() {
    let fen = biome("demo:fen", "");
    for name in ["demo:fen", "minecraft:forest", "forest"] {
        let moor = biome("demo:moor", &transformations(name));
        let found = link(&[(MOOR, moor.as_str()), ("biomes/fen.json", &fen)], &[]);
        assert_eq!(said(&found, &["biome"]), [], "{name}");
    }
    let moor = biome("demo:moor", &transformations("demo:far"));
    let found = link_with(
        &[(MOOR, moor.as_str())],
        &[],
        hosted(&[("biome", "demo:far"), ("biome", "minecraft:plains")]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["biome"]), []);
}

#[test]
fn w12_a_biome_nothing_defines_is_an_error_at_each_member_with_vanillas_text() {
    let moor = biome("demo:moor", &transformations("demo:gone"));
    let set = stacks(&[(MOOR, moor.as_str())], &[], LoadOptions::default());
    let all = set.link();
    // Beside each, the game's validator line, a content-log error.
    let lines: Vec<String> = all
        .iter()
        .filter(|f| crate::common::is_second_validator_line(f, &all))
        .map(|f| format!("{:?} {}", f.severity, f.code))
        .collect();
    assert_eq!(
        lines[0],
        "Error -> components -> minecraft:overworld_generation_rules -> hills_transformation:  Missing referenced asset demo:gone"
    );
    let found = crate::common::family_findings(&all);
    assert_eq!(lines.len(), said(&found, &["biome"]).len());
    let rules = "minecraft:overworld_generation_rules";
    assert_eq!(
        said(&found, &["biome"]),
        [
            error(
                "UnresolvedReference",
                &format!("{rules}/hills_transformation"),
                "biome",
                "demo:gone"
            ),
            error(
                "UnresolvedReference",
                &format!("{rules}/mutate_transformation/1"),
                "biome",
                "demo:gone"
            ),
            error(
                "UnresolvedReference",
                &format!("{rules}/river_transformation/1/0"),
                "biome",
                "demo:gone"
            ),
            error(
                "UnresolvedReference",
                &format!("{rules}/shore_transformation"),
                "biome",
                "demo:gone"
            ),
        ]
    );
    assert_eq!(
        texts(&found, "biome")[0],
        "No biome found with name 'demo:gone'"
    );
    // The pass tells and prunes nothing: the biome is in the registry as it was loaded.
    assert!(set.behavior.biomes().contains(&BiomeId::parse("demo:moor")));
    // An error is no lint.
    assert_eq!(
        said(
            &crate::common::family_findings(&set.link_with(&LinkOptions {
                lints: false,
                ..Default::default()
            })),
            &["biome"]
        )
        .len(),
        4
    );
}

// ---- W13 -------------------------------------------------------------------------------------

/// A biome that replaces `targets` (the elements of the JSON array) in the overworld.
fn replacing(targets: &str) -> String {
    biome(
        "demo:moor",
        &format!(
            r#""minecraft:replace_biomes": {{"replacements": [{{"dimension": "minecraft:overworld", "targets": [{targets}], "amount": 0.5, "noise_frequency_scale": 1.0}}]}}"#
        ),
    )
}

const TARGETS: &str = "minecraft:replace_biomes/replacements/0/targets";

/// The game writes a target without a namespace; the same biome with `minecraft:` in front is
/// the same biome, and neither spelling is a finding.
#[test]
fn w13_a_target_names_a_biome_of_the_game_with_or_without_its_namespace_open_o4() {
    let moor = replacing(r#""plains", "minecraft:plains", "forest""#);
    let found = link(&[(MOOR, moor.as_str())], &[]);
    assert_eq!(said(&found, &["biome"]), []);
    // A host is asked for the same canonical name for both spellings.
    let moor = replacing(r#""far_lands", "minecraft:far_lands""#);
    let found = link_with(
        &[(MOOR, moor.as_str())],
        &[],
        hosted(&[("biome", "minecraft:far_lands")]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["biome"]), []);
}

#[test]
fn w13_a_target_nothing_defines_is_an_error_whatever_its_spelling_open_o4() {
    let moor = replacing(r#""plains", "nowhere", "minecraft:nowhere", "demo:gone""#);
    // The game's validator line beside each is `linked_asset.rs`'s.
    let found = crate::common::family_findings(&link(&[(MOOR, moor.as_str())], &[]));
    assert_eq!(
        said(&found, &["biome"]),
        [
            error(
                "UnresolvedReference",
                &format!("{TARGETS}/1"),
                "biome",
                "nowhere"
            ),
            error(
                "UnresolvedReference",
                &format!("{TARGETS}/2"),
                "biome",
                "minecraft:nowhere"
            ),
            error(
                "UnresolvedReference",
                &format!("{TARGETS}/3"),
                "biome",
                "demo:gone"
            ),
        ]
    );
    assert_eq!(
        texts(&found, "biome")[0],
        "No biome found with name: 'nowhere'"
    );
}

/// Only a biome of the game can be replaced: a biome a pack of the world adds cannot, while a
/// biome of the game that a pack of the world redefines, and a biome a pack of the game
/// defines, can.
#[test]
fn w13_a_target_that_is_no_biome_of_the_game_is_an_error() {
    let (fen, plains, deep) = (
        biome("demo:fen", ""),
        biome("minecraft:plains", ""),
        biome("minecraft:deep", ""),
    );
    let moor = replacing(r#""demo:fen", "plains", "deep""#);
    let set = stacks_with_game(
        &[
            (MOOR, moor.as_str()),
            ("biomes/fen.json", fen.as_str()),
            ("biomes/plains.json", plains.as_str()),
        ],
        &[("biomes/deep.json", deep.as_str())],
        LoadOptions::default(),
    );
    assert_eq!(set.behavior.biomes().len(), 4);
    let found = set.link();
    assert_eq!(
        said(&found, &["biome"]),
        [error(
            "InvalidReference",
            &format!("{TARGETS}/0"),
            "biome",
            "demo:fen"
        )]
    );
    assert_eq!(
        texts(&found, "biome"),
        ["Must replace vanilla biomes only. Attempting to use biome: 'demo:fen'"]
    );
}

// ---- W14 -------------------------------------------------------------------------------------

/// Components that name `block` in a member of every component of the row, with the member's
/// path below `components` and the label vanilla's text gives it.
#[cfg(feature = "block")]
fn naming(block: &str) -> (String, Vec<(&'static str, &'static str)>) {
    let components = format!(
        r#""minecraft:mountain_parameters": {{"steep_material_adjustment": {{"material": "{block}"}}}}, "minecraft:surface_material_adjustments": {{"adjustments": [{{"materials": {{"top_material": "{block}", "sea_material": {{"name": "{block}"}}}}}}]}}, "minecraft:surface_builder": {{"builder": {{"type": "minecraft:capped", "floor_materials": ["minecraft:netherrack", "{block}"], "ceiling_materials": ["minecraft:netherrack"], "sea_material": "minecraft:lava", "foundation_material": "{block}"}}}}, "minecraft:subsurface_builder": {{"builder": {{"type": "minecraft:noise_gradient", "noise_descriptor": {{"name": "demo:gradient", "first_octave": -5, "amplitudes": [1.0]}}, "non_replaceable_blocks": ["{block}"], "noise_block_specifiers": [{{"range": {{"min": 0.0, "max": 0.4}}, "block": "{block}"}}]}}}}"#
    );
    let members = vec![
        (
            "minecraft:mountain_parameters/steep_material_adjustment/material",
            "material",
        ),
        (
            "minecraft:surface_material_adjustments/adjustments/0/materials/top_material",
            "top_material",
        ),
        (
            "minecraft:surface_material_adjustments/adjustments/0/materials/sea_material/name",
            "sea_material",
        ),
        (
            "minecraft:surface_builder/builder/floor_materials/1",
            "floor_materials",
        ),
        (
            "minecraft:surface_builder/builder/foundation_material",
            "foundation_material",
        ),
        (
            "minecraft:subsurface_builder/builder/non_replaceable_blocks/0",
            "non_replaceable_blocks",
        ),
        (
            "minecraft:subsurface_builder/builder/noise_block_specifiers/0/block",
            "noise_block_specifier -> block",
        ),
    ];
    (components, members)
}

#[cfg(feature = "block")]
#[test]
fn w14_a_block_a_pack_defines_or_the_game_has_resolves() {
    let lamp = super::block("demo:lamp");
    for block in ["demo:lamp", "minecraft:stone"] {
        let moor = biome("demo:moor", &naming(block).0);
        let set = stacks(
            &[(MOOR, moor.as_str()), ("blocks/lamp.json", &lamp)],
            &[],
            LoadOptions::default(),
        );
        assert_eq!(
            set.behavior.biomes().len(),
            1,
            "{:#?}",
            set.behavior.biomes().rejected()
        );
        assert_eq!(said(&set.link(), &["block"]), [], "{block}");
    }
    let moor = biome("demo:moor", &naming("demo:lamp").0);
    let found = link_with(
        &[(MOOR, moor.as_str())],
        &[],
        hosted(&[
            ("block", "demo:lamp"),
            ("block", "minecraft:netherrack"),
            ("block", "minecraft:lava"),
        ]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block"]), []);
}

#[cfg(feature = "block")]
#[test]
fn w14_a_block_nothing_defines_is_an_error_at_each_member_with_vanillas_text() {
    let (components, members) = naming("demo:lamp");
    let moor = biome("demo:moor", &components);
    // The game's validator lines (at the member, a block object's at the object) are
    // `linked_asset.rs`'s.
    let found = crate::common::without_validator_lines(&link(&[(MOOR, moor.as_str())], &[]));
    let expected: Vec<_> = members
        .iter()
        .map(|(member, _)| error("UnresolvedReference", member, "block", "demo:lamp"))
        .collect();
    assert_eq!(said(&found, &["block"]), expected);
    let expected: Vec<String> = members
        .iter()
        .map(|(_, label)| format!("'{label}' block with name 'demo:lamp', failed to resolve."))
        .collect();
    assert_eq!(texts(&found, "block"), expected);
}

/// A state of a block of the stack that the block does not have fails the block as a name
/// nothing defines does.
#[cfg(feature = "block")]
#[test]
fn w14_a_state_the_block_does_not_have_is_an_error() {
    let lamp = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}}, "components": {}}}"#;
    let moor = biome(
        "demo:moor",
        r#""minecraft:mountain_parameters": {"steep_material_adjustment": {"material": {"name": "demo:lamp", "states": {"demo:dim": true, "demo:lit": true}}}}"#,
    );
    let found = link(&[(MOOR, moor.as_str()), ("blocks/lamp.json", lamp)], &[]);
    assert_eq!(
        said(&found, &["block", "block state"]),
        [error(
            "UnresolvedReference",
            "minecraft:mountain_parameters/steep_material_adjustment/material/states/demo:dim",
            "block state",
            "demo:dim"
        )]
    );
}

/// Without the `block` module nobody knows a pack's blocks.
#[cfg(not(feature = "block"))]
#[test]
fn w14_without_the_block_module_no_block_of_a_biome_is_reported() {
    let moor = biome(
        "demo:moor",
        r#""minecraft:mountain_parameters": {"steep_material_adjustment": {"material": "demo:lamp"}}"#,
    );
    let found = link(&[(MOOR, moor.as_str())], &[]);
    assert_eq!(said(&found, &["block"]), []);
}

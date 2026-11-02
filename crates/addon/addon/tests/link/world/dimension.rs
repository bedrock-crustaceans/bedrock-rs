//! 20 W15: the default biome of a dimension.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{finding, said};

const OVERWORLD: &str = "dimensions/demo_moon.json";

/// A custom dimension at format 1.26.50 with `members` before its generator: a vanilla
/// dimension takes no default biome in 1.26.60.
fn overworld(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.50", "minecraft:dimension": {{"description": {{"identifier": "demo:moon"}}, "components": {{"minecraft:dimension_height": {{"min_y": -64, "height_range": 384}}, {members}"minecraft:generation": {{"generator_type": "void"}}}}}}}}"#
    )
}

fn default_biome(biome: &str) -> String {
    overworld(&format!(
        r#""minecraft:default_biome": {{"biome": "{biome}"}}, "#
    ))
}

#[cfg(feature = "biome")]
#[test]
fn w15_a_default_biome_a_pack_defines_or_the_game_has_resolves() {
    let fen = r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:fen"}, "components": {}}}"#;
    for biome in ["demo:fen", "minecraft:plains"] {
        let text = default_biome(biome);
        let found = link(&[(OVERWORLD, text.as_str()), ("biomes/fen.json", fen)], &[]);
        assert_eq!(said(&found, &["biome"]), [], "{biome}");
    }
    // A dimension that names none takes the ocean: no reference of the document.
    let text = overworld("");
    let found = link_with(
        &[(OVERWORLD, text.as_str())],
        &[],
        hosted(&[("dimension", "minecraft:overworld")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["biome"]), []);
    let text = default_biome("demo:far");
    let found = link_with(
        &[(OVERWORLD, text.as_str())],
        &[],
        hosted(&[("biome", "demo:far")]),
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &["biome"]), []);
}

#[cfg(feature = "biome")]
#[test]
fn w15_a_default_biome_nothing_defines_is_an_information_and_a_lint() {
    let text = default_biome("demo:nowhere");
    let files = [(OVERWORLD, text.as_str())];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &["biome"]),
        [finding(
            "UnresolvedReference",
            Severity::Info,
            OVERWORLD,
            "minecraft:dimension/components/minecraft:default_biome/biome",
            "biome",
            "demo:nowhere"
        )]
    );
    let found = link_with(
        &files,
        &[],
        LoadOptions::default(),
        &LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(said(&found, &["biome"]), []);
}

/// Without the `biome` module nobody knows a pack's biomes.
#[cfg(not(feature = "biome"))]
#[test]
fn w15_without_the_biome_module_no_default_biome_is_reported() {
    let text = default_biome("demo:nowhere");
    let found = link(&[(OVERWORLD, text.as_str())], &[]);
    assert_eq!(said(&found, &["biome"]), []);
}

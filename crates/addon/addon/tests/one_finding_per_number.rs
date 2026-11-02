#![allow(clippy::pedantic, missing_docs)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "full")]
//! A number no double holds is told once: where the member that holds the number
//! gives a finding of its own at the number (it does not take it, it reads it at a width of
//! its own, it refuses the file), the warning of the clamp ("read as the largest finite
//! double") is not added: its sentence would be false, and the number told twice.
//!
//! Each test puts `1e999` into one typed member and expects at the number's path the
//! member's own finding and nothing else, by code and severity (a manifest's
//! `format_version` is malformed; its `version` is refused whole, at the version).

#[allow(unused_imports)]
#[path = "terrain_texture/common.rs"]
mod common;

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// The findings of one file read through its type's public entry, loaded or rejected.
fn findings<T: Content>(path: &'static str, text: &str) -> Vec<Diagnostic> {
    let cx = LoadContext::detached();
    match T::parse(&PackPath::from_static(path), text.as_bytes(), &cx) {
        Ok(document) => document.diagnostics().iter().cloned().collect(),
        Err(rejected) => rejected.diagnostics.iter().cloned().collect(),
    }
}

/// The variant name of a finding's code.
fn code(finding: &Diagnostic) -> String {
    format!("{:?}", finding.code)
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect()
}

/// Requires the findings at `at` among `found` to be `expected`, by code and severity: what
/// the member says of the number, without the clamp's warning beside it.
#[track_caller]
fn assert_told_once(found: &[Diagnostic], at: &str, expected: &[(&str, Severity)]) {
    let there: Vec<(String, Severity)> = found
        .iter()
        .filter(|finding| finding.location.json_path.to_string() == at)
        .map(|finding| (code(finding), finding.severity))
        .collect();
    let all: Vec<String> = found
        .iter()
        .map(|d| format!("{} {:?} at `{}`", code(d), d.severity, d.location.json_path))
        .collect();
    let expected: Vec<(String, Severity)> = expected
        .iter()
        .map(|(code, severity)| ((*code).to_owned(), *severity))
        .collect();
    assert_eq!(
        there, expected,
        "the number at `{at}` is told once, by its member; every finding: {all:#?}"
    );
}

/// One test: the type, its file path, the text and the number's path, the member's findings.
macro_rules! told_once {
    ($( $test:ident: $type:ty, $path:literal, $text:expr, $at:literal => $($code:literal $severity:ident),+; )*) => {
        $(
            #[test]
            fn $test() {
                let found = findings::<$type>($path, $text);
                assert_told_once(&found, $at, &[$(($code, Severity::$severity)),+]);
            }
        )*
    };
}

told_once! {
    // A number no double holds refuses the document: the one finding is the
    // `MinecraftDocumentInput`'s error.
    block_friction: bedrock_addon::families::block::Block, "blocks/x.json",
        r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 1e999}}}"#,
        "" => "MalformedInput" Error; // a `MinecraftDocumentInput` refuses the number
    block_light_emission: bedrock_addon::families::block::Block, "blocks/x.json",
        r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:light_emission": 1e999}}}"#,
        "" => "MalformedInput" Error; // a `MinecraftDocumentInput` refuses the number
    block_seconds_to_destroy: bedrock_addon::families::block::Block, "blocks/x.json",
        r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:destructible_by_mining": {"seconds_to_destroy": 1e999}}}}"#,
        "" => "MalformedInput" Error; // a `MinecraftDocumentInput` refuses the number
    item_max_stack_size: bedrock_addon::families::item::Item, "items/x.json",
        r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 1e999, "minecraft:glint": true}}}"#,
        "" => "MalformedInput" Error, "MalformedInput" Error; // refused
    // A plain float member: the number is read at the width of a float, and the one
    // finding is the clamp's information; a number no double holds refuses the
    // item (a `MinecraftDocumentInput`).
    item_cooldown_duration_beyond_a_float: bedrock_addon::families::item::Item, "items/x.json",
        r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:cooldown": {"category": "demo", "duration": 1e39}}}}"#,
        "minecraft:item/components/minecraft:cooldown/duration" => "InvalidValue" Info;
    item_cooldown_duration_beyond_a_double: bedrock_addon::families::item::Item, "items/x.json",
        r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:cooldown": {"category": "demo", "duration": 1e999}}}}"#,
        "" => "MalformedInput" Error, "MalformedInput" Error; // refused
    spawn_weight_default: bedrock_addon::families::spawn_rules::SpawnRules, "spawn_rules/x.json",
        r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "demo:x", "population_control": "animal"}, "conditions": [{"minecraft:spawns_on_surface": {}, "minecraft:weight": {"default": 1e999}}]}}"#,
        "minecraft:spawn_rules/conditions/0/minecraft:weight/default" => "InvalidType" Info;
    loot_rolls: bedrock_addon::families::loot::LootTable, "loot_tables/x.json",
        r#"{"pools": [{"rolls": 1e999, "entries": [{"type": "item", "name": "minecraft:stone"}]}]}"#,
        "pools/0/rolls" => "InvalidType" Info;
    recipe_result_count: bedrock_addon::families::recipe::Recipe, "recipes/x.json",
        r#"{"format_version": "1.12.0", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:x"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stone"], "result": {"item": "minecraft:stone", "count": 1e999}}}"#,
        "minecraft:recipe_shapeless/result/count" => "InvalidType" Info;
    biome_climate_temperature: bedrock_addon::families::biome::Biome, "biomes/x.json",
        r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:x"}, "components": {"minecraft:climate": {"temperature": 1e999}}}}"#,
        "" => "MalformedInput" Error; // refused
    // A `format_version` that gives no version (the clamped double's text is none) is
    // "Malformed format version" before any reader runs: one error, no clamp warning.
    manifest_format_version: bedrock_addon::manifest::Manifest, "manifest.json",
        r#"{"format_version": 1e999, "header": {"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]}"#,
        "format_version" => "MalformedFormatVersion" Error;
    terrain_texture_mip_fade_amount: bedrock_addon::families::texture_atlas::TextureAtlas, "textures/terrain_texture.json",
        r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": "textures/blocks/dirt", "mip_fade_amount": 1e999}}}"#,
        "texture_data/dirt/mip_fade_amount" => "InvalidValue" Info;
    atlas_padding: bedrock_addon::families::texture_atlas::TextureAtlas, "textures/terrain_texture.json",
        r#"{"texture_name": "atlas.terrain", "padding": 1e999, "texture_data": {"dirt": {"textures": "textures/blocks/dirt"}}}"#,
        "padding" => "InvalidValue" Info;
    flipbook_ticks_per_frame: bedrock_addon::families::flipbooks::Flipbooks, "textures/flipbook_textures.json",
        r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 1e999}]"#,
        "0/ticks_per_frame" => "InvalidValue" Info;
    music_min_delay: bedrock_addon::families::music::MusicDefinitions, "sounds/music_definitions.json",
        r#"{"game": {"event_name": "music.game", "min_delay": 1e999, "max_delay": 180.5, "replace": false}, "format_version": "1.0.0"}"#,
        "game/min_delay" => "InvalidValue" Info;
}

/// A manifest refuses a version with an element that is no integer at the version itself:
/// that error is the word about the number in it, and nothing is said at the element.
#[test]
fn manifest_header_version_patch() {
    let found = findings::<bedrock_addon::manifest::Manifest>(
        "manifest.json",
        r#"{"format_version": 2, "header": {"name": "x", "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 1e999], "min_engine_version": [1, 21, 0]}, "modules": [{"type": "data", "uuid": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]}"#,
    );
    let told: Vec<(String, Severity, String)> = found
        .iter()
        .map(|d| (code(d), d.severity, d.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [(
            "WrongTypeProperty".to_owned(),
            Severity::Error,
            "header/version".to_owned()
        )]
    );
}

/// The flipbook list of a stack: each pack's copy is read on its own and the list is merged
/// entry by entry; the member's finding and no clamp warning beside it, for each copy, as for
/// the file read on its own.
#[test]
fn flipbook_ticks_per_frame_in_a_stack() {
    const FILE: &str = "textures/flipbook_textures.json";
    let top: &[(&str, &[u8])] = &[(
        FILE,
        br#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 1e999}]"#,
    )];
    let bottom: &[(&str, &[u8])] = &[(
        FILE,
        br#"[{"flipbook_texture": "textures/blocks/y", "atlas_tile": "y"}, {"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": -1e999}]"#,
    )];
    let stack = common::stack_of_files(&[top, bottom]);
    let found: Vec<Diagnostic> = stack.flipbooks().diagnostics().cloned().collect();
    assert_told_once(
        &found,
        "0/ticks_per_frame",
        // the flipbook load lints are informations
        &[("InvalidValue", Severity::Info)],
    );
    // The lower copy's number is below 1 as an integer: its two findings, and no third.
    assert_told_once(
        &found,
        "1/ticks_per_frame",
        &[
            ("InvalidValue", Severity::Info),
            ("ClampedValue", Severity::Info),
        ],
    );
}

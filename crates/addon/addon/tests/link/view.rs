//! What a rule does with a miss of a kind whose definitions are a registry of the game that
//! an add-on does not carry (`material`, `atlas key`), by the view the pass has of the kind
//! (a miss is judged only where the stack or a host has a view of the kind):
//!
//! - no source of the kind on the stack and no host: nothing is said;
//! - a source, but none from a pack the game ships, and no host: the miss is told as an
//!   information (the name may be the game's), with the row's code and detail;
//! - a pack the game ships provides a source, or a host answers: the row's severity.
//!
//! One world per case with a miss in each row: a particle's material (20 R38), a client
//! entity's material (R1), a block's tile (B9), a `blocks.json` tile (R28), an item's icon
//! (R32) and a flipbook's tile (R34).
//!
//! The whole view needs a source of the game's pack that was read into the stack: one that
//! is listed and not resident, an atlas file that builds nothing and a stray file below
//! `materials/` are no view.

#![cfg(all(
    feature = "block",
    feature = "blocks_json",
    feature = "item",
    feature = "entity",
    feature = "client_entity",
    feature = "particle",
    feature = "material",
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "flipbooks"
))]

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, Pack, PackOrigin};

use super::common::{behavior_pack, hosted, resource_pack, stacks_from, unloaded_pack};

const PARTICLE: &str = "particles/a.json";
const CLIENT_ENTITY: &str = "entity/ghost.entity.json";
const BLOCK: &str = "blocks/lamp.json";
const BLOCKS_JSON: &str = "blocks.json";
const ITEM: &str = "items/thing.json";
const FLIPBOOKS: &str = "textures/flipbook_textures.json";

/// The behaviour files: a block and an item, each naming a tile nothing has, and the entity
/// of the client entity.
fn behavior() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            BLOCK,
            r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {"minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "no_tile"}}}}}"#,
        ),
        (
            ITEM,
            r#"{"format_version": "1.21.80", "minecraft:item": {"description": {"identifier": "demo:thing"}, "components": {"minecraft:icon": "no_icon"}}}"#,
        ),
        (
            "entities/ghost.json",
            r#"{"format_version": "1.21.0", "minecraft:entity": {"description": {"identifier": "demo:ghost", "is_spawnable": true, "is_summonable": true, "is_experimental": false}, "components": {}}}"#,
        ),
    ]
}

/// The resource files that name: an effect and a client entity naming a material nothing
/// has, and a `blocks.json` entry naming a tile nothing has.
fn naming() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            PARTICLE,
            r#"{"format_version": "1.26.10", "particle_effect": {"description": {"identifier": "demo:a", "basic_render_parameters": {"material": "no_material", "texture": "textures/particle/mine"}}, "components": {"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1}}}}"#,
        ),
        ("textures/particle/mine.png", "no image is read"),
        (
            CLIENT_ENTITY,
            r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost", "materials": {"default": "no_material"}}}}"#,
        ),
        (
            BLOCKS_JSON,
            r#"{"format_version": "1.21.40", "demo:lamp": {"textures": "no_tile"}}"#,
        ),
        (
            FLIPBOOKS,
            r#"[{"flipbook_texture": "textures/blocks/lava", "atlas_tile": "no_tile"}]"#,
        ),
    ]
}

/// A source of each kind, defining names with the prefix `own`: a material list with its
/// file, a terrain atlas and an item atlas.
fn sources(own: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "materials/common.json",
            r#"[{"path": "materials/my.material"}]"#.to_owned(),
        ),
        (
            "materials/my.material",
            format!(r#"{{"materials": {{"version": "1.0.0", "{own}_material": {{}}}}}}"#),
        ),
        (
            "textures/terrain_texture.json",
            format!(
                r#"{{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {{"{own}_tile": {{"textures": "textures/blocks/a"}}}}}}"#
            ),
        ),
        (
            "textures/item_texture.json",
            format!(
                r#"{{"resource_pack_name": "t", "texture_name": "atlas.items", "texture_data": {{"{own}_icon": {{"textures": "textures/items/a"}}}}}}"#
            ),
        ),
    ]
}

/// The link findings of a world whose add-on resource pack (`rp 2`, on top) names the
/// misses and has sources of its own when `own`, above a pack the game ships (`rp 3`) that
/// has sources when `game`, with a host that knows nothing of these names when `host`.
fn linked(own: bool, game: bool, host: bool) -> Diagnostics {
    let game_sources = if game { sources("game") } else { Vec::new() };
    let shipped: Vec<(&'static str, &str)> = game_sources
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    linked_over(own, resource_pack(3, &shipped), host, true)
}

/// [`linked`] above the pack `game`, registered as one the game ships; with the lints when
/// `lints`.
fn linked_over(own: bool, game: Pack, host: bool, lints: bool) -> Diagnostics {
    let own_sources = if own { sources("own") } else { Vec::new() };
    let mut addon = naming();
    addon.extend(
        own_sources
            .iter()
            .map(|(path, text)| (*path, text.as_str())),
    );
    let options = if host {
        // The host is asked about every kind and believed: it has the entity the files name.
        hosted(&[("entity", "demo:ghost")])
    } else {
        LoadOptions::default()
    };
    let set = stacks_from(
        vec![(behavior_pack(1, &behavior()), PackOrigin::User)],
        vec![
            (resource_pack(2, &addon), PackOrigin::User),
            (game, PackOrigin::Package),
        ],
        options,
    );
    assert_eq!(set.behavior.blocks().len(), 1, "the block loads");
    assert_eq!(set.behavior.items().len(), 1, "the item loads");
    assert_eq!(set.resource.particles().len(), 1, "the effect loads");
    assert_eq!(
        set.resource.client_entities().len(),
        1,
        "the client entity loads"
    );
    set.link_with(&LinkOptions {
        lints,
        ..Default::default()
    })
}

/// The severity of each `UnresolvedReference` of `kind` named `name` in `file`.
fn told(found: &Diagnostics, file: &str, kind: &str, name: &str) -> Vec<Severity> {
    found
        .iter()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == file)
        })
        .filter(|finding| {
            matches!(
                finding.detail.as_deref(),
                Some(DiagnosticDetail::Reference { kind: told, id, .. })
                    if *told == kind && id == name
            )
        })
        .map(|finding| finding.severity)
        .collect()
}

/// The three cases for one row: the miss `name` of `kind` in `file`, which a whole view tells
/// at `whole` (a block tile: the game's link pass logs it at the content log's error level).
fn three_cases_at(file: &str, kind: &str, name: &str, whole: Severity) {
    // (a) No source and no host: nothing is said of the kind.
    assert_eq!(
        told(&linked(false, false, false), file, kind, name),
        [],
        "no view"
    );
    // (c) A source of the add-on's own alone: the name may be the game's.
    assert_eq!(
        told(&linked(true, false, false), file, kind, name),
        [Severity::Info],
        "the add-on's own source"
    );
    // (b) A pack the game ships provides a source, with or without the add-on's own.
    for own in [false, true] {
        assert_eq!(
            told(&linked(own, true, false), file, kind, name),
            [whole],
            "a source of the game's pack (own source: {own})"
        );
    }
    // (b) A host answers for the kind, with a source or without one.
    for own in [false, true] {
        assert_eq!(
            told(&linked(own, false, true), file, kind, name),
            [whole],
            "a host (own source: {own})"
        );
    }
}

#[test]
fn r38_a_particle_material_is_judged_by_the_view_of_the_materials() {
    // The material is looked up at draw time without a line: ours, an information in every
    // view that judges at all.
    three_cases_at(PARTICLE, "material", "no_material", Severity::Info);
}

#[test]
fn r1_a_client_entity_material_is_judged_by_the_view_of_the_materials() {
    // A whole view tells the game's level-3 line (a kept error).
    three_cases_at(CLIENT_ENTITY, "material", "no_material", Severity::Error);
}

#[test]
fn b9_a_block_tile_is_judged_by_the_view_of_the_terrain_atlas() {
    three_cases_at(BLOCK, "atlas key", "no_tile", Severity::Error);
}

#[test]
fn r28_a_blocks_json_tile_is_judged_by_the_view_of_the_terrain_atlas() {
    // A tile the atlas lacks is the atlas's developer-log line, an information in every view
    // that judges at all.
    three_cases_at(BLOCKS_JSON, "atlas key", "no_tile", Severity::Info);
}

#[test]
fn r32_an_item_icon_is_judged_by_the_view_of_the_item_atlas() {
    // A whole view tells the validator's level-3 line (a kept error).
    three_cases_at(ITEM, "atlas key", "no_icon", Severity::Error);
}

/// R34, the tile half: a flipbook's `atlas_tile` is a tile of the terrain atlas like a
/// block's, and is judged by the same view. (The family's checker compares nothing without
/// an atlas on the stack, with a host or without one.)
#[test]
fn r34_a_flipbook_tile_is_judged_by_the_view_of_the_terrain_atlas() {
    let told = |found: &Diagnostics| told(found, FLIPBOOKS, "atlas key", "no_tile");
    assert_eq!(told(&linked(false, false, false)), [], "no view");
    assert_eq!(told(&linked(false, false, true)), [], "a host, no atlas");
    assert_eq!(
        told(&linked(true, false, false)),
        [Severity::Info],
        "the add-on's own atlas"
    );
    // At index 0 the game animates its `missing` tile without a word: an information in a
    // whole view too.
    for own in [false, true] {
        assert_eq!(
            told(&linked(own, true, false)),
            [Severity::Info],
            "the atlas of the game's pack (own atlas: {own})"
        );
    }
    assert_eq!(
        told(&linked(true, false, true)),
        [Severity::Info],
        "a host and the add-on's own atlas"
    );
    // An information is a lint: with the lints off the partial view says nothing.
    let game = resource_pack(3, &[]);
    assert_eq!(told(&linked_over(true, game, false, false)), []);
}

/// The rows about a tile or an icon, and the rows about a material.
const TILE_ROWS: [(&str, &str, &str); 4] = [
    (BLOCK, "atlas key", "no_tile"),
    (BLOCKS_JSON, "atlas key", "no_tile"),
    (FLIPBOOKS, "atlas key", "no_tile"),
    (ITEM, "atlas key", "no_icon"),
];
const MATERIAL_ROWS: [(&str, &str, &str); 2] = [
    (PARTICLE, "material", "no_material"),
    (CLIENT_ENTITY, "material", "no_material"),
];

/// Each row of `rows` tells its miss with exactly `expected`.
#[track_caller]
fn each(found: &Diagnostics, rows: &[(&str, &str, &str)], expected: &[Severity], what: &str) {
    for (file, kind, name) in rows {
        assert_eq!(told(found, file, kind, name), expected, "{what}: {file}");
    }
}

/// An atlas file of the game's pack that builds nothing (no `texture_name`) was not read
/// into the stack: it is no view of the game's tiles. Alone nothing is judged; beside the
/// add-on's own atlas the view is the add-on's, partial.
#[test]
fn an_atlas_of_the_games_pack_that_does_not_build_is_no_whole_view() {
    let unbuilt = || {
        resource_pack(
            3,
            &[
                (
                    "textures/terrain_texture.json",
                    r#"{"texture_data": {"game_tile": {"textures": "textures/blocks/a"}}}"#,
                ),
                (
                    "textures/item_texture.json",
                    r#"{"texture_data": {"game_icon": {"textures": "textures/items/a"}}}"#,
                ),
            ],
        )
    };
    each(
        &linked_over(false, unbuilt(), false, true),
        &TILE_ROWS,
        &[],
        "alone",
    );
    each(
        &linked_over(true, unbuilt(), false, true),
        &TILE_ROWS,
        &[Severity::Info],
        "beside the add-on's own atlas",
    );
}

/// A pack of the game that is opened and not loaded lists its sources without their bytes:
/// nobody read the game's registry, so there is no whole view, and each such file is told
/// once as not loaded so that the caller knows why nothing was judged.
#[test]
fn a_source_of_the_games_pack_that_is_not_resident_is_no_whole_view_and_is_told_once() {
    let listed = || {
        let files = sources("game");
        let files: Vec<(&'static str, &[u8])> = files
            .iter()
            .map(|(path, text)| (*path, text.as_bytes()))
            .collect();
        unloaded_pack(3, "resources", &files)
    };
    let alone = linked_over(false, listed(), false, true);
    each(&alone, &TILE_ROWS, &[], "alone");
    each(&alone, &MATERIAL_ROWS, &[], "alone");
    let beside = linked_over(true, listed(), false, true);
    each(
        &beside,
        &TILE_ROWS,
        &[Severity::Info],
        "beside the add-on's own",
    );
    each(
        &beside,
        &MATERIAL_ROWS,
        &[Severity::Info],
        "beside the add-on's own",
    );
    // Each unread source of the game's pack is told once by the pass.
    for found in [&alone, &beside] {
        for file in [
            "textures/terrain_texture.json",
            "textures/item_texture.json",
            "materials/common.json",
            "materials/my.material",
        ] {
            let told = found
                .iter()
                .filter(|finding| finding.code.name() == "FileNotLoaded")
                .filter(|finding| finding.phase == Phase::Link)
                .filter(|finding| {
                    finding
                        .location
                        .path
                        .as_ref()
                        .is_some_and(|path| path.as_str() == file)
                })
                .count();
            assert_eq!(told, 1, "{file}");
        }
    }
}

/// A file below `materials/` of the game's pack that creates no material is no source of the
/// game's materials: beside the add-on's own list the view is partial.
#[test]
fn a_stray_file_below_materials_of_the_games_pack_is_no_source() {
    let stray = || resource_pack(3, &[("materials/notes.txt", "no material is here")]);
    each(
        &linked_over(false, stray(), false, true),
        &MATERIAL_ROWS,
        &[],
        "alone",
    );
    each(
        &linked_over(true, stray(), false, true),
        &MATERIAL_ROWS,
        &[Severity::Info],
        "beside the add-on's own list",
    );
}

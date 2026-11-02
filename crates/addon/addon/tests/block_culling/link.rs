//! Rows x01 to x06: the cross-checks against the
//! behaviour-pack blocks and the resource-pack geometry. They need both stacks, so they are
//! the link stage's: each test calls `StackSet::link()`.

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::document;
use super::pack_common::{bp, rp};

/// Text that lives as long as the test binary.
fn leak(text: String) -> &'static [u8] {
    Box::leak(text.into_boxed_str()).as_bytes()
}

/// A block `demo:lamp` whose `minecraft:geometry` is `geometry.lamp` with `members` added.
fn block(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:lamp"}}, "components": {{"minecraft:geometry": {{"identifier": "geometry.lamp"{members}}}, "minecraft:material_instances": {{"*": {{"texture": "lamp"}}}}}}}}}}"#
    )
}

/// `geometry.lamp`: the bone `body` with two cubes.
const GEOMETRY: &str = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.lamp", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}, {"origin": [1, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}]}]}]}"#;

// block's texture in every test that holds the whole pass below a warning)
/// The terrain atlas with the tile `lamp`.
const TERRAIN: &str = r#"{"texture_name": "atlas.terrain", "texture_data": {"lamp": {"textures": "textures/blocks/lamp"}}}"#;

/// The link findings of a world with the block `block` and, in its resource pack, the
/// geometry and the culling documents `culling`.
fn link(block: &str, culling: &[String]) -> Diagnostics {
    let behavior = bp(1).file("blocks/lamp.json", leak(block.to_owned()));
    let mut resource = rp(2)
        .file("models/blocks/lamp.geo.json", GEOMETRY.as_bytes())
        .file("textures/terrain_texture.json", TERRAIN.as_bytes());
    for (index, text) in culling.iter().enumerate() {
        let path: &'static str = Box::leak(format!("block_culling/c{index}.json").into_boxed_str());
        resource = resource.file(path, leak(text.clone()));
    }
    let (behavior, resource) = (behavior.open(), resource.open());
    let world = WorldPacks::parse(
        Some(
            format!(
                r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
                behavior.id().uuid
            )
            .as_bytes(),
        ),
        Some(
            format!(
                r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
                resource.id().uuid
            )
            .as_bytes(),
        ),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(behavior, PackOrigin::User);
    repository.add_pack(resource, PackOrigin::User);
    repository.stacks(&world, LoadOptions::default()).link()
}

/// The findings whose text is `text`.
fn with_text<'a>(d: &'a Diagnostics, text: &str) -> Vec<&'a Diagnostic> {
    d.iter().filter(|f| f.code.to_string() == text).collect()
}

/// A culling document `demo:lamp_culling` with `rule`.
fn culling(rule: &str) -> String {
    document("1.21.80", "demo:lamp_culling", &[rule])
}

const WHOLE_BODY: &str = r#"{"geometry_part": {"bone": "body"}, "direction": "up"}"#;

#[test]
fn a_culling_name_no_rules_document_has() {
    let found = link(
        &block(r#", "culling": "demo:nope""#),
        &[culling(WHOLE_BODY)],
    );
    let missing = with_text(
        &found,
        " -> components -> minecraft:geometry -> culling:  Missing referenced asset demo:nope",
    );
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(missing[0].code.name(), "UnresolvedReference");
    // The block loads and its schematic is not built; vanilla logs at level 3:
    // a kept error.
    assert_eq!(missing[0].severity, Severity::Error);
    assert!(missing[0].kept);
    // The lookup is exact and case-sensitive.
    let found = link(
        &block(r#", "culling": "demo:Lamp_culling""#),
        &[culling(WHOLE_BODY)],
    );
    assert_eq!(
        with_text(
            &found,
            " -> components -> minecraft:geometry -> culling:  Missing referenced asset demo:Lamp_culling"
        )
        .len(),
        1
    );
}

#[test]
fn an_empty_culling_name_is_no_reference() {
    let found = link(&block(r#", "culling": """#), &[culling(WHOLE_BODY)]);
    assert!(
        found
            .iter()
            .all(|f| !f.code.to_string().contains("Missing referenced asset")),
        "{found:#?}"
    );
    // A name that resolves is no finding either.
    let found = link(
        &block(r#", "culling": "demo:lamp_culling""#),
        &[culling(WHOLE_BODY)],
    );
    assert!(
        found.iter().all(|f| f.severity < Severity::Warning),
        "{found:#?}"
    );
}

#[cfg(feature = "geometry")]
#[test]
fn a_rule_names_a_bone_the_geometry_lacks() {
    let found = link(
        &block(r#", "culling": "demo:lamp_culling""#),
        &[culling(
            r#"{"geometry_part": {"bone": "ghost"}, "direction": "up"}"#,
        )],
    );
    let missing = with_text(&found, "Did not find any bones with name `ghost`");
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(missing[0].code.name(), "UnresolvedReference");
    assert_eq!(
        missing[0].severity,
        Severity::Error,
        "the rule is ignored after a level-3 line: kept"
    );
    assert!(missing[0].kept);
}

#[cfg(feature = "geometry")]
#[test]
fn a_cube_index_beyond_the_bone_s_cubes() {
    let rule = |cube: u32| {
        format!(
            r#"{{"geometry_part": {{"bone": "body", "cube": {cube}, "face": "up"}}, "direction": "up"}}"#
        )
    };
    let block = block(r#", "culling": "demo:lamp_culling""#);
    // `body` has two cubes: index 2 is the first one out of bounds, index 1 the last one in.
    for cube in [2, 3] {
        let found = link(&block, &[culling(&rule(cube))]);
        let out = with_text(
            &found,
            &format!("Cube index `{cube}` is out of bounds for schematic part `body`"),
        );
        assert_eq!(out.len(), 1, "{cube}: {found:#?}");
        assert_eq!(out[0].code.name(), "InvalidValue");
        assert_eq!(out[0].severity, Severity::Error, "level 3, kept");
        assert!(out[0].kept);
    }
    let found = link(&block, &[culling(&rule(1))]);
    assert!(
        found
            .iter()
            .all(|f| !f.code.to_string().contains("out of bounds")),
        "{found:#?}"
    );
}

const SAME_LAYER: &str =
    r#"{"condition": "same_culling_layer", "geometry_part": {"bone": "body"}, "direction": "up"}"#;
const NO_LAYER: &str =
    "culling condition specified is same_culling_layer, but geometry has no culling layer defined";

#[test]
fn same_culling_layer_on_a_block_without_a_layer() {
    let found = link(
        &block(r#", "culling": "demo:lamp_culling""#),
        &[culling(SAME_LAYER)],
    );
    let warned = with_text(&found, NO_LAYER);
    assert_eq!(warned.len(), 1, "{found:#?}");
    assert_eq!(warned[0].severity, Severity::Warning);
    // The default layer written out is still no layer.
    let found = link(
        &block(
            r#", "culling": "demo:lamp_culling", "culling_layer": "minecraft:culling_layer.undefined""#,
        ),
        &[culling(SAME_LAYER)],
    );
    assert_eq!(with_text(&found, NO_LAYER).len(), 1, "{found:#?}");
    // Another condition does not need a layer.
    let found = link(
        &block(r#", "culling": "demo:lamp_culling""#),
        &[culling(
            r#"{"condition": "same_block", "geometry_part": {"bone": "body"}, "direction": "up"}"#,
        )],
    );
    assert!(with_text(&found, NO_LAYER).is_empty(), "{found:#?}");
}

#[test]
fn same_culling_layer_on_a_block_with_a_layer() {
    let found = link(
        &block(r#", "culling": "demo:lamp_culling", "culling_layer": "demo:hedge""#),
        &[culling(SAME_LAYER)],
    );
    assert!(with_text(&found, NO_LAYER).is_empty(), "{found:#?}");
    assert!(
        found.iter().all(|f| f.severity < Severity::Warning),
        "{found:#?}"
    );
}

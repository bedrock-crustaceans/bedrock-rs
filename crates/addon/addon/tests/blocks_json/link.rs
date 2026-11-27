//! What `blocks.json` says about things other collections
//! define. Each needs the behaviour stack or another resource collection beside this one, so
//! it is the link stage's: each test calls `StackSet::link()`.

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Kind, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::{file, leak};
use super::pack_common::{bp, rp};

/// The link findings of a world with one behaviour pack (`behavior` files) and one resource
/// pack (`resource` files), with `options`.
fn link_with(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    options: LoadOptions,
) -> Diagnostics {
    let mut b = bp(1);
    for (path, text) in behavior {
        b = b.file(path, leak(text));
    }
    let mut r = rp(2);
    for (path, text) in resource {
        r = r.file(path, leak(text));
    }
    let (b, r) = (b.open(), r.open());
    let list = |uuid: uuid::Uuid| format!(r#"[{{"pack_id": "{uuid}", "version": [1, 0, 0]}}]"#);
    let world = WorldPacks::parse(
        Some(list(b.id().uuid).as_bytes()),
        Some(list(r.id().uuid).as_bytes()),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(b, PackOrigin::User);
    repository.add_pack(r, PackOrigin::User);
    repository.stacks(&world, options).link()
}

fn link(behavior: &[(&'static str, &str)], resource: &[(&'static str, &str)]) -> Diagnostics {
    link_with(behavior, resource, LoadOptions::default())
}

/// The findings whose text is `text`.
#[cfg(feature = "block")]
fn with_text<'a>(d: &'a Diagnostics, text: &str) -> Vec<&'a Diagnostic> {
    d.iter().filter(|f| f.code.to_string() == text).collect()
}

/// A custom block `demo:lamp` with `components`.
#[cfg(feature = "block")]
fn block(components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:lamp"}}, "components": {{{components}}}}}}}"#
    )
}

// From 1.21.80 a block needs both a geometry and material instances: with the instances
// alone the block does not load, and is no block to the link rules.
#[cfg(feature = "block")]
const MATERIALS: &str = r#""minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "lamp"}}"#;

// The rule is `block`'s and `blocks_json`'s together.
#[cfg(feature = "block")]
#[test]
fn a_key_that_names_no_block() {
    let found = link(
        &[("blocks/lamp.json", &block(MATERIALS))],
        &[(
            "blocks.json",
            &file(
                r#""not_a_block": {}, "demo:lamp": {"sound": "glass"}, "demo:ghost": {}, "stone": {}"#,
            ),
        )],
    );
    for key in ["not_a_block", "demo:ghost"] {
        let missing = with_text(
            &found,
            &format!(
                "The block named {key} used in a \"blocks.json\" file does not exist in the registry"
            ),
        );
        assert_eq!(missing.len(), 1, "{key}: {found:#?}");
        assert_eq!(missing[0].code.id(), "core/unresolved_reference_worded");
        assert_eq!(missing[0].severity, Severity::Warning);
    }
    // A behaviour-pack block and a built-in block exist.
    assert!(
        found.iter().all(|f| {
            let text = f.code.to_string();
            !text.contains("named demo:lamp ") && !text.contains("named stone ")
        }),
        "{found:#?}"
    );
}

#[cfg(all(feature = "block", feature = "texture_atlas"))]
const TERRAIN: &str = r#"{"resource_pack_name": "t", "texture_name": "atlas.terrain", "texture_data": {"a": {"textures": ["t0"]}, "b": {"textures": []}}}"#;

#[cfg(all(feature = "block", feature = "texture_atlas"))]
#[test]
fn texture_names_against_the_terrain_atlas() {
    let entries = r#""demo:plain": {"textures": "a"}, "demo:first": {"textures": "a:0"},
        "demo:beyond": {"textures": "a:3"}, "demo:empty": {"textures": "b"},
        "demo:missing": {"textures": "missing"}"#;
    // The five keys name blocks of the behaviour pack: a key of no block is reported at
    // the key.
    let blocks = ["plain", "first", "beyond", "empty", "missing"].map(|name| {
        let path: &'static str = Box::leak(format!("blocks/{name}.json").into_boxed_str());
        let text = block(MATERIALS).replace("demo:lamp", &format!("demo:{name}"));
        (path, text)
    });
    let behavior: Vec<(&'static str, &str)> = blocks
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    let found = link(
        &behavior,
        &[
            ("blocks.json", &file(entries)),
            ("textures/terrain_texture.json", TERRAIN),
        ],
    );
    // `a` and `a:0` resolve. `a:3` selects an element `a` does not have: element 0 is used.
    let at = |key: &str| -> Vec<&Diagnostic> {
        found
            .iter()
            .filter(|f| f.location.json_path.to_string().starts_with(key))
            .collect()
    };
    assert!(at("demo:plain").is_empty(), "{found:#?}");
    assert!(at("demo:first").is_empty(), "{found:#?}");
    let beyond = at("demo:beyond");
    assert_eq!(beyond.len(), 1, "{found:#?}");
    // Silent in the game: ours, an information.
    assert_eq!(beyond[0].severity, Severity::Info);
    assert_eq!(beyond[0].code.id(), "core/unresolved_reference");
    // An atlas entry without elements: vanilla's content-log warning. A name the
    // atlas lacks: the atlas's developer-log line, an information.
    for (key, text, severity) in [
        (
            "demo:empty",
            "Terrain texture <b> doesn't have any elements. Using the missing texture.",
            Severity::Warning,
        ),
        (
            "demo:missing",
            "This texture item doesn't exist: Texture [missing]",
            Severity::Info,
        ),
    ] {
        let unresolved = with_text(&found, text);
        assert_eq!(unresolved.len(), 1, "{key}: {found:#?}");
        assert_eq!(unresolved[0].severity, severity, "{key}");
        assert!(
            unresolved[0]
                .location
                .json_path
                .to_string()
                .starts_with(key)
        );
    }
}

// The rule is `block`'s and `blocks_json`'s together.
#[cfg(feature = "block")]
#[test]
fn an_exponent_on_a_block_with_material_instances() {
    let text = "Block demo:lamp has an \"ambient_occlusion_exponent\" in 'blocks.json' file, this value is unused because the value in \"minecraft:material_instances\" component is used instead";
    let run = |exponent: &str| {
        link(
            &[("blocks/lamp.json", &block(MATERIALS))],
            &[(
                "blocks.json",
                &file(&format!(
                    r#""demo:lamp": {{"ambient_occlusion_exponent": {exponent}}}"#
                )),
            )],
        )
    };
    let found = run("0.5");
    let unused = with_text(&found, text);
    assert_eq!(unused.len(), 1, "{found:#?}");
    assert_eq!(unused[0].severity, Severity::Warning);
    // 1.0 is the default: nothing to say. Nor without the component.
    assert!(with_text(&run("1.0"), text).is_empty());
    let found = link(
        &[("blocks/lamp.json", &block(r#""minecraft:friction": 0.4"#))],
        &[(
            "blocks.json",
            &file(r#""demo:lamp": {"ambient_occlusion_exponent": 0.5}"#),
        )],
    );
    assert!(with_text(&found, text).is_empty(), "{found:#?}");
}

// The rule is `block`'s and `blocks_json`'s together.
#[cfg(feature = "block")]
#[test]
fn the_tessellation_fallback_of_upcoming_creator_features() {
    let geometry = r#""minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "lamp"}}"#;
    let entries = r#""demo:lamp": {"textures": "lamp"}"#;
    let custom = "demo:lamp: trying to override the Geometry component with blocks.json settings for a custom block. This isn't supported. Please remove any legacy texture definition or block shape specification for this block.";
    let options = LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    };
    let found = link_with(
        &[("blocks/lamp.json", &block(geometry))],
        &[("blocks.json", &file(entries))],
        options,
    );
    // A custom block on the new pipeline with creator-pack `textures`.
    let refused = with_text(&found, custom);
    assert_eq!(refused.len(), 1, "{found:#?}");
    assert_eq!(refused[0].severity, Severity::Warning);
    // Without the experiment: none of those findings.
    let found = link(
        &[("blocks/lamp.json", &block(geometry))],
        &[("blocks.json", &file(entries))],
    );
    assert!(with_text(&found, custom).is_empty(), "{found:#?}");
    assert!(
        found.iter().all(|f| !f
            .code
            .to_string()
            .contains("textures are missing to be able to override Geometry component")),
        "{found:#?}"
    );
}

#[cfg(feature = "sounds_json")]
const SOUNDS: &str = r#"{"block_sounds": {"normal": {"events": {}}, "stone": {"events": {}}}}"#;

#[cfg(feature = "sounds_json")]
#[test]
fn a_sound_that_sounds_json_does_not_define() {
    let found = link(
        &[],
        &[
            (
                "blocks.json",
                &file(
                    r#""obsidian": {"sound": "stone"}, "glass": {"sound": "squeaky"}, "dirt": {"sound": 5}"#,
                ),
            ),
            ("sounds.json", SOUNDS),
        ],
    );
    let unknown: Vec<&Diagnostic> = found
        .iter()
        .filter(|f| f.code.id() == "core/unknown_sound")
        .collect();
    assert_eq!(unknown.len(), 1, "{found:#?}");
    assert_eq!(
        unknown[0].severity,
        Severity::Info,
        "the client falls back to `normal` with a level-0 line alone"
    );
    assert_eq!(unknown[0].location.json_path.to_string(), "glass/sound");
    // Without a `sounds.json` there is nothing to compare with.
    let found = link(
        &[],
        &[("blocks.json", &file(r#""glass": {"sound": "squeaky"}"#))],
    );
    assert!(
        found
            .iter()
            .all(|f| f.code.kind() != Kind::UnresolvedReference),
        "{found:#?}"
    );
}

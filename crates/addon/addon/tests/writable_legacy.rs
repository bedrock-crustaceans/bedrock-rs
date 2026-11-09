#![allow(
    dead_code,
    unused_imports,
    reason = "the harness is the probe's of every family"
)]
#![cfg(feature = "full")]
//! No upgrade step and no legacy reader rewrites or drops an author's value in silence: the
//! second test of the invariant of written documents (`writable.rs` is the first, over
//! documents of the newest format).
//!
//! One test per family with an upgrade chain or a legacy reader. Its inputs are LEGACY
//! documents of every version a step of the family starts from:
//!
//! * the documents of the family's own tests and fixtures (every JSON literal of
//!   `tests/<family>/` and every small `.json` file of `tests/fixtures/<family>/` that carries
//!   a `format_version` below the newest, or none);
//! * the documents of [`more`], written for the members a step reads with no op that carries
//!   a value (the `callback` ops);
//! * and the inputs the probe MAKES of these, each probed at the one member it was
//!   made for: for every document, the document with each
//!   member already there that its own step, run alone, or the whole chain ADDS to it (once
//!   with another value of the member's type, once with a value of another type); the
//!   document with an unknown member `"zz": 3` in each of its objects; and the document with
//!   one more element in each of its lists.
//!
//! A document is probed when the family loads and writes it. Every value of it below the root
//! is replaced in turn by each of [`REPLACEMENTS`] that is of another class than the value
//! (an integer is not replaced by `7`, a string not by `"x"`), once per version and place
//! (indices as `*`) of the family. With `T` the document and `T'` the document with the
//! replacement, `W` and `W'` what the family writes for them, the outcome must be one of:
//!
//! * **refused**: the load of `T'` yields no document;
//! * **said**: a finding the load of `T` does not have is at an enclosing member; or at the
//!   replaced place or below it, and `W'` loses nothing the author wrote BESIDE the member
//!   that `W` holds (an element of a list is told of as its list is);
//! * **said elsewhere**: a finding the load of `T` does not have is at another place (the
//!   member an upgrade step moved the value to, or what the loss of the value leaves wrong);
//! * **told as not read**: `W'` is `W` and a finding of both loads is at the place or at an
//!   enclosing member (a member the load says it drops or does not know, whatever it holds);
//! * **held**: `W'` holds the replacement where `W` did not (a number as the number it
//!   spells, a float member as the nearest `f32`); or `W'` is `W`, which holds the
//!   replacement, and the load of `T` was told at the place that its value is not read while
//!   this load is not (the replacement is what the member was read as);
//! * **carried**: `W'` depends on the replacement's value and not on its class alone: a second
//!   replacement of the same class (`8` for `7`, `"y"` for `"x"`) is written as another
//!   text, in values of the replacement's own type (a step that converts seconds to ticks, a
//!   name that gains its namespace);
//! * **absent**: the replacement is `null`, `[]` or `{}` and `W'` is what the document without
//!   the member writes (a `null` member is an absent member);
//! * **empty**: the replacement is `[]` or `{}` and `W'` is another text than `W`.
//!
//! Anything else is a FAULT, of one of six kinds:
//!
//! * `no-effect`: `W'` is `W` (the replacement was ignored or defaulted in silence);
//! * `not-written`: the load yields a document and `to_json` refuses it, untold; or the load
//!   or the write panics;
//! * `collateral`: a member of `W` that is not owed to the replaced member is absent from
//!   `W'` (the author's sibling data went with the replaced value), also where a finding is
//!   at the replaced member alone;
//! * `not-reread`: the text a document (as it is, or a generated input) is written as does
//!   not load again to a document that is written as the same text;
//! * `displaced`: the document WITHOUT a member is written with a scalar the author wrote
//!   elsewhere that the document itself is written with fewer of, and no finding the document
//!   without the member lacks is at an enclosing member or at the member that holds the lost
//!   scalar (what a member's presence costs: a step that writes over a sibling);
//! * `changed`: `W'` neither holds nor carries the replacement (read as a constant, clamped,
//!   truncated or converted to another type in silence).
//!
//! Each member and element is also taken OUT of the document once (`(removed)`): the load
//! may refuse the document, say what is missing, or write the rest; a `collateral` fault is a
//! scalar the author wrote elsewhere that the written text loses with no new finding (a
//! component dropped whole for a member it lacks).
//!
//! The probe is seven test binaries, since a test binary runs under a limit of CPU seconds:
//! this file (24 families), `writable_legacy_block.rs`, `writable_legacy_item.rs` and the
//! entity's four (`writable_legacy_entity.rs` to `_entity_4.rs`, by band of versions). The
//! harness (`writable_legacy/harness.rs`) is one.
//!
//! `legacy:` lines (shown with `--show-output`) give the counts per family, `legacy-at:` lines
//! the faults: `family | version | path | replacement | kind | detail`, `legacy-elsewhere:`
//! lines every "said elsewhere": `family | path | replacement | finding`.

#[path = "writable_legacy/candidates.rs"]
mod candidates;
#[allow(
    unused_macros,
    reason = "the list of document types is the other probes'"
)]
#[path = "common/document_types.rs"]
mod document_types;
#[path = "common/places.rs"]
mod places;

include!("writable_legacy/harness.rs");
include!("writable_legacy/families.rs");

legacy_families! {
    block_culling: ["block_culling"], ["minecraft:block_culling_rules"], [
        ("a pack", bedrock_addon::families::block_culling::CullingRules, "block_culling/test.json", LoadContext::detached()),
    ], Vec::new();
    attachable: ["attachable"], ["minecraft:attachable"], [
        ("a pack", bedrock_addon::families::attachable::Attachable, "attachables/x.json", client((1, 21, 0))),
    ], Vec::new();
    client_entity: ["client_entity"], ["minecraft:client_entity"], [
        ("a pack", bedrock_addon::families::client_entity::ClientEntity, "entity/x.entity.json", client((1, 21, 0))),
    ], Vec::new();
    spawn_rules: ["spawn"], ["minecraft:spawn_rules"], [
        ("a pack", bedrock_addon::families::spawn_rules::SpawnRules, "spawn_rules/x.json", LoadContext::detached()),
    ], Vec::new();
    spawn_simplified: ["spawn"], ["minecraft:spawn_rules"], [
        ("a pack", bedrock_addon::families::spawn_rules::SpawnRules, "spawn_rules/x.json", LoadContext::detached()),
    ], Vec::new();
    spawn_group: ["spawn_groups"], ["minecraft:spawn_groups"], [
        ("a pack", bedrock_addon::families::spawn_groups::SpawnGroup, "spawn_groups/x.json", LoadContext::detached()),
    ], Vec::new();
    biome: ["biome"], ["minecraft:biome"], [
        ("a pack", bedrock_addon::families::biome::Biome, "biomes/x.json", LoadContext::detached()),
    ], steps![LoadContext::detached;
        bedrock_addon::families::biome::history::v1_21_110::Biome,
        bedrock_addon::families::biome::history::v1_21_120::Biome,
        bedrock_addon::families::biome::history::v1_26_0::Biome,
    ];
    client_biome: ["biomes_client"], ["minecraft:client_biome"], [
        ("a pack", bedrock_addon::families::client_biome::ClientBiome, "biomes/demo.client_biome.json", pack(PackType::Resource, (1, 26, 30))),
        ("the legacy file", bedrock_addon::families::biomes_client::LegacyClientBiomes, "biomes_client.json", pack(PackType::Resource, (1, 26, 30))),
    ], steps![LoadContext::detached;
        bedrock_addon::families::client_biome::history::v1_21_40::ClientBiome,
        bedrock_addon::families::client_biome::history::v1_21_70::ClientBiome,
        bedrock_addon::families::client_biome::history::v1_21_90::ClientBiome,
        bedrock_addon::families::client_biome::history::v1_21_100::ClientBiome,
        bedrock_addon::families::client_biome::history::v1_21_110::ClientBiome,
        bedrock_addon::families::client_biome::history::v1_21_120::ClientBiome,
    ];
    feature: ["feature"], [
        "minecraft:scatter_feature", "minecraft:single_block_feature", "minecraft:snap_to_surface_feature"
    ], [
        ("a pack", bedrock_addon::families::feature::Feature, "features/test.json", LoadContext::detached()),
    ], Vec::new();
    dimension: ["dimension"], ["minecraft:dimension"], [
        ("a pack", bedrock_addon::families::dimension::Dimension, "dimensions/overworld.json", LoadContext::detached()),
    ], Vec::new();
    jigsaw: ["jigsaw"], [
        "minecraft:jigsaw", "minecraft:template_pool", "minecraft:structure_set", "minecraft:processor_list"
    ], [
        ("a structure", bedrock_addon::families::jigsaw::JigsawStructure, "worldgen/structures/a.json", pack(PackType::Behavior, (1, 26, 20))),
        ("a pool", bedrock_addon::families::jigsaw::TemplatePool, "worldgen/template_pools/p.json", pack(PackType::Behavior, (1, 26, 20))),
        ("a set", bedrock_addon::families::jigsaw::StructureSet, "worldgen/structure_sets/s.json", pack(PackType::Behavior, (1, 26, 20))),
        ("a processor list", bedrock_addon::families::jigsaw::ProcessorList, "worldgen/processors/l.json", pack(PackType::Behavior, (1, 26, 20))),
    ], Vec::new();
    trade_table: ["trading"], ["tiers"], [
        ("a pack", bedrock_addon::families::trade::TradeTable, "trading/x.json", LoadContext::detached()),
    ], Vec::new();
    camera_preset: ["camera"], ["minecraft:camera_preset"], [
        ("a pack", bedrock_addon::families::camera::CameraPreset, "cameras/presets/x.json", LoadContext::detached()),
    ], Vec::new();
    aim_assist: ["aim_assist"], ["minecraft:aim_assist_preset", "minecraft:aim_assist_categories"], [
        ("a preset", bedrock_addon::families::aim_assist::AimAssistPreset, "aim_assist/presets/test.json", LoadContext::detached()),
        ("the categories", bedrock_addon::families::aim_assist::AimAssistCategories, "aim_assist/categories/test.json", LoadContext::detached()),
    ], Vec::new();
    poi_types: ["poi"], ["minecraft:poi_block", "minecraft:poi_tag"], [
        ("a type", bedrock_addon::families::poi::PoiType, "poi/types/x.json", document_types::poi()),
        ("a tag", bedrock_addon::families::poi::PoiTag, "poi/document.json", document_types::poi()),
    ], Vec::new();
    particle: ["particle"], ["particle_effect"], [
        ("a pack", bedrock_addon::families::particle::ParticleEffect, "particles/demo.particle.json", pack(PackType::Resource, (1, 21, 100))),
    ], steps![LoadContext::detached;
        bedrock_addon::families::particle::history::v1_10_0::ParticleEffect,
    ];
    fog: ["fog"], ["minecraft:fog_settings"], [
        ("a pack", bedrock_addon::families::fog::FogSettings, "fogs/x.json", LoadContext::detached()),
    ], steps![LoadContext::detached;
        bedrock_addon::families::fog::history::v1_16_100::FogDocument,
        bedrock_addon::families::fog::history::v1_21_0::FogDocument,
        bedrock_addon::families::fog::history::v1_21_90::FogDocument,
    ];
    render_configs: ["render_configs"], [
        "minecraft:lighting_settings", "minecraft:atmosphere_settings", "minecraft:water_settings",
        "minecraft:color_grading_settings", "minecraft:cubemap_settings", "minecraft:shadow_settings",
        "minecraft:pbr_fallback_settings", "minecraft:local_light_settings"
    ], [
        ("lighting", bedrock_addon::families::deferred_settings::LightingSettings, "test.json", LoadContext::detached()),
        ("atmosphere", bedrock_addon::families::deferred_settings::AtmosphereSettings, "test.json", LoadContext::detached()),
        ("water", bedrock_addon::families::deferred_settings::WaterSettings, "test.json", LoadContext::detached()),
        ("color grading", bedrock_addon::families::deferred_settings::ColorGradingSettings, "test.json", LoadContext::detached()),
        ("cubemap", bedrock_addon::families::deferred_settings::CubemapSettings, "test.json", LoadContext::detached()),
        ("shadows", bedrock_addon::families::deferred_settings::ShadowSettings, "test.json", LoadContext::detached()),
        ("pbr fallback", bedrock_addon::families::deferred_settings::PbrFallbackSettings, "test.json", LoadContext::detached()),
        ("local light", bedrock_addon::families::deferred_settings::LocalLightSettings, "test.json", LoadContext::detached()),
    ], Vec::new();
    texture_set: ["texture_set"], ["minecraft:texture_set"], [
        ("a pack", bedrock_addon::families::texture_set::TextureSet, "textures/blocks/x.texture_set.json", LoadContext::detached()),
    ], Vec::new();
    sound_definitions: ["sound_definitions"], ["sound_definitions"], [
        ("a pack", bedrock_addon::families::sound_definitions::SoundDefinitions, "sounds/sound_definitions.json", LoadContext::detached()),
    ], steps![LoadContext::detached;
        bedrock_addon::families::sound_definitions::history::v1_0_0::SoundDefinitions,
        bedrock_addon::families::sound_definitions::history::v1_14_0::SoundDefinitions,
        bedrock_addon::families::sound_definitions::history::v1_20_20::SoundDefinitions,
    ];
}

/// The saved pack settings have an entry of their own (`SavedSettings::parse`).
#[test]
fn pack_settings() {
    use bedrock_addon::pack::settings::SavedSettings;
    let read: Reader = (
        "the file",
        Box::new(|text: &str| {
            outcome(SavedSettings::parse(text.as_bytes()), |value| {
                value.to_json().map_err(|error| format!("{error:?}"))
            })
        }),
    );
    audit(&Family {
        name: "pack_settings",
        folders: &["pack_settings"],
        keys: &["minecraft:pack_settings"],
        readers: vec![read],
        runs: Vec::new(),
        band: None,
    });
}

// --- the harness itself --------------------------------------------------------------------------

#[test]
fn a_member_is_taken_out_of_a_text_with_its_comma() {
    let text = r#"{"a": 1, "b": [2, {"c": 3}], "d": "x"}"#;
    let all = nodes(text);
    let at = |path: &str| {
        let node = all
            .iter()
            .find(|node| node.shown() == path)
            .expect("the place");
        without(text, node).expect("a member")
    };
    assert_eq!(at("a"), r#"{ "b": [2, {"c": 3}], "d": "x"}"#);
    assert_eq!(at("d"), r#"{"a": 1, "b": [2, {"c": 3}]}"#);
    assert_eq!(at("b/0"), r#"{"a": 1, "b": [ {"c": 3}], "d": "x"}"#);
    assert_eq!(at("b/1"), r#"{"a": 1, "b": [2], "d": "x"}"#);
    assert_eq!(at("b/1/c"), r#"{"a": 1, "b": [2, {}], "d": "x"}"#);
}

#[test]
fn the_json_literals_of_a_source_are_found() {
    let source = "let a = r#\"{\"a\": 1}\"#; let b = r##\"[\"#\"]\"##; let c = r#\"text\"#;";
    assert_eq!(literals(source), [r#"{"a": 1}"#, r##"["#"]"##]);
}

#[test]
fn a_value_has_the_class_of_what_it_spells() {
    let text = r#"{"a": 1, "b": 0.5, "c": 4294967296, "d": 1e39, "e": [], "f": [1], "g": {}, "h": null, "i": false, "j": "x"}"#;
    let classes: Vec<Class> = nodes(text)
        .iter()
        .filter(|node| node.path.len() == 1)
        .map(|node| class_of(text, node))
        .collect();
    assert_eq!(
        classes,
        [
            Class::Integer,
            Class::Fraction,
            Class::Wide,
            Class::Huge,
            Class::EmptyArray,
            Class::Array,
            Class::EmptyObject,
            Class::Null,
            Class::Bool,
            Class::String
        ]
    );
}

/// A text written at the file's own format version leaves out what a step synthesises, and
/// the judge does not count that against it; a value of the author's that a step MOVED and
/// the text lost is a loss all the same. The toy family below moves `old` to `new`, puts
/// `made: true` into every document, and writes a document whose `x` is 7 at its own version
/// without `made`, and without the moved value when `lose` is set.
// Review round 1 of X-WRITER-RT, B2.
#[test]
fn an_own_version_text_that_loses_a_moved_value_of_the_author_is_at_fault() {
    fn toy(lose: bool) -> impl Fn(&str) -> Outcome {
        move |text: &str| {
            let value = parse(text).expect("JSON");
            let payload = value
                .as_object()
                .and_then(|root| root.get("toy"))
                .and_then(Value::as_object)
                .expect("a payload");
            let scalar_of = |key: &str| {
                payload
                    .get(key)
                    .map(|v| bedrock_addon::json::to_json(v).unwrap())
            };
            let moved = scalar_of("old").or_else(|| scalar_of("new"));
            let x = scalar_of("x").unwrap_or_else(|| "0".to_owned());
            let own = x.trim() == "7";
            let mut members = vec![format!(
                r#""description": {{"identifier": "demo:x"}}, "x": {}"#,
                x.trim()
            )];
            if let Some(moved) = moved.filter(|_| !(own && lose)) {
                members.push(format!(r#""new": {}"#, moved.trim()));
            }
            if !own {
                members.push(r#""made": true"#.to_owned());
            }
            let version = if own { "1.0.0" } else { "2.0.0" };
            Outcome {
                loaded: true,
                written: Ok(format!(
                    r#"{{"format_version": "{version}", "toy": {{{}}}}}"#,
                    members.join(", ")
                )),
                findings: Vec::new(),
            }
        }
    }
    let text = r#"{"format_version": "1.0.0", "toy": {"description": {"identifier": "demo:x"}, "x": 1, "old": true}}"#;
    let changed = text.replace(r#""x": 1"#, r#""x": 7"#);
    let all = nodes(text);
    let node = all
        .iter()
        .find(|node| node.shown() == "toy/x")
        .expect("the place");
    let lost = |lose: bool| {
        let read = toy(lose);
        let (written, _) = read_text(&read, text).ok().expect("written");
        let base = base_of("toy", text, parse(text).unwrap(), &all, written, &read);
        let (ours, _) = read_text(&read, &changed).ok().expect("written");
        lost_beside(&base, node, &ours, &[])
    };
    // Without `made` alone: nothing of the author's is lost.
    assert_eq!(lost(false), Vec::<String>::new());
    // Without the moved `true` of the author's too: a loss.
    assert_eq!(lost(true), ["b:true"]);
}

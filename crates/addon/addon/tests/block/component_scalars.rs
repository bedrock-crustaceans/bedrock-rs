//! The plain members of the typed components, read in a legacy document (the non-strict
//! reader), a 1.26.20 one and a 1.26.60 one (the strict reader) from every JSON type at their
//! scalar places. Each line pins what the load says (code, severity, path, span) and, when the
//! block loads, the component written back. The members were strict wrappers and the scalar
//! shorthands were checked by hand; this table is what they read, and the door's scalar
//! pre-pass reads them the same under the slice's `Reading`.

use super::common::{ALLOW_LISTED, document, load, newest_shape};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::version::MinEngineVersion;

/// The component values the table reads, per component key.
const VALUES: &[(&str, &[&str])] = &[
    (
        "minecraft:friction",
        &[
            "0.5", "1", "true", "\"0.5\"", "null", "1e39", "-0.1", "2", "[]",
        ],
    ),
    (
        "minecraft:light_emission",
        &["5", "15", "16", "256", "-1", "1.5", "true", "\"5\"", "null"],
    ),
    (
        "minecraft:light_dampening",
        &["5", "16", "300", "-1", "2.5", "false", "\"5\""],
    ),
    ("minecraft:display_name", &["\"a\"", "5", "true", "null"]),
    ("minecraft:loot", &["\"loot_tables/x.json\"", "5", "true"]),
    ("minecraft:tags", &["[\"a:b\"]", "[5]", "\"a:b\"", "[true]"]),
    (
        "minecraft:map_color",
        &[
            "\"#ffffff\"",
            "[1, 2, 3]",
            "[256, 0, 0]",
            "[1.5, 0, 0]",
            "[\"a\", 0, 0]",
            "[true, 0, 0]",
            "5",
            "true",
            r##"{"color": "#ffffff"}"##,
            r#"{"color": 5}"#,
            r#"{"color": true}"#,
            r#"{"color": [1, 2, 3], "tint_method": "grass"}"#,
        ],
    ),
    (
        "minecraft:geometry",
        &[
            "\"geometry.x\"",
            "5",
            "true",
            r#"{"identifier": "geometry.x", "uv_lock": true}"#,
            r#"{"identifier": "geometry.x", "uv_lock": 5}"#,
            r#"{"identifier": "geometry.x", "uv_lock": "a"}"#,
            r#"{"identifier": "geometry.x", "uv_lock": ["a", 5]}"#,
            r#"{"identifier": 5}"#,
            r#"{"identifier": true}"#,
            r#"{"identifier": "geometry.x", "culling": 5}"#,
            r#"{"identifier": "geometry.x", "n_way_visual_rotation": {"x": 5}}"#,
        ],
    ),
    (
        "minecraft:selection_box",
        &[
            "true",
            "5",
            "\"true\"",
            r#"{"origin": [-8, 0, -8], "size": [16, 16, 16]}"#,
            r#"{"origin": [true, 0, 0]}"#,
            r#"{"origin": ["1", 0, 0]}"#,
            r#"{"origin": [1.5, 0, 0]}"#,
            r#"{"origin": [-8, 0, -8], "size": [1e39, 16, 16]}"#,
        ],
    ),
    (
        "minecraft:collision_box",
        &[
            "true",
            "5",
            "\"x\"",
            r#"{"origin": [-8, 0, -8], "size": [16, 16, 16]}"#,
            r#"{"origin": [true, 0, -8]}"#,
            r#"[{"min": [0, 0, 0], "max": [16, 16, 16]}]"#,
            r#"[{"min": [true, 0, 0]}]"#,
            r#"[{"min": ["1", 0, 0]}]"#,
            "[5]",
        ],
    ),
    (
        "minecraft:flammable",
        &[
            "true",
            "5",
            "\"true\"",
            r#"{"catch_chance_modifier": 5}"#,
            r#"{"catch_chance_modifier": 5.5}"#,
            r#"{"catch_chance_modifier": true}"#,
            r#"{"catch_chance_modifier": 70000}"#,
            r#"{"catch_chance_modifier": -1}"#,
            r#"{"destroy_chance_modifier": "5"}"#,
            r#"{"lava_flammable": "always"}"#,
            r#"{"lava_flammable": 5}"#,
        ],
    ),
    (
        "minecraft:destructible_by_mining",
        &[
            "true",
            "5",
            "\"true\"",
            r#"{"seconds_to_destroy": 1}"#,
            r#"{"seconds_to_destroy": true}"#,
            r#"{"seconds_to_destroy": "1"}"#,
            r#"{"seconds_to_destroy": -1}"#,
            r#"{"seconds_to_destroy": 1, "item_specific_speeds": [{"item": "minecraft:stick", "destroy_speed": true}]}"#,
            r#"{"seconds_to_destroy": 1, "item_specific_speeds": [{"item": "minecraft:stick", "destroy_speed": 2}]}"#,
        ],
    ),
    (
        "minecraft:tick",
        &[
            r#"{"interval_range": [1, 2]}"#,
            r#"{"interval_range": [1.5, 2]}"#,
            r#"{"interval_range": [true, 2]}"#,
            r#"{"interval_range": [-1, 2]}"#,
            r#"{"interval_range": [3, 2]}"#,
            r#"{"interval_range": [1, 2], "looping": 1}"#,
            r#"{"interval_range": [1, 2], "looping": "true"}"#,
        ],
    ),
    (
        "minecraft:random_offset",
        &[
            r#"{"x": {"range": {"min": 0, "max": 1}, "steps": 2}}"#,
            r#"{"x": {"range": {"min": 0, "max": 1}, "steps": 2.5}}"#,
            r#"{"x": {"range": {"min": 0, "max": 1}, "steps": true}}"#,
            r#"{"x": {"range": {"min": 0, "max": 1}, "steps": -1}}"#,
            r#"{"x": {"range": {"min": true, "max": 1}}}"#,
        ],
    ),
    (
        "minecraft:material_instances",
        &[
            r#"{"*": {"texture": "a"}}"#,
            r#"{"*": {"texture": 5}}"#,
            r#"{"*": 5}"#,
            r#"{"*": true}"#,
            r#"{"*": {"texture": "a"}, "up": "*"}"#,
            r#"{"*": {"texture": "a", "ambient_occlusion": true}}"#,
            r#"{"*": {"texture": "a", "ambient_occlusion": 11}}"#,
            r#"{"*": {"texture": "a", "face_dimming": 1}}"#,
            r#"{"*": {"texture": "a", "isotropic": "true"}}"#,
        ],
    ),
    (
        "minecraft:redstone_producer",
        &[
            r#"{"power": 5}"#,
            r#"{"power": 16}"#,
            r#"{"power": true}"#,
            r#"{"power": 5.5}"#,
            r#"{"power": 256}"#,
            r#"{"power": 5, "transform_relative": 1}"#,
        ],
    ),
    (
        "minecraft:redstone_consumer",
        &[
            r#"{"min_power": 5}"#,
            r#"{"min_power": 16}"#,
            r#"{"min_power": true}"#,
            r#"{"propagates_power": 1}"#,
        ],
    ),
    (
        "minecraft:redstone_conductivity",
        &[
            r#"{"redstone_conductor": true}"#,
            r#"{"redstone_conductor": 1}"#,
        ],
    ),
    (
        "minecraft:block_entity",
        &[
            r#"{"container": {"slot_count": 5}}"#,
            r#"{"container": {"slot_count": 55}}"#,
            r#"{"container": {"slot_count": true}}"#,
            r#"{"container": {"slot_count": 1.5}}"#,
            r#"{"container": {"slot_count": 5, "title": 5}}"#,
            r#"{"dynamic_properties": 1}"#,
        ],
    ),
    (
        "minecraft:crafting_table",
        &[
            r#"{"crafting_tags": ["a"]}"#,
            r#"{"crafting_tags": [5]}"#,
            r#"{"crafting_tags": ["a"], "table_name": 5}"#,
        ],
    ),
    (
        "minecraft:leashable",
        &[r#"{"offset": [0, 1, 0]}"#, r#"{"offset": [true, 0, 0]}"#],
    ),
    (
        "minecraft:transformation",
        &[
            r#"{"rotation": [0, 90, 0]}"#,
            r#"{"rotation": [true, 0, 0]}"#,
            r#"{"rotation": ["90", 0, 0]}"#,
            r#"{"rotation": [45, 0, 0]}"#,
            r#"{"scale": [1e39, 1, 1]}"#,
        ],
    ),
    (
        "minecraft:entity_fall_on",
        &[
            r#"{"min_fall_distance": 1}"#,
            r#"{"min_fall_distance": true}"#,
            r#"{"min_fall_distance": "1"}"#,
        ],
    ),
    (
        "minecraft:liquid_detection",
        &[
            r#"{"detection_rules": [{"can_contain_liquid": true}]}"#,
            r#"{"detection_rules": [{"can_contain_liquid": 1}]}"#,
        ],
    ),
    (
        "minecraft:destruction_particles",
        &[
            r#"{"particle_count": 30}"#,
            r#"{"particle_count": 300}"#,
            r#"{"particle_count": 1.5}"#,
            r#"{"texture": 5}"#,
        ],
    ),
    (
        "minecraft:vibration_properties",
        &[
            r#"{"dampen_vibrations": 1}"#,
            r#"{"dampen_vibrations": true}"#,
        ],
    ),
    (
        "minecraft:enchantment_power_transmitter",
        &["true", "1", "\"true\""],
    ),
    ("minecraft:sound", &["\"stone\"", "5"]),
];

/// A pack allowed the legacy creator features, with Upcoming Creator Features on, at the
/// newest engine: no gate stands between a component and its reader.
fn ungated() -> LoadContext {
    let options = LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    };
    let mut cx = LoadContext::for_pack(
        &options,
        ALLOW_LISTED,
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 60)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// One line of the table: the version, the component, the value, and what the load gave.
fn line(version: &str, key: &str, value: &str, cx: &LoadContext) -> String {
    let text = document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"{key}": {value}}}}}"#
        ),
    );
    let (loaded, diagnostics) = match load(&text, cx) {
        Ok(document) => {
            let shape = newest_shape(&document);
            let written = shape
                .as_object()
                .and_then(|o| o.get("minecraft:block"))
                .and_then(|b| b.as_object())
                .and_then(|b| b.get("components"))
                .and_then(|c| c.as_object())
                .and_then(|c| c.get(key))
                .map(|v| bedrock_addon::json::to_facet_json(v).unwrap_or_default());
            (
                format!("loaded {}", written.unwrap_or_else(|| "-".to_owned())),
                document.diagnostics().clone(),
            )
        }
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                bedrock_addon::diagnostic::testing::shown(d),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!(
        "{version} {key} {value} => {loaded} [{}]",
        findings.join("; ")
    )
}

#[test]
fn the_components_read_every_scalar_as_pinned() {
    let cx = ungated();
    let mut table = String::new();
    for version in ["1.21.60", "1.26.20", "1.26.60"] {
        for (key, values) in VALUES {
            for value in *values {
                table.push_str(&line(version, key, value, &cx));
                table.push('\n');
            }
        }
    }
    let expected = include_str!("component_scalars.txt");
    assert!(table == expected, "--- actual ---\n{table}--- end ---");
}

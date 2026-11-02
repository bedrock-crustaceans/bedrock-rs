//! The validation rules that no fixture row covers, the schema constraints, strict scalar types
//! and the inherent helpers. Findings are matched on code, severity and path (and on vanilla's text where the
//! folder gives one).

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::families::block::component::{
    BlockComponent, CollisionBox, ConnectionRule, CraftingTable, DestructibleByExplosion,
    DestructibleByMining, Friction, Geometry, GeometryParams, InstrumentSound, LightDampening,
    LightEmission, LiquidDetection, MaterialInstanceEntry, MaterialInstances, PlacementFilter,
    RedstoneConsumer, RedstoneProducer, Tags, Transformation,
};
use bedrock_addon::families::block::{Block, Permutation, StateValues};
use bedrock_addon::id::BlockId;
use bedrock_addon::json::{StrictFloat, StrictInt};
use bedrock_addon::molang::{Molang, MolangVersion, RawVersion};

use super::common::{context, document, errors, load, loads, rejected, ucf};

/// A 1.26.20 block with `description` members and `components`.
fn block(description: &str, components: &str) -> String {
    document(
        "1.26.20",
        &format!(
            r#"{{"description": {{"identifier": "demo:x"{description}}}, "components": {components}}}"#
        ),
    )
}

/// The findings of a document, loaded or rejected, and the loaded block.
fn outcome(text: &str, cx: &bedrock_addon::load::LoadContext) -> (Option<Block>, Diagnostics) {
    match load(text, cx) {
        Ok(document) => {
            let d = document.diagnostics().clone();
            (Some(document.into_value()), d)
        }
        Err(rejected) => (None, rejected.diagnostics),
    }
}

/// Whether a finding has `code`, `severity`, and a path ending with `path`.
fn at(d: &Diagnostics, code: fn(&DiagnosticCode) -> bool, severity: Severity, path: &str) -> bool {
    d.iter().any(|f| {
        code(&f.code) && f.severity == severity && f.location.json_path.to_string().ends_with(path)
    })
}

/// A test of a finding's code.
type CodeTest = fn(&DiagnosticCode) -> bool;

fn invalid_enum_value(c: &DiagnosticCode) -> bool {
    matches!(c, DiagnosticCode::InvalidEnumValue { .. })
}

fn invalid_value(c: &DiagnosticCode) -> bool {
    matches!(c, DiagnosticCode::InvalidValue { .. })
}

#[test]
fn rule_1_the_identifier_is_namespaced_and_not_minecraft() {
    for id in ["minecraft:stone", "stone", "demo:", "demo: x"] {
        let text = document(
            "1.26.20",
            &format!(r#"{{"description": {{"identifier": "{id}"}}, "components": {{}}}}"#),
        );
        let d = rejected(&text, &context());
        assert!(
            d.iter().any(|f| matches!(f.code, DiagnosticCode::NamespaceRule { .. })
                && f.severity == Severity::Error
                && f.code.to_string()
                    == "Must have a namespace and can't use the \"minecraft\" one. Example: \"namespace:my_block\""),
            "{id}: {d:#?}"
        );
    }
    // A base-game pack's blocks are `minecraft:`.
    let mut base_game = context();
    base_game.is_base_game_pack = true;
    let vanilla = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:stone"}, "components": {}}"#,
    );
    loads(&vanilla, &base_game);
    rejected(&block("", "{}"), &base_game);
}

#[test]
fn rule_3_state_definitions() {
    let namespace = |c: &DiagnosticCode| matches!(c, DiagnosticCode::NamespaceRule { .. });
    let cases: [(&str, CodeTest); 9] = [
        (r#"{"lit": [false, true]}"#, namespace),
        (r#"{"minecraft:lit": [false, true]}"#, namespace),
        (
            r#"{"demo:a": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]}"#,
            invalid_value,
        ),
        (
            r#"{"demo:a": {"values": {"min": 3, "max": 1}}}"#,
            invalid_value,
        ),
        (
            r#"{"demo:a": {"values": {"min": 0, "max": 16}}}"#,
            invalid_value,
        ),
        (r#"{"demo:a": ["x", "x"]}"#, invalid_value),
        (r#"{"demo:a": [true]}"#, invalid_value),
        (r#"{"demo:a": {"min": 0}}"#, |c| {
            matches!(c, DiagnosticCode::InvalidType { .. })
                && c.to_string() == "Failed to create Block State Description"
        }),
        (r#"{"demo:a": []}"#, |c| {
            matches!(c, DiagnosticCode::InvalidType { .. })
        }),
    ];
    for (states, code) in cases {
        let d = rejected(
            &block(&format!(r#", "states": {states}"#), "{}"),
            &context(),
        );
        assert!(
            at(&d, code, Severity::Error, "description/states/demo:a")
                || at(&d, code, Severity::Error, "description/states/lit")
                || at(
                    &d,
                    code,
                    Severity::Error,
                    "description/states/minecraft:lit"
                ),
            "{states}: {d:#?}"
        );
    }
    let ok = block(
        r#", "states": {"demo:a": [false, true], "demo:b": {"values": {"min": 0, "max": 15}}, "demo:c": {"values": ["x", 1, true]}, "demo:d": ["n", "s"]}"#,
        "{}",
    );
    let loaded = loads(&ok, &context());
    assert_eq!(
        loaded.description.states["demo:b"],
        StateValues::range(0, 15)
    );
    assert_eq!(
        loaded.description.states["demo:a"],
        StateValues::Bools(vec![false, true])
    );
    // A state a trait already adds.
    let clash = block(
        r#", "states": {"minecraft:cardinal_direction": ["north"]}, "traits": {"minecraft:placement_direction": {"enabled_states": ["minecraft:cardinal_direction"]}}"#,
        "{}",
    );
    let d = rejected(&clash, &context());
    assert!(at(
        &d,
        invalid_value,
        Severity::Error,
        "description/states/minecraft:cardinal_direction"
    ));
}

/// Rules 7–10 and 10a: each drops its component with a warning.
#[test]
fn rules_7_to_10_drop_the_component_with_a_warning() {
    let tags: Vec<String> = (0..65).map(|i| format!("\"t{i}\"")).collect();
    let long_tag = "x".repeat(65);
    let instances: Vec<String> = (0..65)
        .map(|i| format!(r#""i{i}": {{"texture": "x"}}"#))
        .collect();
    let bones: Vec<String> = (0..65).map(|i| format!(r#""b{i}": true"#)).collect();
    let filters = vec![r#""demo:x""#; 65].join(", ");
    let conditions = vec![r#"{"allowed_faces": ["up"]}"#; 65].join(", ");
    let uv: Vec<String> = (0..65).map(|i| format!("\"b{i}\"")).collect();
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let cases: Vec<(String, &str)> = vec![
        (
            format!(
                r#"{{"minecraft:crafting_table": {{"crafting_tags": [{}]}}}}"#,
                tags.join(", ")
            ),
            "minecraft:crafting_table/crafting_tags",
        ),
        (
            format!(r#"{{"minecraft:crafting_table": {{"crafting_tags": ["{long_tag}"]}}}}"#),
            "minecraft:crafting_table/crafting_tags/0",
        ),
        (
            r#"{"minecraft:liquid_detection": {"detection_rules": [{}, {}]}}"#.to_owned(),
            "minecraft:liquid_detection/detection_rules",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {{{}}}}}"#,
                instances.join(", ")
            ),
            "minecraft:material_instances",
        ),
        (
            r#"{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {"bad key": {"texture": "x"}}}"#.to_owned(),
            "minecraft:material_instances",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "bone_visibility": {{{}}}}}, {mi}}}"#,
                bones.join(", ")
            ),
            "minecraft:geometry/bone_visibility",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "uv_lock": [{}]}}, {mi}}}"#,
                uv.join(", ")
            ),
            "minecraft:geometry/uv_lock",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "minecraft:geometry.full_block", "bone_visibility": {{"a": true}}}}, {mi}}}"#
            ),
            "minecraft:geometry/bone_visibility",
        ),
        (
            format!(r#"{{"minecraft:placement_filter": {{"conditions": [{conditions}]}}}}"#),
            "minecraft:placement_filter",
        ),
        (
            format!(
                r#"{{"minecraft:placement_filter": {{"conditions": [{{"block_filter": [{filters}]}}]}}}}"#
            ),
            "minecraft:placement_filter",
        ),
        (
            r#"{"demo:deep": {"a": {"b": {"c": 1}}}}"#.to_owned(),
            "demo:deep",
        ),
    ];
    for (components, path) in cases {
        // A Cereal constraint of a 1.26.20 document fails the block; the depth of
        // custom parameters is ours (the game binds no such constraint in production), an
        // information, and the component is kept.
        let severity = if path == "demo:deep" {
            Severity::Info
        } else {
            Severity::Error
        };
        let (block_value, d) = outcome(&block("", &components), &ucf());
        assert!(at(&d, invalid_value, severity, path), "{path}: {d:#?}");
        if let Some(loaded) = block_value {
            let name = path.split('/').next().unwrap_or_default();
            let present = match name {
                "minecraft:crafting_table" => loaded.components.get::<CraftingTable>().is_some(),
                "minecraft:liquid_detection" => {
                    loaded.components.get::<LiquidDetection>().is_some()
                }
                "minecraft:collision_box" => loaded.components.get::<CollisionBox>().is_some(),
                "minecraft:material_instances" => {
                    loaded.components.get::<MaterialInstances>().is_some()
                }
                "minecraft:placement_filter" => {
                    loaded.components.get::<PlacementFilter>().is_some()
                }
                "minecraft:geometry" => loaded.components.get::<Geometry>().is_some(),
                _ => !loaded.components.is_empty(),
            };
            if path == "demo:deep" {
                assert!(present, "{path}: kept");
            } else {
                assert!(!present, "{path}: dropped");
            }
        }
    }
    // Two levels of custom parameters are fine.
    let loaded = loads(&block("", r#"{"demo:ok": {"a": {"b": 1}}}"#), &context());
    assert_eq!(loaded.components.len(), 1);
}

/// Rules 9 and 10 on `minecraft:collision_box` are the reader's constraint since 1.26.60:
/// a 1.26.20 box out of its range, or more than 16 boxes, fails the block (the game's
/// collision description overflow test: the block is not loaded).
#[test]
fn a_collision_box_out_of_its_constraint_fails_a_1_26_20_block() {
    let boxes = vec![r#"{"origin": [-8, 0, -8], "size": [1, 1, 1]}"#; 17].join(", ");
    for components in [
        format!(r#"{{"minecraft:collision_box": [{boxes}]}}"#),
        r#"{"minecraft:collision_box": {"origin": [-9, 0, -8], "size": [8, 8, 8]}}"#.to_owned(),
    ] {
        let d = rejected(&block("", &components), &ucf());
        assert!(
            at(
                &d,
                invalid_value,
                Severity::Error,
                "minecraft:collision_box"
            ),
            "{components}: {d:#?}"
        );
    }
}

/// Rule 9 and the constraints: in a 1.26.20 document a value out of its range fails the
/// block (a Cereal constraint; `tags` drops with a warning); a step clamps an older one.
#[test]
fn rule_9_schema_constraints_drop_the_component() {
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let cases: Vec<(String, &str)> = vec![
        (r#"{"minecraft:friction": 0.95}"#.to_owned(), "minecraft:friction"),
        (r#"{"minecraft:light_emission": 16}"#.to_owned(), "minecraft:light_emission"),
        (r#"{"minecraft:light_dampening": 20}"#.to_owned(), "minecraft:light_dampening"),
        (r#"{"minecraft:redstone_producer": {"power": 16}}"#.to_owned(), "minecraft:redstone_producer/power"),
        (r#"{"minecraft:redstone_consumer": {"min_power": 16}}"#.to_owned(), "minecraft:redstone_consumer/min_power"),
        (r#"{"minecraft:transformation": {"rotation": [0, 45, 0]}}"#.to_owned(), "minecraft:transformation/rotation"),
        (r#"{"minecraft:destructible_by_explosion": {"explosion_resistance": -1}}"#.to_owned(), "minecraft:destructible_by_explosion/explosion_resistance"),
        (r#"{"minecraft:destructible_by_mining": {"seconds_to_destroy": -1}}"#.to_owned(), "minecraft:destructible_by_mining/seconds_to_destroy"),
        (r#"{"minecraft:connection_rule": {"enabled_directions": ["north", "north"]}}"#.to_owned(), "minecraft:connection_rule/enabled_directions"),
        (r#"{"minecraft:instrument_sound": {"left": "note.bit"}}"#.to_owned(), "minecraft:instrument_sound"),
        (r#"{"minecraft:instrument_sound": {}}"#.to_owned(), "minecraft:instrument_sound"),
        (r#"{"minecraft:tags": []}"#.to_owned(), "minecraft:tags"),
        (r#"{"minecraft:tags": [""]}"#.to_owned(), "minecraft:tags"),
        (format!(r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "minecraft:stairs"}}, {mi}}}"#), "minecraft:geometry/culling_shape"),
        (format!(r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "_shape"}}, {mi}}}"#), "minecraft:geometry/culling_shape"),
        (format!(r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "minecraft:culling_layer.glass"}}, {mi}}}"#), "minecraft:geometry/culling_layer"),
        (r#"{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "ambient_occlusion": 11}}}"#.to_owned(), "minecraft:material_instances"),
    ];
    for (components, path) in cases {
        // A Cereal constraint fails the block; `minecraft:tags` has none, so the
        // drop is an information of ours.
        let severity = if path == "minecraft:tags" {
            Severity::Info
        } else {
            Severity::Error
        };
        let (loaded, d) = outcome(&block("", &components), &ucf());
        assert!(at(&d, invalid_value, severity, path), "{path}: {d:#?}");
        if let Some(loaded) = loaded {
            let name = path.split('/').next().unwrap_or_default();
            assert!(
                !loaded.components.iter().any(|c| matches!(c,
                    bedrock_addon::document::ComponentRef::Typed { name: n, .. } if n == name)),
                "{path}: dropped"
            );
        }
    }
    // Accepted shapes.
    let loaded = loads(
        &block(
            "",
            &format!(
                r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "demo:slab", "culling_layer": "minecraft:culling_layer.leaves"}}, {mi},
                    "minecraft:redstone_producer": {{"power": 15}}, "minecraft:transformation": {{"rotation": [0, 270, 90]}},
                    "minecraft:tags": ["demo:ore"], "minecraft:instrument_sound": {{"up": "note.bit"}}}}"#
            ),
        ),
        &ucf(),
    );
    assert!(loaded.components.get::<Geometry>().is_some());
    assert!(loaded.components.get::<RedstoneProducer>().is_some());
    assert!(loaded.components.get::<Transformation>().is_some());
    assert!(loaded.components.get::<Tags>().is_some());
    assert!(loaded.components.get::<InstrumentSound>().is_some());
    // Below 1.26.20 the legacy friction description binds the same range,
    // and a failed constraint refuses the block; inside it, it loads.
    let legacy = |friction: &str| {
        document(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:friction": {friction}}}}}"#
            ),
        )
    };
    let found = rejected(&legacy("0.95"), &context());
    assert!(
        found
            .iter()
            .any(|f| f.code.name() == "InvalidValue" && f.severity == Severity::Error),
        "{found:#?}"
    );
    let loaded = loads(&legacy("0.9"), &context());
    assert_eq!(
        loaded.components.get::<Friction>(),
        Some(&Friction(StrictFloat(0.9)))
    );
    let _: Option<(&RedstoneConsumer, &ConnectionRule, &DestructibleByExplosion)> = None;
    let _: Option<(&LightDampening, &LightEmission)> = None;
}

/// A 1.26.20 document's scalars take only their JSON type
/// (`InvalidType`, the component dropped); the legacy path stays lenient.
#[test]
fn strict_types_reject_a_wrong_json_type_in_a_1_26_20_document() {
    for (components, path) in [
        (r#"{"minecraft:friction": "0.4"}"#, "minecraft:friction"),
        (
            r#"{"minecraft:light_emission": 4.0}"#,
            "minecraft:light_emission",
        ),
        (
            r#"{"minecraft:light_emission": true}"#,
            "minecraft:light_emission",
        ),
        (r#"{"minecraft:display_name": 5}"#, "minecraft:display_name"),
        (
            r#"{"minecraft:collision_box": "true"}"#,
            "minecraft:collision_box",
        ),
        (
            r#"{"minecraft:redstone_conductivity": {"redstone_conductor": 1}}"#,
            "minecraft:redstone_conductivity",
        ),
        (
            r#"{"minecraft:tick": {"interval_range": [1.5, 2]}}"#,
            "minecraft:tick",
        ),
    ] {
        // A typed read that fails in a cereal document rejects the block
        // (0x8 is an error under both masks).
        let d = rejected(&block("", components), &ucf());
        assert!(
            d.iter().any(|f| matches!(
                f.code,
                DiagnosticCode::InvalidType { .. } | DiagnosticCode::InvalidValue { .. }
            ) && f.severity == Severity::Error
                && f.location.json_path.to_string().contains(path)),
            "{components}: {d:#?}"
        );
    }
    let loaded = loads(
        &block(
            "",
            r#"{"minecraft:friction": 0.4, "minecraft:light_emission": 4, "minecraft:collision_box": false}"#,
        ),
        &context(),
    );
    assert_eq!(loaded.components.len(), 3);
    assert_eq!(
        loaded.components.get::<LightEmission>(),
        Some(&LightEmission(StrictInt(4)))
    );
}

#[test]
fn base_game_members_are_unknown_elsewhere() {
    let components = r#"{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "dithering": true}},
        "minecraft:connection_rule": {"vanilla_connections_from": ["fence"]}}"#;
    let unknown = |c: &DiagnosticCode| matches!(c, DiagnosticCode::UnrecognizedField { .. });
    // In a cereal document the hidden members are unknown ones, 0x4 under 0xfffc: the block
    // is refused, as for `internal_vanilla_data`.
    let d = rejected(&block("", components), &ucf());
    assert!(at(&d, unknown, Severity::Error, "*/dithering"), "{d:#?}");
    assert!(
        at(&d, unknown, Severity::Error, "vanilla_connections_from"),
        "{d:#?}"
    );
    // In a legacy document they are silent unknown members (their only binds are the 1.26.20
    // ones): kept as unknown JSON, an information of ours.
    let loaded = loads(
        &document(
            "1.21.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        ),
        &ucf(),
    );
    let Some(MaterialInstanceEntry::Instance(instance)) = loaded
        .components
        .get::<MaterialInstances>()
        .and_then(|m| m.0.get("*"))
    else {
        panic!()
    };
    assert!(instance.dithering.is_none() && instance.extra.get("dithering").is_some());
    let rule = loaded.components.get::<ConnectionRule>().unwrap();
    assert!(
        rule.vanilla_connections_from.is_none()
            && rule.extra.get("vanilla_connections_from").is_some()
    );
    assert!(at(
        loaded.diagnostics(),
        unknown,
        Severity::Info,
        "*/dithering"
    ));
    assert!(at(
        loaded.diagnostics(),
        unknown,
        Severity::Info,
        "vanilla_connections_from"
    ));
    let mut base_game = ucf();
    base_game.is_base_game_pack = true;
    let vanilla = loads(
        &document(
            "1.26.20",
            &format!(
                r#"{{"description": {{"identifier": "minecraft:x"}}, "components": {components}}}"#
            ),
        ),
        &base_game,
    );
    assert!(
        vanilla
            .components
            .get::<ConnectionRule>()
            .unwrap()
            .vanilla_connections_from
            .is_some()
    );
}

#[test]
fn rule_11_a_duplicate_component_keeps_the_first() {
    let text = document(
        "1.21.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.5, "minecraft:friction": 0.8}}"#,
    );
    let loaded = loads(&text, &context());
    assert_eq!(
        loaded.components.get::<Friction>(),
        Some(&Friction(StrictFloat(0.5)))
    );
    assert!(loaded.diagnostics().iter().any(|f| matches!(
        f.code,
        DiagnosticCode::DroppedComponent { .. }
    ) && f.severity == Severity::Error
        && f.kept
        && f.code.to_string()
            == "Duplicate component minecraft:friction found. Please remove one of the duplicates."));
}

#[test]
fn rule_15_trait_checks() {
    let traits = |traits: &str, version: &str| {
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x", "traits": {traits}}}, "components": {{}}}}"#
            ),
        )
    };
    let missing = |c: &DiagnosticCode| matches!(c, DiagnosticCode::MissingRequiredField { .. });
    let toggles = |c: &DiagnosticCode| matches!(c, DiagnosticCode::MissingToggles { .. });
    let cases: [(&str, &str, CodeTest, &str); 6] = [
        (
            r#"{"minecraft:placement_position": {"enabled_states": []}}"#,
            "1.26.20",
            missing,
            "\"minecraft:placement_position\" trait requires at least one 'enabled_state' to be specified.",
        ),
        (
            r#"{"minecraft:placement_position": {"enabled_states": ["minecraft:upside"]}}"#,
            "1.26.20",
            invalid_value,
            "Invalid state option \"minecraft:upside\" for \"minecraft:placement_position\" trait.",
        ),
        (
            r#"{"minecraft:placement_direction": {"enabled_states": ["minecraft:cardinal_direction"]}}"#,
            "1.20.10",
            toggles,
            "To use trait 'minecraft:placement_direction' in json formats earlier than 1.20.20, turn on the 'upcoming_creator_features' toggle in settings",
        ),
        (
            r#"{"minecraft:connection": {"enabled_states": ["minecraft:cardinal_connections"]}}"#,
            "1.21.90",
            toggles,
            "To use trait 'minecraft:connection' in json formats earlier than 1.26.0, turn on the 'upcoming_creator_features' toggle in settings",
        ),
        (
            r#"{"minecraft:placement_direction": {"enabled_states": ["minecraft:cardinal_direction"], "blocks_to_corner_with": ["demo:x"]}}"#,
            "1.26.20",
            invalid_value,
            "\"blocks_to_corner_with\" can only be used when \"minecraft:corner_and_cardinal_direction\" is enabled in \"enabled_states\".",
        ),
        (
            r#"{"demo:spin": {}}"#,
            "1.26.20",
            invalid_enum_value,
            "Failed to find BlockTrait with identifier: 'demo:spin'",
        ),
    ];
    for (json, version, code, text) in cases {
        let d = rejected(&traits(json, version), &context());
        assert!(
            d.iter().any(|f| code(&f.code)
                && f.severity == Severity::Error
                && f.code.to_string() == text),
            "{text}: {d:#?}"
        );
    }
    // The multi-block trait is released by 1.26.20 (`multi_block-1`):
    // gated below it only.
    let d = rejected(
        &traits(
            r#"{"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}"#,
            "1.26.0",
        ),
        &context(),
    );
    assert!(
        d.iter().any(|f| toggles(&f.code)
            && f.code.to_string()
                == "To use trait 'minecraft:multi_block' in json formats earlier than 1.26.40, turn on the 'upcoming_creator_features' toggle in settings"),
        "{d:#?}"
    );
    let loaded = loads(
        &traits(
            r#"{"minecraft:connection": {"enabled_states": ["minecraft:cardinal_connections"]}, "minecraft:placement_position": {"enabled_states": ["minecraft:vertical_half"]}}"#,
            "1.26.20",
        ),
        &context(),
    );
    let states = loaded.states_with_traits();
    assert_eq!(
        states.get("minecraft:connection_north"),
        Some(&StateValues::bools())
    );
    assert!(states.contains_key("minecraft:vertical_half"));
    assert_eq!(loaded.permutation_count(), 2 * 2 * 2 * 2 * 2);
}

/// engine-gated members newer than the pack's `min_engine_version` are
/// `FieldSinceEngine` warnings.
#[test]
fn engine_gated_members_newer_than_the_pack_are_reported() {
    // A base-game pack: `vertical_connections` is base-game only.
    let mut cx = super::common::pack_version(1, 26, 20);
    cx.is_base_game_pack = true;
    let since = |c: &DiagnosticCode| matches!(c, DiagnosticCode::FieldSinceEngine { .. });
    let loaded = loads(
        &block(
            r#", "traits": {"minecraft:placement_direction": {"enabled_states": ["minecraft:sixteen_way_rotation"]}, "minecraft:connection": {"enabled_states": ["minecraft:vertical_connections"]}}"#,
            r#"{"minecraft:flammable": {"lava_flammable": "always"}}"#,
        )
        .replace("demo:x", "minecraft:x"),
        &cx,
    );
    let d = loaded.diagnostics();
    assert!(
        at(
            d,
            since,
            Severity::Info,
            "minecraft:placement_direction/enabled_states"
        ),
        "{d:#?}"
    );
    assert!(
        at(
            d,
            since,
            Severity::Info,
            "minecraft:connection/enabled_states"
        ),
        "{d:#?}"
    );
    assert!(
        at(
            d,
            since,
            Severity::Info,
            "minecraft:flammable/lava_flammable"
        ),
        "{d:#?}"
    );
    let newer = loads(
        &block(
            "",
            r#"{"minecraft:flammable": {"lava_flammable": "always"}}"#,
        ),
        &super::common::pack_version(1, 26, 50),
    );
    assert!(newer.diagnostics().is_empty(), "{:#?}", newer.diagnostics());
}

#[test]
fn rule_15a_legacy_structural_errors() {
    let invalid_type = |c: &DiagnosticCode| matches!(c, DiagnosticCode::InvalidType { .. });
    let cases: [(&str, &str); 8] = [
        (
            r#"{"components": {}}"#,
            " -> minecraft:block:  missing required field",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": []}"#,
            " -> minecraft:block:  the 'components' member should be an object",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": {}}"#,
            " -> minecraft:block:  the 'permutations' member should be an array",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": {}}"#,
            "Permutations requires an array object",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1"}]}"#,
            " -> minecraft:block -> permutations:  every permutation should be an object containing a 'components' member object",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [1]}"#,
            "Permutations requires an array of objects",
        ),
        (
            r#"{"description": {"identifier": "demo:x", "states": []}, "components": {}}"#,
            "Block description states must be an object",
        ),
        (
            r#"{"description": {"identifier": "demo:x", "states": []}, "components": {}}"#,
            "Failed to parse block 'demo:x' ",
        ),
    ];
    for (payload, text) in cases {
        let d = rejected(&document("1.21.0", payload), &context());
        assert!(
            d.iter().any(|f| invalid_type(&f.code)
                && f.severity == Severity::Error
                && f.code.to_string() == text),
            "{text}: {d:#?}"
        );
    }
    // `description.use_beta_features` is no member of 1.26.60: no
    // beta gate; the member is an unknown one, a warning of the legacy reader.
    let beta = loads(
        &document(
            "1.21.0",
            r#"{"description": {"identifier": "demo:x", "use_beta_features": true}, "components": {}}"#,
        ),
        &context(),
    );
    let d = beta.diagnostics();
    assert!(
        !d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::BetaFeaturesNotEnabled { .. })),
        "{d:#?}"
    );
    assert!(
        d.iter().any(
            |f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. })
                && f.severity == Severity::Warning
                && f.location.json_path.to_string()
                    == "minecraft:block/description/use_beta_features"
        ),
        "{d:#?}"
    );
    let d = rejected(
        &document(
            "1.21.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": {"e": {"condition": 1}}}"#,
        ),
        &super::common::allow_listed(),
    );
    assert!(
        d.iter()
            .any(|f| invalid_type(&f.code) && f.code.to_string() == "Invalid, nonstring Json ")
    );
}

#[test]
fn internal_vanilla_data_is_an_unknown_member_outside_the_base_game() {
    // A cereal document of another pack fails (`loader-10`);
    // the legacy reader warns and keeps it as an unknown member.
    let d = rejected(
        &block(
            r#", "internal_vanilla_data": {"block_id": 7, "material": "plant"}"#,
            "{}",
        ),
        &context(),
    );
    assert!(at(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Error,
        "internal_vanilla_data"
    ));
    let text = document(
        "1.21.0",
        r#"{"description": {"identifier": "demo:x", "internal_vanilla_data": {"block_id": 7, "material": "plant"}}, "components": {}}"#,
    );
    let loaded = loads(&text, &context());
    assert!(loaded.description.internal_vanilla_data.is_none());
    assert!(
        loaded
            .description
            .extra
            .get("internal_vanilla_data")
            .is_some()
    );
    // The legacy description loader reads by name and says nothing of it: an
    // information of ours.
    assert!(at(
        loaded.diagnostics(),
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Info,
        "internal_vanilla_data"
    ));
    let mut base_game = context();
    base_game.is_base_game_pack = true;
    let vanilla = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:x", "internal_vanilla_data": {"block_id": 7, "material": "solid_plant"}}, "components": {}}"#,
    );
    let data = loads(&vanilla, &base_game)
        .description
        .internal_vanilla_data
        .clone()
        .unwrap();
    assert_eq!(data.block_id, Some(StrictInt(7)));
    assert_eq!(
        data.material,
        Some(bedrock_addon::families::block::MaterialType::SolidPlant)
    );
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_molang_feature_expressions_are_counted_not_checked() {
    let text = block(
        "",
        r#"{}, "permutations": [{"condition": "q.block_state('demo:a')", "components": {}}]"#,
    );
    let loaded = loads(&text, &context());
    assert!(loaded.diagnostics().iter().any(|f| matches!(
        f.code,
        DiagnosticCode::MolangNotChecked { .. }
    ) && f.severity == Severity::Info));
}

#[test]
fn the_constructors_build_a_valid_block() {
    let mut ruby = Block::new(BlockId::from_static("demo:ruby_lamp"));
    ruby.description
        .states
        .insert("demo:lit".to_owned(), StateValues::bools());
    ruby.components.insert(DestructibleByMining::seconds(1.5));
    ruby.permutations
        .push(Permutation::when("q.block_state('demo:lit')").with(LightEmission(StrictInt(15))));
    assert_eq!(
        ruby.permutations[0]
            .condition
            .source()
            .and_then(|s| s.raw_version()),
        Some(RawVersion::from(MolangVersion::V13))
    );
    assert!(
        errors(&ruby.validate(&context())).is_empty(),
        "{:#?}",
        ruby.validate(&context())
    );
    assert_eq!(ruby.permutation_count(), 2);
    let path = bedrock_addon::document::Placed::default_path(&ruby);
    assert_eq!(path.as_str(), "blocks/demo/ruby_lamp.json");
    // Written and read back.
    let written = bedrock_addon::document::Content::to_json(&ruby).unwrap();
    let back = loads(&written, &context());
    assert_eq!(back.value(), &ruby);
    // A broken one reports what loading would.
    let mut broken = Block::new(BlockId::parse("minecraft:ruby"));
    broken
        .permutations
        .push(Permutation::new(Molang::Const(1.0)));
    let d = broken.validate(&context());
    assert!(at(
        &d,
        |c| matches!(c, DiagnosticCode::NamespaceRule { .. }),
        Severity::Error,
        "description/identifier"
    ));
    let _: Option<(BlockComponent, GeometryParams)> = None;
}

/// `instrument_sound` in a cereal document needs no toggle in 1.26.60 (the older gate and its two
/// texts are gone) and is kept and written back.
#[test]
fn instrument_sound_is_ungated_in_a_cereal_document() {
    let base = |version: &str| {
        document(
            version,
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:instrument_sound": {"up": "note.bassattack"}}}"#,
        )
    };
    // In a pack for engine 1.26.50, as vanilla's (an older pack gets `FieldSinceEngine`).
    let vanilla = super::common::pack_version(1, 26, 50);
    for version in ["1.26.30", "1.26.50"] {
        let loaded = loads(&base(version), &vanilla);
        assert!(
            loaded.diagnostics().is_empty(),
            "{version}: {:#?}",
            loaded.diagnostics()
        );
        assert!(loaded.components.get::<InstrumentSound>().is_some());
        let written = bedrock_addon::document::Content::to_json(loaded.value()).unwrap();
        assert!(
            written.contains("\"minecraft:instrument_sound\""),
            "{written}"
        );
        assert_eq!(loads(&written, &vanilla).value(), loaded.value());
    }
}

/// The `determinism/` fragment the determinism test picks up is a valid block.
#[test]
fn the_determinism_fragment_loads_cleanly() {
    let text = super::common::fixture("determinism/behavior_pack/blocks/demo/ruby_lamp.json");
    let block = loads(&text, &context());
    assert!(
        block
            .diagnostics()
            .at_least(Severity::Warning)
            .next()
            .is_none(),
        "{:#?}",
        block.diagnostics()
    );
}

#[test]
fn the_menu_category_needs_a_category_and_a_namespaced_group() {
    for (menu, path, code) in [
        (
            r#"{"group": "minecraft:itemGroup.name.ore"}"#,
            "menu_category/category",
            (|c: &DiagnosticCode| matches!(c, DiagnosticCode::MissingRequiredField { .. }))
                as fn(&DiagnosticCode) -> bool,
        ),
        (
            r#"{"category": "nature", "group": "my group"}"#,
            "menu_category/group",
            |c: &DiagnosticCode| matches!(c, DiagnosticCode::NamespaceRule { .. }),
        ),
        (
            r#"{"category": "nature", "group": "plain"}"#,
            "menu_category/group",
            |c: &DiagnosticCode| matches!(c, DiagnosticCode::NamespaceRule { .. }),
        ),
    ] {
        let d = rejected(
            &block(&format!(r#", "menu_category": {menu}"#), "{}"),
            &context(),
        );
        assert!(at(&d, code, Severity::Error, path), "{menu}: {d:#?}");
    }
    loads(
        &block(
            r#", "menu_category": {"category": "nature", "group": "demo:group"}"#,
            "{}",
        ),
        &context(),
    );
}

#[test]
fn shared_descriptors_report_their_own_findings() {
    let text = block(
        "",
        r#"{"minecraft:placement_filter": {"conditions": [{"block_filter": [{"name": 5}]}]}}"#,
    );
    let loaded = loads(&text, &context());
    assert!(
        loaded.diagnostics().iter().any(|f| invalid_value(&f.code)
            && f.severity == Severity::Error
            && f.location.json_path.to_string().ends_with("block_filter/0")),
        "{:#?}",
        loaded.diagnostics()
    );
}

/// a vanilla block of format 1.26.50 that uses `minecraft:instrument_sound`, loaded as
/// the base game loads it, rewrites byte for byte: write, parse, write gives the same text. The
/// writer is not canonical to the input: it writes struct members in declaration order; apart from that
/// the written text is the input's JSON value, with no member the file does not have
/// (`redstone_conductivity`'s `allows_wire_to_step_down` is written only when the file has it).
// Fixture: the 1.26.50 vanilla behavior pack's
// `blocks/red_concrete_double_slab.block.json`, copied unchanged.
#[test]
fn a_vanilla_1_26_50_block_with_instrument_sound_rewrites_byte_stable() {
    use bedrock_addon::document::Content;
    use bedrock_addon::json::Value;

    let input = super::common::fixture("vanilla/red_concrete_double_slab.block.json");
    let mut cx = bedrock_addon::load::LoadContext::for_pack(
        &bedrock_addon::load::LoadOptions::default(),
        uuid::Uuid::from_u128(7),
        bedrock_addon::pack::PackType::Behavior,
        Some(bedrock_addon::version::MinEngineVersion::new(1, 26, 50)),
        true,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    let first = loads(&input, &cx);
    assert!(first.diagnostics().is_empty(), "{:#?}", first.diagnostics());
    assert!(first.components.get::<InstrumentSound>().is_some());
    let written = first.value().to_json().unwrap();
    let second = loads(&written, &cx);
    assert_eq!(second.value(), first.value());
    let rewritten = second.value().to_json().unwrap();
    assert_eq!(rewritten, written, "write, parse, write is byte-stable");
    let read = |text: &str| facet_json::from_str::<Value>(text).unwrap();
    assert_eq!(
        sorted(&read(&written)),
        sorted(&read(&input)),
        "the writer gives the input back, member order and formatting aside:\n{written}"
    );
}

/// `value` as compact JSON text with every object's members sorted by key.
fn sorted(value: &bedrock_addon::json::Value) -> String {
    if let Some(object) = value.as_object() {
        let mut members: Vec<_> = object.iter().collect();
        members.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        let inner: Vec<String> = members
            .into_iter()
            .map(|(k, v)| format!("{:?}:{}", k.as_str(), sorted(v)))
            .collect();
        format!("{{{}}}", inner.join(","))
    } else if let Some(array) = value.as_array() {
        let inner: Vec<String> = array.iter().map(sorted).collect();
        format!("[{}]", inner.join(","))
    } else {
        facet_json::to_string(value).unwrap()
    }
}

// Facet reports an object given to an identifier as `Unsupported`; it is a wrong type at the
// member and must not panic.
#[test]
fn an_identifier_that_is_an_object_is_a_type_error_and_no_panic() {
    let d = rejected(
        &document(
            "1.26.20",
            r#"{"description": {"identifier": {}}, "components": {}}"#,
        ),
        &context(),
    );
    let found: Vec<_> = d
        .iter()
        .filter(|f| matches!(f.code, DiagnosticCode::InvalidType { .. }))
        .collect();
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:block/description/identifier"
    );
}

// The block family reads its documents as written
// (rule 11 needs to see a doubled component); every other member written twice keeps its
// last value, as the core rule says.
#[test]
fn a_member_written_twice_keeps_its_last_value() {
    for version in ["1.21.0", "1.26.20"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:block": {{"description": {{"identifier": "demo:first", "identifier": "demo:last"}}, "components": {{}}}}}}"#
        );
        let block = loads(&text, &context());
        assert_eq!(
            block.description.identifier.as_str(),
            "demo:last",
            "{version}"
        );
        assert!(errors(block.diagnostics()).is_empty(), "{version}");
    }
}

// ---- a block of a file below 1.26.20 that the shape of 1.26.20 does not hold ----

/// What a block of `version` with `components` says in the upgrade phase, and what the writer
/// makes of it.
fn told_and_written(
    version: &str,
    components: &str,
) -> (Vec<(String, String)>, Result<String, String>) {
    use bedrock_addon::diagnostic::Phase;
    use bedrock_addon::document::Content;
    let text = document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
        ),
    );
    let loaded = loads(&text, &context());
    let told = loaded
        .diagnostics()
        .iter()
        .filter(|f| f.phase == Phase::Upgrade && f.severity == Severity::Info)
        .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
        .collect();
    (told, loaded.to_json().map_err(|error| format!("{error:?}")))
}

/// A legacy block is held in the shape of 1.26.20. A value only a file of 1.26.20 is held to a
/// rule for (rule 9: `friction` in 0..=0.9, a light level up to 15, …) loads, as the game
/// reads the file at its own version, and the load says so at the member. Where the block
/// holds legacy components of an allow-listed pack (kept beside the components), they are
/// written back in their legacy shape at that version.
#[test]
fn a_legacy_block_the_newest_format_refuses_with_legacy_components_is_written_in_its_legacy_shape()
{
    use bedrock_addon::diagnostic::Phase;
    use bedrock_addon::document::Content;
    for (components, at) in [
        (
            r#""minecraft:destructible_by_explosion": {"explosion_resistance": -1}"#,
            "minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance",
        ),
        (
            r#""minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 20, 16]}"#,
            "minecraft:block/components/minecraft:selection_box",
        ),
    ] {
        let text = document(
            "1.21.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}, "minecraft:breathability": "air"}}}}"#
            ),
        );
        let loaded = loads(&text, &super::common::allow_listed());
        assert!(!loaded.deprecated.is_empty(), "{components}");
        let told: Vec<(String, String)> = loaded
            .diagnostics()
            .iter()
            .filter(|f| f.phase == Phase::Upgrade && f.severity == Severity::Info)
            .filter(|f| f.code.name() == "InvalidValue")
            .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
            .collect();
        assert_eq!(
            told,
            [("InvalidValue".to_owned(), at.to_owned())],
            "{components}"
        );
        // The legacy component is written back in its legacy shape at the block's own
        // version, where the rule does not hold, and the text reads back as the block
        // (the side-car has a legacy form, so the block has a text).
        let written = loaded.to_json().expect("a text of 1.21.60");
        assert!(
            written.contains(r#""format_version": "1.21.60""#)
                && written.contains(r#""minecraft:breathability": "air""#),
            "{components}: {written}"
        );
        let again = loads(&written, &super::common::allow_listed());
        assert_eq!(again.value(), loaded.value(), "{components}");
        let _ = at;
    }
}

/// A legacy block that a rule of 1.26.20 refuses is written at its own format version, where
/// the game holds it to no such rule: the text reads back as the same block with the same
/// warning, and is written as itself.
#[test]
fn a_legacy_block_the_newest_format_refuses_is_written_at_its_own_format_version_and_reads_back() {
    use bedrock_addon::diagnostic::Phase;
    use bedrock_addon::document::Content;
    for (version, components, at) in [
        (
            "1.21.60",
            r#""minecraft:destructible_by_explosion": {"explosion_resistance": -1}"#,
            "minecraft:block/components/minecraft:destructible_by_explosion/explosion_resistance",
        ),
        (
            "1.21.60",
            r#""minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 20, 16]}"#,
            "minecraft:block/components/minecraft:selection_box",
        ),
        (
            "1.26.0",
            r#""minecraft:light_dampening": 16"#,
            "minecraft:block/components/minecraft:light_dampening",
        ),
    ] {
        let text = document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        );
        let told = |loaded: &bedrock_addon::document::Document<Block>| -> Vec<(String, String)> {
            loaded
                .diagnostics()
                .iter()
                .filter(|f| f.phase == Phase::Upgrade && f.severity == Severity::Info)
                .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
                .collect()
        };
        let loaded = loads(&text, &context());
        assert_eq!(
            told(&loaded),
            [("InvalidValue".to_owned(), at.to_owned())],
            "{text}"
        );
        let written = loaded
            .to_json()
            .unwrap_or_else(|error| panic!("{text} is not written: {error:?}"));
        assert!(
            written.contains(&format!(r#""format_version": "{version}""#)),
            "{written}"
        );
        let again = loads(&written, &context());
        assert_eq!(again.value(), loaded.value(), "{written}");
        assert_eq!(told(&again), told(&loaded), "{written}");
        assert_eq!(again.to_json().ok(), Some(written));
    }
}

/// The bounds themselves, and a block the rules have nothing against, are written.
#[test]
fn a_legacy_block_within_the_rules_of_the_newest_format_is_written_and_reads_back() {
    use bedrock_addon::document::Content;
    for components in [
        r#""minecraft:friction": 0"#,
        r#""minecraft:friction": 0.9"#,
        r#""minecraft:light_emission": 15"#,
        r#""minecraft:destructible_by_explosion": {"explosion_resistance": 0}"#,
        r#""minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 16, 16]}"#,
    ] {
        let (told, written) = told_and_written("1.21.60", components);
        assert!(told.is_empty(), "{components}: {told:?}");
        let written = written.expect("a block within the rules is written");
        let again = loads(&written, &context());
        assert_eq!(again.to_json().ok(), Some(written), "{components}");
    }
}

/// In a file of 1.26.20 the rule drops the component at the load (rule 9), and what is left is
/// written: nothing of the upgrade phase.
#[test]
fn a_block_of_the_newest_format_is_held_to_its_rules_at_the_load() {
    // A Cereal constraint of a 1.26.20 document fails the block at the load.
    let d = rejected(
        &document(
            "1.26.20",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": -1}}"#,
        ),
        &context(),
    );
    assert!(
        at(&d, invalid_value, Severity::Error, "minecraft:friction"),
        "{d:#?}"
    );
}

/// A rule's finding of a file below 1.26.20 is under the payload's key, where the file has the
/// member and where the parse findings of such a file are.
#[test]
fn a_rule_of_a_legacy_block_reports_under_the_payload_key() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:liquid_detection": {"detection_rules": []}}}"#,
    );
    // The legacy description binds `minSize(1)` / `maxSize(1)` of the rules:
    // the block is refused.
    let diagnostics = rejected(&text, &context());
    let at: Vec<String> = diagnostics
        .iter()
        .filter(|f| f.severity >= Severity::Error)
        .map(|f| f.location.json_path.to_string())
        .filter(|path| path.contains("liquid_detection"))
        .collect();
    assert_eq!(
        at,
        ["minecraft:block/components/minecraft:liquid_detection/detection_rules"]
    );
}

// ---- the checks of core on a legacy block report below the payload key ----

/// A file below 1.26.20 is read whole, its block is checked in the shape of 1.26.20: the
/// findings of core's component gates and of the Molang compile check are at the members the
/// file has, below `minecraft:block`, as the legacy reader's and the rules' are.
#[cfg(feature = "molang")]
#[test]
fn the_generic_checks_of_a_legacy_block_report_below_the_payload_key() {
    let mut cx = context();
    cx.molang = Some(std::sync::Arc::new(
        bedrock_addon::molang::MolangCheck::new(),
    ));
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "demo:x", "states": {"demo:s": [0, 1]}},
            "components": {"minecraft:friction": 0.5},
            "permutations": [
                {"condition": "q.block_state('demo:s') == ", "components": {"minecraft:connection_rule": {}}}
            ]}"#,
    );
    let found = match load(&text, &cx) {
        Ok(document) => document.diagnostics().clone(),
        Err(rejected) => rejected.diagnostics,
    };
    let at = |code: fn(&DiagnosticCode) -> bool| -> Vec<String> {
        found
            .iter()
            .filter(|f| code(&f.code))
            .map(|f| f.location.json_path.to_string())
            .collect()
    };
    // Core's gate of a component (`versioning::validate_components`).
    assert_eq!(
        at(|c| matches!(c, DiagnosticCode::MissingToggles { .. })),
        ["minecraft:block/permutations/0/components/minecraft:connection_rule"]
    );
    // Molang pass 2 (`versioning::load::molang_pass_two`).
    assert_eq!(
        at(|c| matches!(c, DiagnosticCode::InvalidMolang { .. })),
        ["minecraft:block/permutations/0/condition"]
    );
    // And no finding of the load names a member the file does not have at its root.
    for finding in found.iter() {
        let path = finding.location.json_path.to_string();
        assert!(
            path.is_empty() || path == "format_version" || path.starts_with("minecraft:block"),
            "{} at `{path}`",
            finding.code.name()
        );
    }
}

// ---- what is told of a permutation is at the index the file has it at -------------------------

/// The paths below `minecraft:block/permutations/` of what `found` tells, sorted, each once.
fn permutation_paths(found: &Diagnostics) -> Vec<String> {
    let mut paths: Vec<String> = found
        .iter()
        .filter_map(|finding| {
            let path = finding.location.json_path.to_string();
            Some(
                path.strip_prefix("minecraft:block/permutations/")?
                    .to_owned(),
            )
        })
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// `cx` with the Molang compile check (with the `molang` feature).
fn compiling(cx: bedrock_addon::load::LoadContext) -> bedrock_addon::load::LoadContext {
    #[cfg(feature = "molang")]
    {
        let mut cx = cx;
        cx.molang = Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        ));
        cx
    }
    #[cfg(not(feature = "molang"))]
    cx
}

const LIT: &str = r#""description": {"identifier": "demo:x", "states": {"demo:lit": [false, true]}}, "components": {}"#;
const LEGACY_ONLY: &str = r#"{"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}}"#;
const UNRELEASED: &str = r#"{"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:sound": "stone"}}"#;

/// The last step of the chain takes out a permutation with a legacy component of a pack
/// that is not allow-listed. What the reading of the converted block, the checks and the
/// rules of 1.26.20 tell of a permutation behind it is at the index the file has it at.
#[test]
fn a_finding_behind_a_permutation_a_legacy_component_skips_is_at_the_index_of_the_file() {
    let text = document(
        "1.21.60",
        &format!(
            r#"{{{LIT}, "permutations": [
            {LEGACY_ONLY},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{
                "minecraft:destructible_by_explosion": {{"explosion_resistance": -1, "bogus": 1}}}}}},
            {LEGACY_ONLY},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{
                "minecraft:light_dampening": 16}}}}]}}"#
        ),
    );
    let loaded = loads(&text, &compiling(context()));
    assert_eq!(loaded.permutations.len(), 2);
    assert_eq!(
        permutation_paths(loaded.diagnostics()),
        [
            "0/components/minecraft:breathability",
            "1/components/minecraft:destructible_by_explosion/bogus",
            "1/components/minecraft:destructible_by_explosion/explosion_resistance",
            "2/components/minecraft:breathability",
            "3/components/minecraft:light_dampening",
        ],
        "{:#?}",
        loaded
            .diagnostics()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// The same for a condition the compile pass rejects, which rejects the block: the errors
/// are at the permutation the file has.
#[cfg(feature = "molang")]
#[test]
fn a_rejected_condition_behind_a_permutation_a_legacy_component_skips_is_at_the_index_of_the_file()
{
    let text = document(
        "1.21.60",
        &format!(
            r#"{{{LIT}, "permutations": [
            {LEGACY_ONLY},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{}}}},
            {{"condition": "q.is_baby", "components": {{}}}}]}}"#
        ),
    );
    let found = rejected(&text, &compiling(context()));
    assert_eq!(
        permutation_paths(&found),
        ["0/components/minecraft:breathability", "2/condition"],
        "{:#?}",
        found.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
}

/// A rule of the load takes out a permutation (`minecraft:sound`, whose legacy row needs
/// the experiment below 1.26.20). What the rules of 1.26.20 then tell of a block of an
/// older file is at the index the file has the permutation at.
#[test]
fn a_finding_behind_a_permutation_a_rule_skips_is_at_the_index_of_the_file() {
    let text = document(
        "1.21.60",
        &format!(
            r#"{{{LIT}, "permutations": [
            {UNRELEASED},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{"minecraft:friction": 0.95}}}},
            {UNRELEASED},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{
                "minecraft:friction": 0.95, "minecraft:light_emission": 16}}}}]}}"#
        ),
    );
    // A permutation's `minecraft:sound` the legacy registry gates fails the component load: the
    // block is refused, the findings stay at the file's indices.
    let found = rejected(&text, &compiling(context()));
    assert_eq!(
        permutation_paths(&found),
        [
            "0",
            "0/components/minecraft:sound",
            "1/components/minecraft:friction",
            "2",
            "2/components/minecraft:sound",
            "3/components/minecraft:friction",
            "3/components/minecraft:light_emission",
        ],
        "{:#?}",
        found.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
}

/// Both at once: the step takes out two permutations and a rule two of those left. Every
/// index is the file's, the skipped permutations' own too.
#[test]
fn a_finding_behind_permutations_a_step_and_a_rule_skip_is_at_the_index_of_the_file() {
    let text = document(
        "1.21.60",
        &format!(
            r#"{{{LIT}, "permutations": [
            {LEGACY_ONLY},
            {UNRELEASED},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{"minecraft:friction": 0.95}}}},
            {LEGACY_ONLY},
            {UNRELEASED},
            {{"condition": "q.block_state('demo:lit') == true", "components": {{
                "minecraft:friction": 0.95, "minecraft:light_emission": 16}}}}]}}"#
        ),
    );
    // A permutation's `minecraft:sound` the legacy registry gates fails the component load: the
    // block is refused, the findings stay at the file's indices.
    let found = rejected(&text, &compiling(context()));
    assert_eq!(
        permutation_paths(&found),
        [
            "0/components/minecraft:breathability",
            "1",
            "1/components/minecraft:sound",
            "2/components/minecraft:friction",
            "3/components/minecraft:breathability",
            "4",
            "4/components/minecraft:sound",
            "5/components/minecraft:friction",
            "5/components/minecraft:light_emission",
        ],
        "{:#?}",
        found.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
}

/// A block the writer refuses (it holds what no file has, and a member the newest format
/// refuses) is refused at the member the file has: behind a permutation the load took out,
/// at the index of the file.
#[cfg(feature = "molang")]
#[test]
fn the_writer_refuses_a_member_behind_a_skipped_permutation_at_the_index_of_the_file() {
    use bedrock_addon::document::Content;
    let text = document(
        "1.19.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [
            {"condition": "q.is_baby", "components": {}},
            {"condition": "1.0", "components": {"minecraft:light_dampening": 16}}]}"#,
    );
    let loaded = loads(&text, &compiling(super::common::allow_listed()));
    assert_eq!(loaded.permutations.len(), 1);
    assert_eq!(
        permutation_paths(loaded.diagnostics()),
        ["0", "0/condition", "1/components/minecraft:light_dampening"],
        "{:#?}",
        loaded
            .diagnostics()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let error = format!("{:?}", loaded.to_json().expect_err("no text"));
    assert!(
        error.contains(
            r#"Key("permutations"), Index(1), Key("components"), Key("minecraft:light_dampening")"#
        ),
        "{error}"
    );
}

/// Rule N2: a 1.26.20+ block is read from a
/// cereal document input, whose reader finds a member written as `null` and fails its type.
#[test]
fn a_null_optional_member_of_a_cereal_block_rejects_the_document() {
    let mut base_game = context();
    base_game.is_base_game_pack = true;
    let null = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:x", "internal_vanilla_data": {"requires_correct_tool_for_drops": null}}, "components": {}}"#,
    );
    let d = rejected(&null, &base_game);
    assert!(
        at(
            &d,
            |c| matches!(c, DiagnosticCode::InvalidType { .. }),
            Severity::Error,
            "description/internal_vanilla_data/requires_correct_tool_for_drops"
        ),
        "{d:#?}"
    );
    // Absent, the member takes its default, as before.
    let absent = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:x", "internal_vanilla_data": {}}, "components": {}}"#,
    );
    let data = loads(&absent, &base_game)
        .description
        .internal_vanilla_data
        .clone()
        .unwrap();
    assert_eq!(data.requires_correct_tool_for_drops, None);
}

/// The legacy descriptions bind the same constraints as the 1.26.20 components for crafting
/// table (list `maxSize(64)`, tag `maxSize(64)`), liquid detection (`minSize(1)`,
/// `maxSize(1)`), placement filter (both lists `maxSize(64)`) and geometry
/// (`bone_visibility` `maxSize(64)`; the geometry constraint logs "invalid bones: cannot use
/// bone visibility feature with: " with code 0x40). A failed constraint is a level-3 failure that refuses the block,
/// so the drop of a legacy document is an Error, not an information.
#[test]
fn a_legacy_description_constraint_refuses_the_block() {
    let tags: Vec<String> = (0..65).map(|i| format!("\"t{i}\"")).collect();
    let bones: Vec<String> = (0..65).map(|i| format!(r#""b{i}": true"#)).collect();
    let filters = vec![r#""demo:x""#; 65].join(", ");
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let cases: Vec<(String, &str)> = vec![
        (
            format!(
                r#"{{"minecraft:crafting_table": {{"crafting_tags": [{}]}}}}"#,
                tags.join(", ")
            ),
            "minecraft:crafting_table/crafting_tags",
        ),
        (
            format!(
                r#"{{"minecraft:crafting_table": {{"crafting_tags": ["{}"]}}}}"#,
                "x".repeat(65)
            ),
            "minecraft:crafting_table/crafting_tags/0",
        ),
        (
            r#"{"minecraft:liquid_detection": {"detection_rules": [{}, {}]}}"#.to_owned(),
            "minecraft:liquid_detection/detection_rules",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "geometry.x", "bone_visibility": {{{}}}}}, {mi}}}"#,
                bones.join(", ")
            ),
            "minecraft:geometry/bone_visibility",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "minecraft:geometry.full_block", "bone_visibility": {{"a": true}}}}, {mi}}}"#
            ),
            "minecraft:geometry/bone_visibility",
        ),
        (
            format!(
                r#"{{"minecraft:geometry": {{"identifier": "minecraft:geometry.full_block_v1", "bone_visibility": {{"a": true}}}}, {mi}}}"#
            ),
            "minecraft:geometry/bone_visibility",
        ),
        (
            format!(
                r#"{{"minecraft:placement_filter": {{"conditions": [{{"block_filter": [{filters}]}}]}}}}"#
            ),
            "minecraft:placement_filter",
        ),
    ];
    for (components, path) in cases {
        let text = document(
            "1.21.90",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        );
        let d = rejected(&text, &ucf());
        assert!(
            at(&d, invalid_value, Severity::Error, path),
            "{path}: {d:#?}"
        );
    }
}

/// A legacy `uv_lock` list binds no size constraint (the game binds the locked UVs
/// with none, where the 1.26.20 component has `maxSize`): the game
/// keeps the component in silence, an information of ours.
#[test]
fn a_legacy_uv_lock_list_of_any_size_is_kept() {
    let uv: Vec<String> = (0..65).map(|i| format!("\"b{i}\"")).collect();
    let text = document(
        "1.21.90",
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:geometry": {{"identifier": "geometry.x", "uv_lock": [{}]}}, "minecraft:material_instances": {{"*": {{"texture": "x"}}}}}}}}"#,
            uv.join(", ")
        ),
    );
    let loaded = loads(&text, &ucf());
    assert!(loaded.components.get::<Geometry>().is_some(), "kept");
    assert!(
        loaded
            .diagnostics()
            .iter()
            .all(|f| f.severity < Severity::Warning),
        "{:#?}",
        loaded.diagnostics()
    );
}

/// The game's legacy tick component logs
/// "minecraft:tick: maximum ticks in interval range must be more than or equal to minimum
/// ticks" at level 3 and stores both ends as
/// read: a legacy tick whose range is out of order is a kept error and the
/// component stays. The `%s` is the description's name ("minecraft:tick").
#[test]
fn a_legacy_tick_range_out_of_order_is_a_kept_error() {
    let text = document(
        "1.21.90",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:tick": {"interval_range": [20, 10]}}}"#,
    );
    let loaded = loads(&text, &ucf());
    assert!(
        loaded.diagnostics().iter().any(|f| invalid_value(&f.code)
            && f.severity == Severity::Error
            && f.kept
            && f.location
                .json_path
                .to_string()
                .ends_with("minecraft:tick/interval_range")),
        "{:#?}",
        loaded.diagnostics()
    );
    assert!(
        loaded
            .components
            .get::<bedrock_addon::families::block::component::Tick>()
            .is_some(),
        "kept"
    );
}

/// The game's legacy selection box component logs
/// "minecraft:selection_box: origin can't be below (-8, 0, -8) and can't be more than
/// (8, 16, 8)" at level 3 and "... origin with added
/// size ...", each on its own test, and stores the box as read:
/// two kept errors.
#[test]
fn a_legacy_selection_box_out_of_its_bounds_is_a_kept_error() {
    let cases = [
        (
            r#"{"origin": [-9, 0, -8], "size": [1, 1, 1]}"#,
            "minecraft:selection_box: origin can't be below (-8, 0, -8) and can't be more than (8, 16, 8)",
            1,
        ),
        (
            r#"{"origin": [0, 0, 0], "size": [9, 1, 1]}"#,
            "minecraft:selection_box: origin with added size can't be below (-8, 0, -8) and can't be more than (8, 16, 8)",
            1,
        ),
        (r#"{"origin": [-8, 0, -8], "size": [16, 16, 16]}"#, "", 0),
    ];
    for (selection, text, count) in cases {
        let doc = document(
            "1.21.90",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:selection_box": {selection}}}}}"#
            ),
        );
        let loaded = loads(&doc, &ucf());
        let kept: Vec<_> = loaded
            .diagnostics()
            .iter()
            .filter(|f| f.severity == Severity::Error && f.kept)
            .collect();
        assert_eq!(
            kept.len(),
            count,
            "{selection}: {:#?}",
            loaded.diagnostics()
        );
        if count == 1 {
            assert!(
                kept[0]
                    .location
                    .json_path
                    .to_string()
                    .ends_with("minecraft:selection_box"),
                "{selection}: {:#?}",
                kept[0]
            );
            assert!(
                kept[0].code.to_string().contains(text),
                "{selection}: {:#?}",
                kept[0].code
            );
        }
        assert!(
            loaded
                .components
                .get::<bedrock_addon::families::block::component::SelectionBox>()
                .is_some()
        );
    }
}

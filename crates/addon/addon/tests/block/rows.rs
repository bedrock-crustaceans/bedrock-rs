//! The load rows: `f00`, `f01`, `v01`–`v13`, `m01`–`m06`.
//!
//! Findings are matched on their code, severity and path, and where the row (or the rule it
//! names) gives vanilla's text, on that text exactly.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::families::block::component::{
    BlockEntity, ChestObstruction, DestructibleByMining, DestructionParticles, Flammable, Friction,
    Geometry, InstrumentSound, LightEmission, Loot, MapColor, Movable, PrecipitationBehavior,
    PrecipitationInteractions, RandomOffset, RedstoneConductivity, Support,
};
use bedrock_addon::families::block::{Block, StateValues};
use bedrock_addon::json::{StrictFloat, StrictInt};
use bedrock_addon::molang::{Molang, MolangVersion, RawVersion};
use bedrock_addon::version::FormatVersion;

use super::common::{
    allow_listed, context, document, errors, fixture, load, loads, pack_version, rejected, ucf,
};

/// A 1.26.20 document of `payload`.
fn modern(payload: &str) -> String {
    document("1.26.20", payload)
}

/// A 1.26.20 block `demo:x` with `components`.
fn modern_components(components: &str) -> String {
    modern(&format!(
        r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
    ))
}

/// The findings at warning level or above.
fn warnings(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.at_least(Severity::Warning).collect()
}

/// The finding with `code`, `severity` and vanilla's `text` exactly, if any.
fn find<'d>(
    d: &'d Diagnostics,
    code: fn(&DiagnosticCode) -> bool,
    severity: Severity,
    text: &str,
) -> Option<&'d Diagnostic> {
    d.iter()
        .find(|f| code(&f.code) && f.severity == severity && f.code.to_string() == text)
}

/// The findings in one line each: severity, path, code and text.
fn brief(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .map(|f| {
            format!(
                "{:?} {} {:?} {}",
                f.severity, f.location.json_path, f.code, f.code
            )
        })
        .collect()
}

/// Requires a finding with `code`, `severity` and vanilla's `text`.
#[track_caller]
fn expect(d: &Diagnostics, code: fn(&DiagnosticCode) -> bool, severity: Severity, text: &str) {
    assert!(
        find(d, code, severity, text).is_some(),
        "no {severity:?} finding with the text {text:?}: {:#?}",
        brief(d)
    );
}

/// Requires a finding with `code` and `severity` whose path ends with `path`.
#[track_caller]
fn expect_at(d: &Diagnostics, code: fn(&DiagnosticCode) -> bool, severity: Severity, path: &str) {
    assert!(
        d.iter().any(|f| code(&f.code)
            && f.severity == severity
            && f.location.json_path.to_string().contains(path)),
        "no {severity:?} finding at …{path}: {:#?}",
        brief(d)
    );
}

/// The findings of a document, loaded or rejected.
fn findings(text: &str, cx: &bedrock_addon::load::LoadContext) -> (Option<Block>, Diagnostics) {
    match load(text, cx) {
        Ok(document) => {
            let d = document.diagnostics().clone();
            (Some(document.into_value()), d)
        }
        Err(rejected) => (None, rejected.diagnostics),
    }
}

/// A test of a finding's code.
type CodeTest = fn(&DiagnosticCode) -> bool;

fn invalid_value(c: &DiagnosticCode) -> bool {
    matches!(c, DiagnosticCode::InvalidValue { .. })
}

fn missing_toggles(c: &DiagnosticCode) -> bool {
    matches!(c, DiagnosticCode::MissingToggles { .. })
}

#[test]
fn below_floor() {
    let payload = r##"{"description": {"identifier": "demo:x", "is_experimental": true},
        "components": {"minecraft:destroy_time": {"value": 3}, "minecraft:loot": {"table": "loot_tables/x.json"},
                       "minecraft:map_color": {"color": "#ffffff"}}}"##;
    for version in ["1.12.0", "1.15.0"] {
        let d = rejected(&document(version, payload), &context());
        expect(
            &d,
            |c| matches!(c, DiagnosticCode::UnsupportedVersion { .. }),
            Severity::Error,
            "To use block 'demo:x', use json format version 1.16.0 or higher",
        );
    }
}

#[test]
fn envelope() {
    let payload =
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.4}}"#;
    // (1) No format_version: our warning and a 1.16.0 parse.
    let block = loads(&format!(r#"{{"minecraft:block": {payload}}}"#), &context());
    assert_eq!(block.version().parsed_as, FormatVersion::new(1, 16, 0));
    expect_at(
        block.diagnostics(),
        |c| matches!(c, DiagnosticCode::MissingFormatVersion { .. }),
        Severity::Info,
        "",
    );
    // (2) A number is read as its text on the document input:
    // `1.26` is 1.26.0, the legacy struct's.
    let numeric = loads(
        &format!(r#"{{"format_version": 1.26, "minecraft:block": {payload}}}"#),
        &context(),
    );
    assert_eq!(numeric.version().parsed_as, FormatVersion::new(1, 26, 0));
    // (3) Unreadable versions.
    for version in [r#""one.two.three""#, "true", "[1, 26]", "[1, 26, -1]"] {
        let text = format!(r#"{{"format_version": {version}, "minecraft:block": {payload}}}"#);
        let d = rejected(&text, &context());
        expect(
            &d,
            |c| matches!(c, DiagnosticCode::MalformedFormatVersion { .. }),
            Severity::Error,
            "Malformed format version",
        );
    }
    // (3) Three integers read as the version string.
    let array = loads(
        &format!(r#"{{"format_version": [1, 26, 30], "minecraft:block": {payload}}}"#),
        &context(),
    );
    let string = loads(&document("1.26.30", payload), &context());
    assert_eq!(array.value(), string.value());
    // (4) A third root member at 1.26.30.
    let third = |version: &str| {
        format!(r#"{{"format_version": "{version}", "minecraft:block": {payload}, "extra": {{}}}}"#)
    };
    let d = rejected(&third("1.26.30"), &context());
    expect(
        &d,
        |c| matches!(c, DiagnosticCode::MalformedInput { .. }),
        Severity::Error,
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
    );
    // (5) The same member at 1.21.60: an unknown member, the block loads.
    let block = loads(&third("1.21.60"), &context());
    expect_at(
        block.diagnostics(),
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Info,
        "extra",
    );
    // (6) Above the engine version (the game's version text).
    let d = rejected(&document("1.26.61", payload), &context());
    expect(
        &d,
        |c| matches!(c, DiagnosticCode::UnsupportedVersion { .. }),
        Severity::Error,
        "unsupported version: no parser available for version 1.26.61",
    );
}

#[test]
fn redstone_producer_rule() {
    let d = rejected(
        &modern(
            r#"{"description": {"identifier": "demo:x"}, "components": {},
            "permutations": [{"condition": "1", "components": {"minecraft:redstone_producer": {"power": 15}}}]}"#,
        ),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "For 'format_version' >= 1.26.20 the block cannot define 'minecraft:redstone_producer' in the 'permutations' array and not define it in the base 'components' object",
    );
}

/// A 1.16.100 block using every legacy shape the chain converts.
const FULL_CHAIN: &str = r##"{
  "format_version": "1.16.100",
  "minecraft:block": {
    "description": {
      "identifier": "demo:everything",
      "properties": { "demo:open": [false, true] }
    },
    "components": {
      "minecraft:entity_collision": { "origin": [-8, 0, -8], "size": [16, 8, 16] },
      "minecraft:pick_collision": { "origin": [-8, 0, -8], "size": [16, 8, 16], "enabled": true },
      "minecraft:block_light_absorption": 3,
      "minecraft:block_light_emission": 0.5,
      "minecraft:destroy_time": 2,
      "minecraft:explosion_resistance": { "resistance": 6 },
      "minecraft:friction": 0.6,
      "minecraft:flammable": { "flame_odds": 5, "burn_odds": 20 },
      "minecraft:display_name": "everything",
      "minecraft:geometry": "geometry.everything",
      "minecraft:material_instances": { "*": { "texture": "everything", "render_method": "alpha_test" } },
      "minecraft:loot": { "loot_table": "loot_tables/everything.json" },
      "minecraft:map_color": "#123456",
      "minecraft:creative_category": { "category": "construction", "group": "itemGroup.name.stone" },
      "minecraft:rotation": [0, 90, 0],
      "minecraft:ticking": { "on_tick": { "event": "demo:tick" }, "range": [1, 2], "looping": true },
      "tag:stone": {}
    },
    "permutations": [
      {
        "condition": "q.block_property('demo:open') == 1",
        "components": { "minecraft:block_light_emission": 1.0, "minecraft:friction": 0.1 }
      }
    ],
    "events": {
      "demo:tick": { "set_block_property": { "demo:open": "!q.block_property('demo:open')" } }
    }
  }
}"##;

#[test]
fn full_chain_1_16_100() {
    let block = loads(FULL_CHAIN, &allow_listed());
    assert!(block.version().upgraded);
    let written = bedrock_addon::document::Content::to_json(block.value()).unwrap();
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(
            super::common::fixture_dir("v02_full_chain_1_16_100.snapshot.json"),
            &written,
        )
        .unwrap();
    }
    let expected = fixture("v02_full_chain_1_16_100.snapshot.json");
    assert_eq!(
        written.trim(),
        expected.trim(),
        "the upgraded block differs from the snapshot"
    );
}

#[test]
fn full_chain_1_16_0() {
    let block = loads(
        &fixture("upgrade/s00_1_16_0_description_flags.before.json"),
        &context(),
    );
    let c = &block.components;
    assert!(
        matches!(c.get::<DestructibleByMining>(), Some(DestructibleByMining::Params(p)) if p.seconds_to_destroy == StrictFloat(4.0))
    );
    assert!((c.get::<Friction>().unwrap().0.0 - 0.4).abs() < 1e-6);
    assert_eq!(
        c.get::<LightEmission>(),
        Some(&LightEmission(StrictInt(15)))
    );
    assert_eq!(
        c.get::<Loot>().map(|l| l.0.0.as_str()),
        Some("loot_tables/chests/simple_dungeon.json")
    );
    assert!(matches!(c.get::<MapColor>(), Some(MapColor::Hex(h)) if h == "#00ff00"));
    assert!(
        matches!(c.get::<Flammable>(), Some(Flammable::Params(f)) if f.catch_chance_modifier() == 50 && f.destroy_chance_modifier() == 0)
    );
    assert!(
        c.get::<RedstoneConductivity>()
            .is_some_and(|r| r.redstone_conductor())
    );
    assert!(c.get::<ChestObstruction>().is_some());
    let synthesized = c
        .iter()
        .filter(|e| {
            matches!(
                e,
                bedrock_addon::document::ComponentRef::Typed {
                    synthesized: true,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        synthesized, 2,
        "redstone_conductivity and chest_obstruction are synthesised"
    );
}

#[test]
fn release_gates() {
    let base = |version: &str, components: &str| {
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        )
    };
    // 1.19.30 base geometry without the legacy creator features: not loaded.
    let d = rejected(
        &base("1.19.30", r#"{"minecraft:geometry": "geometry.x"}"#),
        &context(),
    );
    expect(
        &d,
        missing_toggles,
        Severity::Error,
        "To use component 'minecraft:geometry', use json format version 1.19.40 or higher",
    );
    // The same in a permutation: the toggle line is level 3 and the failing permutation fails
    // the component load, the block is refused whole.
    let text = document(
        "1.19.30",
        r#"{"description": {"identifier": "demo:x"}, "components": {},
        "permutations": [{"condition": "q.block_state('demo:a') == 1", "components": {"minecraft:geometry": "geometry.x"}}]}"#,
    );
    let d = rejected(&text, &context());
    expect(
        &d,
        missing_toggles,
        Severity::Error,
        "To use component 'minecraft:geometry', use json format version 1.19.40 or higher",
    );
    // 1.21.20 redstone_conductivity without Upcoming Creator Features.
    let d = rejected(
        &base(
            "1.21.20",
            r#"{"minecraft:redstone_conductivity": {"redstone_conductor": true}}"#,
        ),
        &context(),
    );
    expect(
        &d,
        missing_toggles,
        Severity::Error,
        "To use component 'minecraft:redstone_conductivity' in json formats earlier than 1.21.30, turn on the 'upcoming_creator_features' toggle in settings",
    );
    // destruction_particles below 1.21.70.
    let d = rejected(
        &base(
            "1.21.60",
            r#"{"minecraft:destruction_particles": {"texture": "x"}}"#,
        ),
        &context(),
    );
    // The game's post-parse check logs it at level 3 and fails the component: the block
    // is refused.
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "Format version 1.21.70 is at least required to use minecraft:destruction_particles",
    );
}

#[test]
fn field_gates() {
    let text_of = |version: &str, components: &str| {
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        )
    };
    let with = |version: &str, components: &str, cx: &bedrock_addon::load::LoadContext| {
        loads(&text_of(version, components), cx)
    };
    // Each gate is a legacy description's post-parse check at level 3 that fails the
    // component's load, which refuses the block: `uv_lock`,
    // `isotropic`, `alpha_masked_tint`, `particle_count`.
    let d = rejected(
        &text_of(
            "1.21.80",
            r#"{"minecraft:geometry": {"identifier": "geometry.x", "uv_lock": true}, "minecraft:material_instances": {"*": {"texture": "x"}}}"#,
        ),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "Cannot use field \"uv_lock\" before 'format_version' 1.21.90 when 'minecraft:transformation' component was released.",
    );
    let d = rejected(
        &text_of(
            "1.21.60",
            r#"{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "isotropic": true}}}"#,
        ),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "'minecraft:material_instances' cannot use 'isotropic' member with 'format_version' < 1.21.70",
    );
    for (version, instance_json, text) in [
        (
            "1.21.110",
            r#"{"texture": "x", "tint_method": "grass", "alpha_masked_tint": true}"#,
            "'alpha_masked_tint' is only supported at format_version 1.21.120 or higher",
        ),
        (
            "1.21.120",
            r#"{"texture": "x", "alpha_masked_tint": true}"#,
            "'alpha_masked_tint' requires a non-default 'tint_method'",
        ),
        (
            "1.21.120",
            r#"{"texture": "x", "tint_method": "grass", "render_method": "blend", "alpha_masked_tint": true}"#,
            "'alpha_masked_tint' requires 'render_method' to be 'opaque'",
        ),
        (
            "1.26.20",
            r#"{"texture": "x", "alpha_masked_tint": true}"#,
            "\"alpha_masked_tint\" requires a non-default \"tint_method\".",
        ),
        (
            "1.26.20",
            r#"{"texture": "x", "tint_method": "grass", "render_method": "blend", "alpha_masked_tint": true}"#,
            "\"alpha_masked_tint\" requires \"render_method\" to be \"opaque\".",
        ),
    ] {
        let text_of = |version: &str| {
            document(
                version,
                &format!(
                    r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {{"*": {instance_json}}}}}}}"#
                ),
            )
        };
        // In a cereal document the instance's Cereal constraint fails the block; in
        // a legacy one the post-parse check does.
        let d = rejected(&text_of(version), &context());
        expect(&d, invalid_value, Severity::Error, text);
    }
    // The destruction particles post-parse check never reads the experiments:
    // the gate is the version alone. A 1.21.90 document keeps `particle_count` without the toggle.
    for cx in [context(), ucf()] {
        let block = with(
            "1.21.90",
            r#"{"minecraft:destruction_particles": {"particle_count": 20}}"#,
            &cx,
        );
        assert_eq!(
            block
                .components
                .get::<DestructionParticles>()
                .and_then(|p| p.particle_count),
            Some(StrictInt(20))
        );
        assert!(
            !block.diagnostics().iter().any(|f| invalid_value(&f.code)),
            "{:#?}",
            block.diagnostics()
        );
    }
    let d = rejected(
        &text_of(
            "1.21.80",
            r#"{"minecraft:destruction_particles": {"particle_count": 20}}"#,
        ),
        &ucf(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "Format version 1.21.90 is at least required to use particle_count and experimental creator features",
    );
}

/// The multi-block trait binds `parts` with a numeric constraint of min 2
/// / max 4: a 0x40 under both masks refuses the block.
#[test]
fn a_multi_block_with_parts_outside_2_to_4_is_refused() {
    for parts in [1, 5] {
        let text = modern(&format!(
            r#"{{"description": {{"identifier": "demo:door", "traits": {{"minecraft:multi_block": {{"enabled_states": ["minecraft:multi_block_part"], "direction": "up", "parts": {parts}}}}}}},
               "components": {{"minecraft:movable": {{"movement_type": "popped"}}}}}}"#
        ));
        let d = rejected(&text, &ucf());
        assert!(
            d.iter().any(|f| invalid_value(&f.code)
                && f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .ends_with("minecraft:multi_block/parts")),
            "{parts}: {d:#?}"
        );
    }
}

#[test]
fn cross_component() {
    let base = |version: &str, components: &str| {
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        )
    };
    let cases = [
        (
            "1.26.20",
            r#"{"minecraft:geometry": "geometry.x"}"#,
            "Block needs both a \"geometry\" and \"material_instances\" component.",
        ),
        (
            "1.21.80",
            r#"{"minecraft:material_instances": {"*": {"texture": "x"}}}"#,
            "Block needs both a geometry and material instances component.",
        ),
        (
            "1.26.20",
            r#"{"minecraft:flower_pottable": {}}"#,
            "Block that uses \"flower_pottable\" needs both a \"geometry\" and \"material_instances\" component.",
        ),
        (
            "1.21.60",
            r#"{"minecraft:item_visual": {"geometry": "geometry.x", "material_instances": {"*": {"texture": "x"}}}}"#,
            "Block demo:x needs both a geometry and material instances component when using an Item Visual Component.",
        ),
        (
            "1.26.20",
            r#"{"minecraft:redstone_producer": {"power": 3}, "minecraft:redstone_consumer": {}}"#,
            "Block cannot have both \"minecraft:redstone_producer\" and \"minecraft:redstone_consumer\" components.",
        ),
        (
            "1.21.120",
            r#"{"minecraft:redstone_producer": {"power": 3}, "minecraft:redstone_consumer": {}}"#,
            "Block cannot have both 'minecraft:redstone_producer' and 'minecraft:redstone_consumer' components",
        ),
        (
            "1.26.20",
            r#"{"minecraft:collision_box": true, "minecraft:precipitation_interactions": {"precipitation_behavior": "snowlogging"}}"#,
            "Blocks with precipitation interactions \"snowlogging\" must have collision box disabled",
        ),
        (
            "1.26.20",
            r#"{"minecraft:geometry": "minecraft:geometry.full_block", "minecraft:material_instances": {"*": {"texture": "x"}}, "minecraft:random_offset": {"x": {"range": {"min": 0, "max": 0}}}}"#,
            "Block with \"geometry\" set at \"minecraft:geometry.full_block\" cannot have a \"random_offset\"",
        ),
    ];
    for (version, components, text) in cases {
        let (_, d) = findings(&base(version, components), &ucf());
        expect(&d, invalid_value, Severity::Error, text);
    }
    // Rule 13 below 1.26.20: clamped with the legacy text (`%f`), the component kept.
    let block = loads(
        &base(
            "1.21.110",
            r#"{"minecraft:random_offset": {"x": {"range": {"min": -2, "max": 2}}}}"#,
        ),
        &context(),
    );
    let offset = block.components.get::<RandomOffset>().expect("kept");
    let x = offset.x.as_ref().unwrap();
    assert_eq!((x.range.min.0, x.range.max.0), (0.0, 0.0));
    expect(
        block.diagnostics(),
        |c| matches!(c, DiagnosticCode::ClampedValue { .. }),
        Severity::Warning,
        "Random offset pushes collision box outside of cube for 'x->min'. Restricting value from -2.000000 to 0.000000",
    );
    // Rule 14: a multi-block permutation may not define `movable`, `replaceable`,
    // `random_offset` or `placement_filter`.
    for (component, json) in [
        ("minecraft:movable", r#"{"movement_type": "immovable"}"#),
        ("minecraft:replaceable", "{}"),
        (
            "minecraft:random_offset",
            r#"{"y": {"range": {"min": 0, "max": 0}}}"#,
        ),
        ("minecraft:placement_filter", r#"{"conditions": []}"#),
    ] {
        let text = modern(&format!(
            r#"{{"description": {{"identifier": "demo:x", "traits": {{"minecraft:multi_block": {{"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}}}}},
            "components": {{"minecraft:movable": {{"movement_type": "popped"}}}},
            "permutations": [{{"condition": "1", "components": {{"{component}": {json}}}}}]}}"#
        ));
        let d = rejected(&text, &ucf());
        let message = format!(
            "Component '{component}' is not valid in multi block permutations, only in the base BlockType components"
        );
        assert!(
            d.iter().any(|f| invalid_value(&f.code)
                && f.severity == Severity::Error
                && f.code.to_string() == message),
            "{component}: {d:#?}"
        );
    }
}

#[test]
fn format_1_26_30_stamp() {
    for name in ["v06_shelf_mushroom.json", "v06_red_shrub.json"] {
        let text = fixture(name);
        let block = loads(&text, &ucf());
        assert_eq!(
            block.version().parsed_as,
            FormatVersion::new(1, 26, 20),
            "{name}"
        );
        // Every document below 1.26.60 runs the 1.26.20 → 1.26.60 step.
        assert!(block.version().upgraded);
        assert_eq!(block.source_format_version, FormatVersion::new(1, 26, 30));
        assert!(
            warnings(block.diagnostics()).is_empty(),
            "{name}: {:#?}",
            brief(block.diagnostics())
        );
        // Written back: the input with `format_version` unchanged.
        let written: bedrock_addon::json::Value = facet_json::from_str(
            &bedrock_addon::document::Content::to_json(block.value()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            written.as_object().and_then(|o| o.get("format_version")),
            Some(&bedrock_addon::json::Value::from("1.26.30"))
        );
        let reread = loads(&bedrock_addon::json::to_json(&written).unwrap(), &ucf());
        assert_eq!(reread.value(), block.value(), "{name}");
        // Stamped 1.26.20: the same value except the source version.
        let older = loads(&text.replacen("\"1.26.30\"", "\"1.26.20\"", 1), &ucf());
        let mut same = older.into_value();
        same.source_format_version = FormatVersion::new(1, 26, 30);
        assert_eq!(&same, block.value(), "{name}");
        // Stamped 1.26.61: above the engine.
        let d = rejected(&text.replacen("\"1.26.30\"", "\"1.26.61\"", 1), &ucf());
        expect(
            &d,
            |c| matches!(c, DiagnosticCode::UnsupportedVersion { .. }),
            Severity::Error,
            "unsupported version: no parser available for version 1.26.61",
        );
    }
}

#[test]
fn engine_gated_1_26_30() {
    let block = loads(
        &modern_components(
            r#"{"minecraft:flammable": {"catch_chance_modifier": 5, "destroy_chance_modifier": 20, "lava_flammable": "always"}}"#,
        ),
        // A pack whose engine knows the member (an older one gets `FieldSinceEngine`).
        &pack_version(1, 26, 50),
    );
    assert!(block.diagnostics().is_empty(), "{:#?}", block.diagnostics());
    // The validator of 1.26.60 does not gate `block_entity` as a whole.
    let entity = modern_components(r#"{"minecraft:block_entity": {"dynamic_properties": true}}"#);
    let block = loads(&entity, &context());
    assert!(
        block
            .components
            .get::<BlockEntity>()
            .is_some_and(|b| b.dynamic_properties())
    );
    let in_permutation = modern(
        r#"{"description": {"identifier": "demo:x"}, "components": {},
        "permutations": [{"condition": "1", "components": {"minecraft:block_entity": {"dynamic_properties": true}}}]}"#,
    );
    // A rejected key fails the block.
    let d = rejected(&in_permutation, &ucf());
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "key minecraft:block_entity is not allowed",
    );
    // `instrument_sound` in a 1.26.20 document needs no toggle (the two toggle texts of the older
    // gate are no strings of 1.26.60).
    let instrument = modern_components(
        r#"{"minecraft:instrument_sound": {"up": "note.bit", "down": "note.bell"}}"#,
    );
    let block = loads(&instrument, &context());
    assert_eq!(
        block.components.get::<InstrumentSound>().map(|i| i.0.len()),
        Some(2)
    );
}

#[test]
fn snowlogging_released() {
    for spelling in ["snowlogging", "snow_log_no_collision"] {
        let text = document(
            "1.21.120",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:precipitation_interactions": {{"precipitation_behavior": "{spelling}"}}}}}}"#
            ),
        );
        let block = loads(&text, &context());
        assert!(
            block.diagnostics().is_empty(),
            "{spelling}: {:#?}",
            block.diagnostics()
        );
        assert_eq!(
            block
                .components
                .get::<PrecipitationInteractions>()
                .map(|p| p.precipitation_behavior),
            Some(PrecipitationBehavior::Snowlogging),
            "{spelling}"
        );
    }
}

/// `snow_log_no_collision` is read as `snowlogging`.
#[test]
fn snow_log_no_collision_is_normalised_deviation() {
    let text = modern_components(
        r#"{"minecraft:precipitation_interactions": {"precipitation_behavior": "snow_log_no_collision"}}"#,
    );
    let block = loads(&text, &context());
    let written = bedrock_addon::document::Content::to_json(block.value()).unwrap();
    assert!(
        written.contains("\"snowlogging\"") && !written.contains("snow_log_no_collision"),
        "{written}"
    );
}

#[test]
fn retail_multi_block_rules() {
    let multi = |traits: &str, components: &str, permutations: &str| {
        modern(&format!(
            r#"{{"description": {{"identifier": "demo:door", "traits": {{"minecraft:multi_block": {{"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}{traits}}}}},
               "components": {components}{permutations}}}"#
        ))
    };
    let popped = r#"{"minecraft:movable": {"movement_type": "popped"}}"#;
    let geometry = r#""minecraft:geometry": {"identifier": "geometry.x", "n_way_visual_rotation": {"y": "minecraft:cardinal_direction"}}, "minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let cases: [(String, CodeTest, &str); 5] = [
        (
            multi("", "{}", ""),
            |c| matches!(c, DiagnosticCode::MissingRequiredField { .. }),
            "Blocks with trait 'minecraft:multi_block' must define component 'minecraft:movable'",
        ),
        (
            multi(
                "",
                r#"{"minecraft:movable": {"movement_type": "push"}}"#,
                "",
            ),
            invalid_value,
            "Blocks with trait 'minecraft:multi_block' for component 'minecraft:movable' the 'movement_type' must either be 'popped' or 'immovable'",
        ),
        (
            multi(
                r#", "minecraft:connection": {"enabled_states": ["minecraft:cardinal_connections"]}"#,
                popped,
                "",
            ),
            invalid_value,
            "A block with trait 'minecraft:multi_block' cannot also define trait 'minecraft:connection'",
        ),
        (
            multi(
                r#", "minecraft:placement_direction": {"enabled_states": ["minecraft:facing_direction"]}"#,
                popped,
                "",
            ),
            invalid_value,
            "A block with trait 'minecraft:multi_block' and trait 'minecraft:placement_direction' can only enable state 'minecraft:cardinal_direction'",
        ),
        (
            multi(
                r#", "minecraft:placement_direction": {"enabled_states": ["minecraft:cardinal_direction"]}"#,
                &format!(r#"{{"minecraft:movable": {{"movement_type": "popped"}}, {geometry}}}"#),
                "",
            ),
            invalid_value,
            "'minecraft:geometry' field 'n_way_visual_rotation' cannot be defined for a block with trait 'minecraft:multi_block'",
        ),
    ];
    for (text, code, message) in cases {
        let d = rejected(&text, &ucf());
        expect(&d, code, Severity::Error, message);
    }
    // `n_way_visual_rotation` in a permutation, naming an unknown state, or with no axis.
    let plain = |components: &str, permutations: &str| {
        modern(&format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}{permutations}}}"#
        ))
    };
    let d = rejected(
        &plain(
            r#"{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x"}}}"#,
            r#", "permutations": [{"condition": "1", "components": {"minecraft:geometry": {"identifier": "geometry.x", "n_way_visual_rotation": {"x": "demo:s"}}}}]"#,
        ),
        &ucf(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "minecraft:geometry 'n_way_visual_rotation' may only be defined on the base block and is not allowed in permutations.",
    );
    let d = rejected(
        &plain(
            &format!("{{{geometry}}}").replace("minecraft:cardinal_direction", "demo:missing"),
            "",
        ),
        &ucf(),
    );
    expect(
        &d,
        |c| matches!(c, DiagnosticCode::UnresolvedReference { .. }),
        Severity::Error,
        "'minecraft:geometry' field 'n_way_visual_rotation.y' references unknown state 'demo:missing'",
    );
    let d = rejected(
        &plain(
            r#"{"minecraft:geometry": {"identifier": "geometry.x", "n_way_visual_rotation": {}}, "minecraft:material_instances": {"*": {"texture": "x"}}}"#,
            "",
        ),
        &ucf(),
    );
    expect(
        &d,
        |c| matches!(c, DiagnosticCode::MissingRequiredField { .. }),
        Severity::Error,
        "'n_way_visual_rotation' must define at least one axis (x, y, or z)",
    );
    // Without Upcoming Creator Features: a horizontal direction.
    let text = modern(
        r#"{"description": {"identifier": "demo:door", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "north"}}},
        "components": {"minecraft:movable": {"movement_type": "popped"}}}"#,
    );
    let d = rejected(&text, &context());
    expect(
        &d,
        missing_toggles,
        Severity::Error,
        "Upcoming Creator Features experiment is required for trait 'minecraft:multi_block' for horizontal directions",
    );
}

#[test]
fn y_rotation_offset_snap() {
    let direction = |states: &str| {
        modern(&format!(
            r#"{{"description": {{"identifier": "demo:x", "traits": {{"minecraft:placement_direction": {{"enabled_states": [{states}], "y_rotation_offset": 100}}}}}}, "components": {{}}}}"#
        ))
    };
    let block = loads(&direction(r#""minecraft:cardinal_direction""#), &context());
    let traits = &block.description.traits;
    assert_eq!(
        traits
            .placement_direction
            .as_ref()
            .and_then(|p| p.y_rotation_offset),
        Some(StrictFloat(90.0))
    );
    expect(
        block.diagnostics(),
        |c| matches!(c, DiagnosticCode::ClampedValue { .. }),
        Severity::Warning,
        "\"minecraft:placement_direction\" trait \"y_rotation_offset\" must be a factor of 90.0. Defaulting to the nearest axis-aligned angle.",
    );
    let block = loads(
        &direction(r#""minecraft:sixteen_way_rotation""#),
        &context(),
    );
    assert!(!block.diagnostics().iter().any(|f| invalid_value(&f.code)));
    let d = rejected(
        &direction(r#""minecraft:corner_and_cardinal_direction""#),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "\"y_rotation_offset\" can only be used when \"minecraft:cardinal_direction\", \"minecraft:facing_direction\", and/or \"minecraft:sixteen_way_rotation\" are enabled in \"enabled_states\".",
    );
}

#[test]
fn cereal_rejected_keys() {
    // Each an error that fails the block (`loader-9`).
    let d = rejected(
        &modern(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:unit_cube": {}},
        "permutations": [{"condition": "1", "components": {"minecraft:support": {"shape": "fence"}, "minecraft:block_entity": {}}}]}"#,
        ),
        &ucf(),
    );
    for key in [
        "minecraft:unit_cube",
        "minecraft:support",
        "minecraft:block_entity",
    ] {
        expect(
            &d,
            invalid_value,
            Severity::Error,
            &format!("key {key} is not allowed"),
        );
    }
    let legacy = document(
        "1.20.10",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:unit_cube": {}, "minecraft:material_instances": {"*": {"texture": "x"}}}}"#,
    );
    let block = loads(&legacy, &allow_listed());
    assert!(
        !block
            .diagnostics()
            .iter()
            .any(|f| f.code.to_string().ends_with("is not allowed")),
        "{:#?}",
        block.diagnostics()
    );
    assert_eq!(
        block.deprecated.unit_cube,
        Some(bedrock_addon::json::StrictBool(true))
    );
}

#[test]
fn required_members() {
    for (component, path) in [
        (r#""minecraft:movable": {}"#, "minecraft:movable"),
        (r#""minecraft:support": {}"#, "minecraft:support"),
        (r#""minecraft:tick": {"looping": true}"#, "minecraft:tick"),
        (
            r#""minecraft:precipitation_interactions": {}"#,
            "minecraft:precipitation_interactions",
        ),
        (
            r#""minecraft:random_offset": {"x": {"steps": 2}}"#,
            "minecraft:random_offset/x",
        ),
    ] {
        // A required member missing (0x80) rejects a cereal block.
        let d = rejected(&modern_components(&format!("{{{component}}}")), &ucf());
        expect_at(
            &d,
            |c| matches!(c, DiagnosticCode::MissingRequiredField { .. }),
            Severity::Error,
            path,
        );
    }
    // `map_color` is a colour or an object; the object without `color` fits no form.
    let d = rejected(
        &modern_components(r#"{"minecraft:map_color": {"tint_method": "grass"}}"#),
        &ucf(),
    );
    expect_at(
        &d,
        |c| {
            matches!(
                c,
                DiagnosticCode::MissingRequiredField { .. } | DiagnosticCode::InvalidType { .. }
            )
        },
        Severity::Error,
        "minecraft:map_color",
    );
    let block = loads(
        &modern(
            r#"{"description": {"identifier": "demo:door", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "down"}}},
        "components": {"minecraft:movable": {"movement_type": "popped"}}}"#,
        ),
        &ucf(),
    );
    assert_eq!(
        block
            .description
            .traits
            .multi_block
            .as_ref()
            .map(|m| m.parts()),
        Some(2)
    );
    assert_eq!(
        block.states_with_traits().get("minecraft:multi_block_part"),
        Some(&StateValues::range(0, 1))
    );
    let d = rejected(
        &modern(
            r#"{"description": {"identifier": "demo:x", "states": {"demo:b": [true, true]}}, "components": {}}"#,
        ),
        &context(),
    );
    expect_at(
        &d,
        invalid_value,
        Severity::Error,
        "description/states/demo:b",
    );
    let _: Option<&Support> = None;
    let _: Option<&Movable> = None;
}

#[test]
fn constraint_texts() {
    let base = |version: &str, components: &str| {
        document(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#
            ),
        )
    };
    // Each a Cereal constraint of a 1.26.20 document: an error that fails the block.
    let d = rejected(
        &base(
            "1.26.20",
            r#"{"minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 17, 16]}}"#,
        ),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "\"origin\" + \"size\", must be inside the following limit: (0, 0, 0) and (16, 16, 16)",
    );
    for (collision, text) in [
        (
            r#"{"origin": [-8, 0, -8], "size": [16, 25, 16]}"#,
            "\"origin\" + \"size\", must be inside the following limit: (0, 0, 0) and (16, 24, 16)",
        ),
        (
            r#"[{"origin": [-8, 0, -8], "size": [16, 8, 16]}, {"origin": [-8, 8, -8], "size": [16, 17, 16]}]"#,
            "The total of each \"origin\" + \"size\" pair, must be inside the following limit: (0, 0, 0) and (16, 24, 16)",
        ),
    ] {
        // The 1.26.20 constraint fails the block.
        let d = rejected(
            &base(
                "1.26.20",
                &format!(r#"{{"minecraft:collision_box": {collision}}}"#),
            ),
            &context(),
        );
        expect(&d, invalid_value, Severity::Error, text);
    }
    let d = rejected(
        &base(
            "1.26.20",
            r#"{"minecraft:tick": {"interval_range": [10, 5]}}"#,
        ),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "Invalid interval_range: The first value must be equal or lower than the second value",
    );
    let d = rejected(
        &base("1.26.20", r#"{"minecraft:geometry": "geometry.x"}"#),
        &context(),
    );
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "Block needs both a \"geometry\" and \"material_instances\" component.",
    );
    let offset = r#"{"minecraft:random_offset": {"x": {"range": {"min": -2, "max": 2}}}}"#;
    let d = rejected(&base("1.26.20", offset), &ucf());
    expect(
        &d,
        invalid_value,
        Severity::Error,
        "\"random_offset pushes \"collision_box\" outside of unit cube for \"x\" \"range\" \"min\" of -2",
    );
    let block = loads(&base("1.21.100", offset), &context());
    assert!(block.components.get::<RandomOffset>().is_some());
    expect(
        block.diagnostics(),
        |c| matches!(c, DiagnosticCode::ClampedValue { .. }),
        Severity::Warning,
        "Random offset pushes collision box outside of cube for 'x->min'. Restricting value from -2.000000 to 0.000000",
    );
}

// ---- Molang -------------------------------------------------------------------------------

#[test]
fn condition_float() {
    let block = loads(
        &modern(
            r#"{"description": {"identifier": "demo:x"}, "components": {},
        "permutations": [{"condition": 1, "components": {}}]}"#,
        ),
        &context(),
    );
    assert_eq!(block.permutations[0].condition, Molang::Const(1.0));
    assert!(block.diagnostics().is_empty(), "{:#?}", block.diagnostics());
    let d = rejected(
        &document(
            "1.21.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {},
        "permutations": [{"condition": 1, "components": {}}]}"#,
        ),
        &context(),
    );
    expect(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidType { .. }),
        Severity::Error,
        "permutation condition must be a string value",
    );
}

/// The raw version a permutation condition was read at.
fn condition_version(block: &Block, index: usize) -> Option<RawVersion> {
    block.permutations[index]
        .condition
        .source()
        .and_then(|s| s.raw_version())
}

/// The compiler's diagnostic kinds behind the `InvalidMolang` / `MolangLint` findings, with
/// their severities.
#[cfg(feature = "molang")]
fn molang_kinds(d: &Diagnostics) -> Vec<(bedrock_addon::molang::DiagCode, Severity)> {
    d.iter()
        .filter_map(|f| match f.detail.as_deref() {
            Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(m)) => {
                Some((m.code(), f.severity))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn condition_string() {
    let text = modern(
        r#"{"description": {"identifier": "demo:x", "states": {"demo:lit": [false, true]}}, "components": {},
        "permutations": [{"condition": "q.block_state('demo:lit')", "components": {}}]}"#,
    );
    let block = loads(&text, &molang(pack_version(1, 16, 0)));
    assert_eq!(
        condition_version(&block, 0),
        Some(RawVersion::from(MolangVersion::V13)),
        "13, not the pack's 0"
    );
    assert!(
        warnings(block.diagnostics()).is_empty(),
        "{:#?}",
        brief(block.diagnostics())
    );
    let legacy = |condition: &str| {
        document(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}},
            "permutations": [{{"condition": "{condition}", "components": {{}}}}]}}"#
            ),
        )
    };
    let cx = molang(pack_version(1, 21, 0));
    #[cfg(feature = "molang")]
    {
        use bedrock_addon::molang::DiagCode;
        for (condition, kinds) in [
            ("math.random(0, 1) > 0.5", &[DiagCode::InvalidOperation][..]),
            ("q.is_baby", &[DiagCode::UnknownQuery, DiagCode::Syntax][..]),
        ] {
            let d = rejected(&legacy(condition), &cx);
            let found = molang_kinds(&d);
            for kind in kinds {
                assert!(
                    found.contains(&(*kind, Severity::Error)),
                    "{condition}: {kind:?} in {found:?}"
                );
            }
            expect(
                &d,
                invalid_value,
                Severity::Error,
                "permutation condition failed to parse",
            );
        }
    }
    let block = loads(&legacy("math.die_roll(1, 1, 2) > 1"), &cx);
    #[cfg(feature = "molang")]
    assert!(block.diagnostics().is_empty(), "{:#?}", block.diagnostics());
    #[cfg(not(feature = "molang"))]
    let _ = block;
}

#[test]
fn condition_object_version() {
    let text = modern(
        r#"{"description": {"identifier": "demo:x", "states": {"demo:lit": [false, true]}}, "components": {},
        "permutations": [{"condition": {"expression": "q.block_state('demo:lit') && 1 || 0", "version": 5}, "components": {}}]}"#,
    );
    let block = loads(&text, &molang(context()));
    assert_eq!(condition_version(&block, 0), Some(RawVersion(5)));
    #[cfg(feature = "molang")]
    assert!(block.diagnostics().is_empty(), "{:#?}", block.diagnostics());
    #[cfg(feature = "nbt")]
    {
        let nbt = block.to_network_nbt();
        let nbtx::Value::Compound(root) = nbt else {
            panic!()
        };
        let version = root
            .iter()
            .find(|(key, _)| key.as_slice() == b"molangVersion")
            .map(|(_, v)| v);
        assert_eq!(version, Some(&nbtx::Value::Int(5)));
    }
}

#[test]
fn condition_object_invalid_and_high() {
    for condition in [
        r#"{"expression": "q.block_state('demo:lit')", "version": -1}"#,
        r#""""#,
    ] {
        let text = modern(&format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}},
            "permutations": [{{"condition": {condition}, "components": {{}}}}]}}"#
        ));
        let block = loads(&text, &molang(context()));
        // The game says nothing of the empty node: an information of ours.
        expect_at(
            block.diagnostics(),
            |c| matches!(c, DiagnosticCode::DroppedField { .. }),
            Severity::Info,
            "permutations/0/condition",
        );
        assert!(
            !block
                .diagnostics()
                .iter()
                .any(|f| matches!(f.code, DiagnosticCode::InvalidMolang { .. })),
            "{condition}: not compiled"
        );
    }
    let text = modern(
        r#"{"description": {"identifier": "demo:x"}, "components": {},
        "permutations": [{"condition": {"expression": "q.block_state('demo:lit')", "version": 14}, "components": {}}]}"#,
    );
    let block = loads(&text, &molang(context()));
    assert_eq!(block.permutations.len(), 1, "kept; it never matches");
    #[cfg(feature = "molang")]
    {
        use bedrock_addon::molang::DiagCode;
        let found = molang_kinds(block.diagnostics());
        // The rejecting kinds are the parser's level-3 lines, after which the
        // game's Molang loader goes on: kept errors, and the block loads.
        for expected in [
            // molangx's own version lint, an information.
            (DiagCode::InvalidVersion, Severity::Info),
            (DiagCode::QueryDeprecated, Severity::Info),
            (DiagCode::UnknownQuery, Severity::Error),
            (DiagCode::Syntax, Severity::Error),
        ] {
            assert!(found.contains(&expected), "{expected:?} in {found:?}");
        }
        assert!(
            block
                .diagnostics()
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .all(|f| f.kept),
            "kept: {:#?}",
            block.diagnostics()
        );
    }
}

#[test]
fn condition_pre_1_19_70() {
    let text = |condition: &str| {
        document(
            "1.19.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}},
            "permutations": [{{"condition": "{condition}", "components": {{}}}}]}}"#
            ),
        )
    };
    let cx = molang(allow_listed());
    let block = loads(&text("q.is_baby"), &cx);
    #[cfg(feature = "molang")]
    {
        assert!(block.permutations.is_empty(), "compiles under B0: dropped");
        // The drop is silent in the game: an information of ours.
        expect_at(
            block.diagnostics(),
            |c| matches!(c, DiagnosticCode::DroppedField { .. }),
            Severity::Info,
            "permutations/0",
        );
        let found = molang_kinds(block.diagnostics());
        assert!(!found.is_empty(), "the findings are reported");
        // The parse logs its lines at level 3 and the component load goes on:
        // kept errors.
        assert!(
            found
                .iter()
                .all(|(_, severity)| *severity == Severity::Error),
            "{found:?}"
        );
        assert!(
            block
                .diagnostics()
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .all(|f| f.kept),
            "{:#?}",
            block.diagnostics()
        );
    }
    #[cfg(not(feature = "molang"))]
    let _ = &block;
    let block = loads(&text("q.block_state("), &cx);
    assert_eq!(block.permutations.len(), 1, "fails both: kept");
    #[cfg(feature = "molang")]
    {
        let found = molang_kinds(block.diagnostics());
        assert!(
            found.contains(&(
                bedrock_addon::molang::DiagCode::StatementForm,
                Severity::Error
            )),
            "{found:?}"
        );
        assert!(block.diagnostics().iter().all(|f| f.kept), "kept errors");
    }
}

#[test]
fn bone_visibility_and_descriptors() {
    let text = modern(
        r#"{"description": {"identifier": "demo:x", "states": {"demo:open": [false, true]}},
        "components": {
          "minecraft:geometry": {"identifier": "geometry.x", "bone_visibility": {"lid": "q.block_state('demo:open')", "lock": true, "hinge": "math.random(0, 1)"}},
          "minecraft:material_instances": {"*": {"texture": "x"}},
          "minecraft:placement_filter": {"conditions": [{"block_filter": [{"tags": "query.any_tag('stone')"}, {"tags": "q.block_state('x')"}]}]}}}"#,
    );
    let block = loads(&text, &molang(context()));
    let Some(Geometry::Params(geometry)) = block.components.get::<Geometry>() else {
        panic!()
    };
    assert_eq!(
        geometry.bone_visibility.as_ref().unwrap().get("lock"),
        Some(&Molang::Bool(true))
    );
    assert_eq!(geometry.bone_visibility_len(), 3, "values kept");
    #[cfg(feature = "molang")]
    {
        use bedrock_addon::molang::DiagCode;
        let at = |suffix: &str| -> Vec<DiagCode> {
            block
                .diagnostics()
                .iter()
                .filter(|f| f.location.json_path.to_string().ends_with(suffix))
                .filter_map(|f| match f.detail.as_deref() {
                    Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(m)) => Some(m.code()),
                    _ => None,
                })
                .collect()
        };
        assert!(at("bone_visibility/hinge").contains(&DiagCode::InvalidOperation));
        let tags = at("block_filter/1/tags");
        assert!(
            tags.contains(&DiagCode::UnknownQuery) && tags.contains(&DiagCode::Syntax),
            "{tags:?}"
        );
        assert!(at("bone_visibility/lid").is_empty() && at("block_filter/0/tags").is_empty());
    }
    // The parser's line is level 3 and the block loads on: kept errors.
    #[cfg(feature = "molang")]
    {
        let molang_findings: Vec<_> = block
            .diagnostics()
            .iter()
            .filter(|f| {
                matches!(
                    f.detail.as_deref(),
                    Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(_))
                ) && (f
                    .location
                    .json_path
                    .to_string()
                    .ends_with("bone_visibility/hinge")
                    || f.location
                        .json_path
                        .to_string()
                        .ends_with("block_filter/1/tags"))
            })
            .collect();
        assert!(!molang_findings.is_empty());
        for f in molang_findings {
            assert!(
                f.severity == Severity::Error && f.kept,
                "{}: {:?} kept={}",
                f.location.json_path,
                f.severity,
                f.kept
            );
        }
    }
}

/// An item descriptor's `tags` of `item_specific_speeds` is its own spec, a kept
/// error.
#[cfg(feature = "molang")]
#[test]
fn item_specific_speeds_tags_is_a_kept_error() {
    let text = modern_components(
        r#"{"minecraft:destructible_by_mining": {"seconds_to_destroy": 1, "item_specific_speeds": [{"item": {"tags": "q.block_state('x')"}, "destroy_speed": 2}]}}"#,
    );
    let block = loads(&text, &molang(context()));
    let found: Vec<_> = block
        .diagnostics()
        .iter()
        .filter(|f| {
            matches!(
                f.detail.as_deref(),
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(_))
            ) && f.location.json_path.to_string().ends_with("item/tags")
        })
        .collect();
    assert!(!found.is_empty(), "{:#?}", brief(block.diagnostics()));
    for f in found {
        assert!(f.severity == Severity::Error && f.kept, "{f:?}");
    }
}

/// The `condition` of a legacy trigger and of a legacy event is a kept error
/// (the parsed result is dropped).
#[cfg(feature = "molang")]
#[test]
fn a_legacy_trigger_and_event_condition_is_a_kept_error() {
    let text = document(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:on_interact": {"event": "e", "condition": "'a' + 1"}}, "events": {"e": {"condition": "'a' + 1", "responses": [{"run_command": {"command": ["say hi"]}}]}}}"#,
    );
    let mut cx = allow_listed();
    cx = molang(cx);
    let block = loads(&text, &cx);
    let found: Vec<_> = block
        .diagnostics()
        .iter()
        .filter(|f| {
            matches!(
                f.detail.as_deref(),
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(_))
            ) && f.location.json_path.to_string().ends_with("condition")
        })
        .collect();
    assert!(
        found.len() >= 2,
        "trigger and event: {:#?}",
        brief(block.diagnostics())
    );
    for f in found {
        assert!(f.severity == Severity::Error && f.kept, "{f:?}");
    }
}

/// The same loader as the item's logs the level-2
/// "Item event has both randomize and sequence tags. Sequence will be ignored."
/// for a block's legacy event: a warning at its `sequence`, the `sequence` not evaluated.
#[test]
fn a_block_event_with_randomize_and_sequence_warns_and_ignores_the_sequence() {
    let text = document(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": {
            "e": {"randomize": [{"weight": 1, "run_command": {"command": ["say a"]}}], "sequence": [{"condition": "'a' + 1", "run_command": {"command": ["say b"]}}]},
            "ok": {"sequence": [{"run_command": {"command": ["say c"]}}]}}}"#,
    );
    let block = loads(&text, &molang(allow_listed()));
    let told: Vec<(String, Severity, String)> = block
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "IgnoredMember")
        .map(|f| {
            (
                f.location.json_path.to_string(),
                f.severity,
                f.code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        told,
        [(
            "minecraft:block/events/e/sequence".to_owned(),
            Severity::Warning,
            "Item event has both randomize and sequence tags. Sequence will be ignored.".to_owned()
        )],
        "{:#?}",
        brief(block.diagnostics())
    );
    // no condition of the ignored sequence is compiled
    assert!(
        block
            .diagnostics()
            .iter()
            .all(|f| f.code.name() != "InvalidMolang"),
        "{:#?}",
        brief(block.diagnostics())
    );
}

/// `cx` with the Molang compile check (with the `molang` feature).
fn molang(cx: bedrock_addon::load::LoadContext) -> bedrock_addon::load::LoadContext {
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

/// `findings` and `errors` are used by the feature-gated halves.
#[allow(dead_code)]
fn _uses(text: &str) -> usize {
    let (_, d) = findings(text, &context());
    errors(&d).len() + warnings(&d).len()
}

/// The game's loader performance test document for 1.26.60 ("timing only; each document must
/// load"): the 1.21.100 perf
/// block (cardinal placement trait, four rotated permutations, boxes) loads without a warning or
/// an error, and its newest shape reads back to the same block, written the same again.
#[test]
fn loader_perf_tests_4_the_perf_block_loads_and_round_trips() {
    use bedrock_addon::document::Content;
    let document = loads(&fixture("loader_perf/LoaderPerfTests-4.json"), &context());
    assert!(
        errors(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(
        document
            .diagnostics()
            .iter()
            .all(|d| d.severity < Severity::Warning),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.value().permutations.len(), 4);
    let written = document.value().to_json().unwrap();
    let again = loads(&written, &context());
    assert_eq!(
        super::common::newest_shape(again.value()),
        super::common::newest_shape(document.value())
    );
    assert_eq!(again.value().to_json().unwrap(), written);
}

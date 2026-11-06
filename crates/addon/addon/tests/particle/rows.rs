//! The fixtures, one test per row (the Molang rows p25–p28 are in
//! `molang.rs`).

use bedrock_addon::families::particle::component::{
    ColorExpr, DirectionMode, EmitterLocalSpace, EmitterRateInstant, EmitterShapeBox,
    EmitterShapeCustom, EmitterShapePoint, ParticleAppearanceBillboard, ParticleAppearanceTinting,
    ParticleInitialSpeed, ParticleMotionCollision, TintColor,
};
use bedrock_addon::families::particle::{Curve, ParticleEffect};
use bedrock_addon::molang::Molang;
use bedrock_addon::version::ENGINE_VERSION;

use super::common::{
    BASE, CEREAL, LEGACY, assert_has, assert_lacks, at, component, context, document, effect,
    effect_with, errors, findings, fixture, json, load, loaded, loads, number, object,
    pack_context, rejected, texts, written,
};

/// `p01_below_floor`: no slice reads a version below 1.10.0.
#[test]
fn p01_below_floor() {
    let d = rejected(&effect("1.8.0", ""));
    assert_eq!(
        texts(&d),
        [
            "Error format_version UnsupportedVersion: unsupported version: no parser available for version 1.8.0"
        ]
    );
}

/// `p02_bezier_chain_1_10_0`: 1.15.0 is no gate; a `bezier_chain` curve loads at 1.10.0.
#[test]
fn p02_bezier_chain_1_10_0() {
    let text = effect_with(
        LEGACY,
        r#""curves": {"variable.fade": {"type": "bezier_chain", "input": "variable.particle_age", "nodes": {"0.0": {"value": 1, "slope": 0}, "1.0": {"value": 0, "slope": -1}}}}"#,
    );
    let effect = loaded(&text);
    assert_eq!(findings(effect.diagnostics()), [] as [&str; 0]);
    let curves = effect.curves.as_ref().expect("curves");
    let Some(Curve::BezierChain(curve)) = curves.get("variable.fade") else {
        panic!("{curves:?}");
    };
    assert_eq!(curve.input, object("variable.particle_age", 13));
    assert_eq!(curve.nodes.len(), 2);
    assert_eq!(curve.nodes["1.0"].values(), (0.0, 0.0));
    assert_eq!(curve.nodes["1.0"].slopes(), (-1.0, -1.0));
}

/// `p03_vanilla_1_26_10_serialised`: the machine-written vanilla file parses as the 1.20.80
/// struct; each expression object keeps its own version, whatever the pack's.
#[test]
fn p03_vanilla_1_26_10_serialised() {
    let text = fixture("p03_vanilla_1_26_10_serialised.json");
    let effect = loads(&text, &pack_context(1, 13, 0));
    assert_eq!(findings(effect.diagnostics()), [] as [&str; 0]);
    assert_eq!(
        effect.description.identifier.as_str(),
        "minecraft:arrow_spell_emitter"
    );
    let custom = effect.components.get::<EmitterShapeCustom>().unwrap();
    let direction = custom.direction.as_ref().unwrap();
    assert_eq!(direction[0], object("Math.random(-1, 1)", 0));
    // The same nodes under a newer pack: the object's version stands.
    let newer = loads(&text, &pack_context(1, 21, 100));
    assert_eq!(newer.value(), effect.value());
    // Written back and read again, it is the same document; the text names every member the
    // file has.
    let rewritten = effect.to_json().unwrap();
    let again = loads(&rewritten, &pack_context(1, 13, 0));
    assert_eq!(again.value(), effect.value());
    assert_eq!(again.to_json().unwrap(), rewritten);
    let source = json(&text);
    let payload = |value: &facet_value::Value| at(value, "particle_effect").cloned();
    assert_eq!(payload(&written(&effect)), payload(&source));
}

/// `p04_molang_from_pack`: a legacy string compiles at the pack's Molang version, never the
/// file's; the upgrade writes that version out.
#[test]
fn p04_molang_from_pack() {
    let text = effect(
        LEGACY,
        r#""minecraft:emitter_rate_manual": {"max_particles": "variable.count"}"#,
    );
    for ((major, minor, patch), version) in [((1, 16, 0), 0), ((1, 21, 100), 13)] {
        let effect = loads(&text, &pack_context(major, minor, patch));
        assert_eq!(
            at(
                &written(&effect),
                "particle_effect/components/minecraft:emitter_rate_manual/max_particles"
            ),
            Some(&json(&format!(
                r#"{{"expression": "variable.count", "version": {version}}}"#
            )))
        );
    }
}

/// `p05_missing_required`: a legacy effect without a rate component gets vanilla's line and
/// loads (a warning: an error would refuse the document).
#[test]
fn p05_missing_required() {
    let text = document(
        LEGACY,
        r#""components": {"minecraft:emitter_lifetime_once": {}}"#,
    );
    let effect = loaded(&text);
    assert_eq!(
        findings(effect.diagnostics()),
        [
            "Error particle_effect/components MissingRequiredField: Particle effect needs an emitter rate component!"
        ]
    );
    assert_eq!(effect.components.len(), 1);
}

/// `p06_upgrade_full`: one of each of the 29 components, two curves and one event at 1.10.0,
/// and the 1.20.80 document the upgrade gives.
#[test]
fn p06_upgrade_full() {
    let effect = loaded(&fixture("p06_upgrade_full.before.json"));
    assert_eq!(findings(effect.diagnostics()), [] as [&str; 0]);
    assert_eq!(effect.version().parsed_as.to_string(), "1.10.0");
    assert!(effect.version().upgraded);
    assert_eq!(effect.components.len(), 29);
    assert_eq!(effect.curves.as_ref().map(indexmap::IndexMap::len), Some(2));
    let event = &effect.events.as_ref().unwrap()["demo:burst"];
    assert_eq!(event.expression, Some(number(0.0)));
    let started = event.particle_effect.as_ref().unwrap();
    assert_eq!(started.pre_effect_expression, Some(number(0.0)));
    let after = json(&fixture("p06_upgrade_full.after.json"));
    assert_eq!(written(&effect), after);
    // The upgraded text is a 1.20.80 document of the same value.
    let again = loaded(&effect.to_json().unwrap());
    assert_eq!(again.value(), effect.value());
    assert_eq!(findings(again.diagnostics()), [] as [&str; 0]);
}

/// `p07_point_direction_default`: the legacy default direction is compiled at Molang version
/// 13 whatever the pack's version.
#[test]
fn p07_point_direction_default() {
    let text = effect(LEGACY, r#""minecraft:emitter_shape_point": {}"#);
    let effect = loads(&text, &pack_context(1, 16, 0));
    let point = effect.components.get::<EmitterShapePoint>().unwrap();
    let random = object("math.random(-1, 1)", 13);
    assert_eq!(
        point.direction,
        Some([random.clone(), random.clone(), random])
    );
    // A 1.20.80 document has no default direction.
    let cereal = loaded(&super::common::effect(
        CEREAL,
        r#""minecraft:emitter_shape_point": {}"#,
    ));
    let point = cereal.components.get::<EmitterShapePoint>().unwrap();
    assert_eq!(point.direction, None);
}

/// `p08_min_speed_threshold`: the legacy default 0.01 is written out by the upgrade; a
/// 1.20.80 document without the member has the default 0.
#[test]
fn p08_min_speed_threshold() {
    let billboard = r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "direction": {"mode": "custom", "custom_direction": [0, 1, 0]}}"#;
    let legacy = loaded(&effect(LEGACY, billboard));
    let settings = |effect: &ParticleEffect| {
        let billboard = effect.components.get::<ParticleAppearanceBillboard>();
        billboard.unwrap().direction.clone().unwrap()
    };
    assert_eq!(settings(&legacy).mode, DirectionMode::Custom);
    assert_eq!(settings(&legacy).min_speed_threshold, Some(0.01));
    let cereal = loaded(&effect(CEREAL, billboard));
    assert_eq!(settings(&cereal).min_speed_threshold, None);
    assert_eq!(settings(&cereal).min_speed_threshold(), 0.0);
}

/// `p09_initial_speed_shapes`: a scalar is the speed on every axis; an array overrides the
/// initial direction; both in both eras.
#[test]
fn p09_initial_speed_shapes() {
    for version in [LEGACY, CEREAL] {
        let uniform = loaded(&effect(version, r#""minecraft:particle_initial_speed": 2"#));
        let speed = uniform.components.get::<ParticleInitialSpeed>().unwrap();
        assert_eq!(speed.speed(), [number(2.0), number(2.0), number(2.0)]);
        assert!(!speed.overrides_initial_direction(), "{version}");
        assert_eq!(
            component(&uniform, "minecraft:particle_initial_speed"),
            json("2")
        );
        let vector = loaded(&effect(
            version,
            r#""minecraft:particle_initial_speed": [1, 2, 3]"#,
        ));
        let speed = vector.components.get::<ParticleInitialSpeed>().unwrap();
        assert_eq!(speed.speed(), [number(1.0), number(2.0), number(3.0)]);
        assert!(speed.overrides_initial_direction(), "{version}");
        assert_eq!(
            component(&vector, "minecraft:particle_initial_speed"),
            json("[1, 2, 3]")
        );
    }
}

/// `p10_object_molang_legacy`: the legacy parser has no object form; the 1.20.80 reader keeps
/// it with its version.
#[test]
fn p10_object_molang_legacy() {
    let components = r#"{"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": {"expression": "5", "version": 12}}}"#;
    let legacy = loaded(&document(LEGACY, &format!(r#""components": {components}"#)));
    assert_eq!(
        findings(legacy.diagnostics()),
        [
            "Error particle_effect/components/minecraft:emitter_rate_instant/num_particles InvalidValue: field is not a number or a Molang expression"
        ]
    );
    let rate = legacy.components.get::<EmitterRateInstant>().unwrap();
    assert_eq!(rate.num_particles, number(0.0));
    let cereal = loaded(&document(CEREAL, &format!(r#""components": {components}"#)));
    assert_eq!(findings(cereal.diagnostics()), [] as [&str; 0]);
    let rate = cereal.components.get::<EmitterRateInstant>().unwrap();
    assert_eq!(rate.num_particles, object("5", 12));
}

/// `p11_local_space_coercion`: rotation without position is not supported, and velocity
/// counts only when both are false; in both eras.
#[test]
fn p11_local_space_coercion() {
    let space = "particle_effect/components/minecraft:emitter_local_space";
    for version in [LEGACY, CEREAL] {
        let rotated = loaded(&effect(
            version,
            r#""minecraft:emitter_local_space": {"position": false, "rotation": true, "velocity": true}"#,
        ));
        assert_has(
            rotated.diagnostics(),
            &format!(
                "Error {space}/rotation InvalidValue: We do not support rotation being true and position being false"
            ),
        );
        let stored = rotated.components.get::<EmitterLocalSpace>().unwrap();
        assert_eq!(
            (stored.position, stored.rotation, stored.velocity),
            (Some(false), Some(false), Some(false)),
            "{version}"
        );
        let positioned = loaded(&effect(
            version,
            r#""minecraft:emitter_local_space": {"position": true, "velocity": true}"#,
        ));
        assert_eq!(
            findings(positioned.diagnostics()),
            [format!("Info {space}/velocity InvalidValue: invalid value")],
            "{version}"
        );
        let stored = positioned.components.get::<EmitterLocalSpace>().unwrap();
        assert_eq!(
            (stored.position, stored.rotation, stored.velocity),
            (Some(true), None, Some(false)),
            "{version}"
        );
        // Velocity alone is honoured.
        let free = loaded(&effect(
            version,
            r#""minecraft:emitter_local_space": {"position": false, "rotation": false, "velocity": true}"#,
        ));
        assert_eq!(findings(free.diagnostics()), [] as [&str; 0], "{version}");
        let stored = free.components.get::<EmitterLocalSpace>().unwrap();
        assert_eq!(stored.velocity, Some(true), "{version}");
    }
}

/// `p12_no_description`: ours rejects where vanilla registers an effect without a name.
#[test]
fn p12_no_description_deviation() {
    let text = format!(
        r#"{{"format_version": "1.10.0", "particle_effect": {{"components": {{{BASE}}}}}}}"#
    );
    let d = rejected(&text);
    assert_eq!(
        texts(&d),
        [
            "Error particle_effect/description MissingRequiredField: Particle effects file has no description block!"
        ]
    );
}

/// `p13_strict_unknown_1_20_80`: an unknown component is a warning in both eras (vanilla
/// refuses the 1.20.80 document); the legacy line names the key.
#[test]
fn p13_strict_unknown_1_20_80() {
    // The cereal slice is registered with `StrictErrorCodes` 0xfffc:
    // an unknown component refuses the document.
    let unknown = r#""x:y": {"a": 1}"#;
    let cereal = rejected(&effect(CEREAL, unknown));
    assert!(
        findings(&cereal).contains(
            &"Error particle_effect/components/x:y UnrecognizedComponent: unrecognized component"
                .to_owned()
        ),
        "{:?}",
        findings(&cereal)
    );
    // The legacy parser reads past it with its content-log line; the document loads.
    let legacy = loaded(&effect(LEGACY, unknown));
    assert_eq!(
        findings(legacy.diagnostics()),
        [
            "Error particle_effect/components/x:y UnrecognizedComponent: Unrecognized component name 'x:y' in particle effect"
        ]
    );
}

/// `p14_curve_nodes`: the node counts a kind takes, at 1.10.0.
#[test]
fn p14_curve_nodes() {
    let nodes = |count: usize| vec!["0"; count].join(", ");
    let curve = |kind: &str, count: usize| {
        effect_with(
            LEGACY,
            &format!(
                r#""curves": {{"variable.c": {{"type": "{kind}", "input": 0, "nodes": [{}]}}}}"#,
                nodes(count)
            ),
        )
    };
    let line = "Error particle_effect/curves/variable.c/nodes InvalidValue: Invalid number of control nodes in curve!  Defaulting curve to 0.0";
    for (kind, count) in [("bezier", 3), ("catmull_rom", 3), ("linear", 101)] {
        let effect = loaded(&curve(kind, count));
        assert_eq!(findings(effect.diagnostics()), [line], "{kind} {count}");
    }
    // Both ends of each range.
    for (kind, count) in [
        ("bezier", 4),
        ("catmull_rom", 4),
        ("catmull_rom", 100),
        ("linear", 1),
        ("linear", 100),
    ] {
        let effect = loaded(&curve(kind, count));
        assert_eq!(
            findings(effect.diagnostics()),
            [] as [&str; 0],
            "{kind} {count}"
        );
    }
    for (kind, count) in [("bezier", 5), ("catmull_rom", 101), ("linear", 0)] {
        let effect = loaded(&curve(kind, count));
        assert_eq!(findings(effect.diagnostics()), [line], "{kind} {count}");
    }
}

/// `p15_above_engine`: no slice reads a version above the engine's.
#[test]
fn p15_above_engine() {
    let engine = ENGINE_VERSION;
    let above = format!("{}.{}.{}", engine.major, engine.minor, engine.patch + 10);
    let d = rejected(&effect(&above, ""));
    assert_eq!(
        texts(&d),
        [format!(
            "Error format_version UnsupportedVersion: unsupported version: no parser available for version {above}"
        )]
    );
    // The engine's own version is the last one read.
    let at_engine = format!("{}.{}.{}", engine.major, engine.minor, engine.patch);
    loaded(&effect(&at_engine, ""));
}

/// `p16_vanilla_1_26_30_geyser`: a hand-written vanilla file at 1.26.30 is read by the
/// 1.20.80 struct; its strings take the pack's Molang version, and its local space is stored
/// as the game stores it. The one finding is ours: the file's `velocity: true` beside
/// `position: true` counts for nothing.
#[test]
fn p16_vanilla_1_26_30_geyser() {
    let text = fixture("p16_vanilla_1_26_30_geyser.json");
    let effect = loads(&text, &pack_context(1, 26, 30));
    assert_eq!(effect.version().parsed_as.to_string(), "1.20.80");
    assert!(!effect.version().upgraded);
    assert_eq!(
        findings(effect.diagnostics()),
        [
            "Info particle_effect/components/minecraft:emitter_local_space/velocity InvalidValue: invalid value"
        ]
    );
    let space = effect.components.get::<EmitterLocalSpace>().unwrap();
    assert_eq!(
        (space.position, space.rotation, space.velocity),
        (Some(true), Some(false), Some(false))
    );
    let shape = effect.components.get::<EmitterShapeBox>().unwrap();
    let offset = shape.offset.as_ref().unwrap();
    let Molang::Expr(source) = &offset[0] else {
        panic!("{offset:?}");
    };
    assert_eq!(source.as_str(), "Math.random(-0.4, 0.4)");
    assert_eq!(
        source.raw_version(),
        pack_context(1, 26, 30).molang_version.into()
    );
    let again = loads(&effect.to_json().unwrap(), &pack_context(1, 26, 30));
    assert_eq!(again.value(), effect.value());
}

/// `p17_camera_distance_fade`: the legacy registry's 30th name is accepted at 1.10.0 with any
/// value, dropped by the upgrade with an info (vanilla: silently), and an unknown component
/// that refuses the document from 1.20.80.
#[test]
fn p17_camera_distance_fade_deviation() {
    let fade = "minecraft:particle_appearance_camera_distance_fade";
    let at_fade = format!("particle_effect/components/{fade}");
    for value in [r#"{"distance": [4, 16], "alpha": [1, 0]}"#, "7", "[]"] {
        let legacy = loaded(&effect(LEGACY, &format!(r#""{fade}": {value}"#)));
        assert_eq!(
            findings(legacy.diagnostics()),
            [format!("Info {at_fade} NoEffect: member has no effect")],
            "{value}"
        );
        assert_eq!(legacy.components.len(), 2, "{value}");
        assert_eq!(at(&written(&legacy), &at_fade), None, "{value}");
    }
    // From 1.20.80 an unknown component under the strict mask 0xfffc: refused.
    let cereal = rejected(&effect(
        "1.26.30",
        &format!(r#""{fade}": {{"distance": [4, 16], "alpha": [1, 0]}}"#),
    ));
    assert_eq!(
        findings(&cereal),
        [format!(
            "Error {at_fade} UnrecognizedComponent: unrecognized component"
        )]
    );
}

/// `p18_envelope_split`: below 1.20.80 the legacy parser reads the text itself; from 1.20.80
/// the envelope has exactly two members.
#[test]
fn p18_envelope_split_deviation() {
    let with_extra = |version: &str| {
        let text = effect(version, "");
        format!(r#"{{"x": 1, {}"#, &text[1..])
    };
    let legacy = loaded(&with_extra(LEGACY));
    assert_eq!(
        findings(legacy.diagnostics()),
        ["Info x UnrecognizedField: unrecognized field"]
    );
    assert_eq!(
        texts(&rejected(&with_extra("1.26.30"))),
        [
            "Error  MalformedInput: A Minecraft Document must have exactly two members: \"format_version\" and a payload member"
        ]
    );
    // The boundary: 1.20.79 is still the raw text, 1.20.80 the envelope.
    loaded(&with_extra("1.20.79"));
    assert_eq!(errors(&rejected(&with_extra("1.20.80"))).len(), 1);
    assert_eq!(
        texts(&rejected(r#"{"format_version": "1.26.30"}"#)),
        ["Error  MissingPayload: The payload of the Minecraft Document is missing"]
    );
    assert_eq!(
        texts(&rejected(r#"{"format_version": "1.10.0"}"#)),
        ["Error particle_effect MissingRequiredField: no particle_effect field was given"]
    );
    // The game builds its input with the key `particle_effect`, so a payload keyed
    // `particle_effects` fails the payload-key check and logs A11 with 0x10.
    let renamed = effect("1.26.30", "").replace("\"particle_effect\"", "\"particle_effects\"");
    assert_eq!(
        texts(&rejected(&renamed)),
        [
            "Error  MissingPayload: The payload key does not match the expected one. Expected: 'particle_effect'. Actual: 'particle_effects'."
        ]
    );
}

/// `p19_required_1_20_80`: three members the 1.20.80 reader requires and the legacy parser
/// has a fallback for (vanilla refuses the 1.20.80 document; so do we).
#[test]
fn p19_required_1_20_80() {
    let components = "particle_effect/components";
    let tinting = r#""minecraft:particle_appearance_tinting": {}"#;
    assert_eq!(
        texts(&rejected(&effect(CEREAL, tinting))),
        [format!(
            "Error {components}/minecraft:particle_appearance_tinting/color MissingRequiredField: missing required field"
        )]
    );
    let legacy = loaded(&effect(LEGACY, tinting));
    assert_eq!(
        findings(legacy.diagnostics()),
        [format!(
            "Error {components}/minecraft:particle_appearance_tinting/color InvalidValue: Component has no color field!"
        )]
    );
    let white = [number(1.0), number(1.0), number(1.0), number(1.0)];
    assert_eq!(
        legacy
            .components
            .get::<ParticleAppearanceTinting>()
            .unwrap()
            .color,
        TintColor::Color(ColorExpr::Rgba(white))
    );

    let billboard = r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "direction": {"custom_direction": [0, 1, 0]}}"#;
    assert_eq!(
        texts(&rejected(&effect(CEREAL, billboard))),
        [format!(
            "Error {components}/minecraft:particle_appearance_billboard/direction/mode MissingRequiredField: missing required field"
        )]
    );
    let legacy = loaded(&effect(LEGACY, billboard));
    assert_eq!(findings(legacy.diagnostics()), [] as [&str; 0]);
    let stored = legacy
        .components
        .get::<ParticleAppearanceBillboard>()
        .unwrap();
    assert_eq!(
        stored.direction.as_ref().unwrap().mode,
        DirectionMode::DeriveFromVelocity
    );

    let events = r#""events": {"e": {"randomize": [{"log": "x"}]}}"#;
    assert_eq!(
        texts(&rejected(&effect_with(CEREAL, events))),
        [
            "Error particle_effect/events/e/randomize/0/weight MissingRequiredField: missing required field"
        ]
    );
    let legacy = loaded(&effect_with(LEGACY, events));
    assert_eq!(
        findings(legacy.diagnostics()),
        [
            "Error particle_effect/events/e/randomize/0/weight InvalidValue: Error reading weight field!  Ignoring random option",
            // The option goes with its `log`.
            "Info particle_effect/events/e/randomize/0 DroppedField: field dropped",
        ]
    );
    let node = &legacy.events.as_ref().unwrap()["e"];
    assert_eq!(node.randomize.as_ref().map(Vec::len), Some(0));
}

/// `p20_min_speed_floor_deviation`: the legacy parser raises `min_speed` to 2 (ours says so at info);
/// the 1.20.80 reader keeps the value.
#[test]
fn p20_min_speed_floor_deviation() {
    let collision = r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [{"event": "e", "min_speed": 0.5}]}"#;
    let speed = |effect: &ParticleEffect| {
        let collision = effect.components.get::<ParticleMotionCollision>().unwrap();
        collision.events.as_ref().unwrap().as_slice()[0].min_speed
    };
    let legacy = loaded(&effect(LEGACY, collision));
    assert_eq!(speed(&legacy), Some(2.0));
    assert_eq!(
        findings(legacy.diagnostics()),
        [
            "Info particle_effect/components/minecraft:particle_motion_collision/events/0/min_speed InvalidValue: invalid value"
        ]
    );
    let cereal = loaded(&effect(CEREAL, collision));
    assert_eq!(speed(&cereal), Some(0.5));
    assert_eq!(findings(cereal.diagnostics()), [] as [&str; 0]);
}

/// `p21_uv_messages`: a 1.20.80 `uv` without a flipbook says what it misses.
#[test]
fn p21_uv_messages() {
    let billboard = |uv: &str| {
        effect(
            CEREAL,
            &format!(
                r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "uv": {uv}}}"#
            ),
        )
    };
    let at_uv = "particle_effect/components/minecraft:particle_appearance_billboard/uv";
    let bare = loaded(&billboard(r#"{"texture_width": 16, "texture_height": 16}"#));
    assert_eq!(
        findings(bare.diagnostics()),
        [
            format!("Error {at_uv} InvalidValue: missing uv field"),
            format!("Error {at_uv} InvalidValue: missing uv_size field"),
        ]
    );
    let stored = bare
        .components
        .get::<ParticleAppearanceBillboard>()
        .unwrap();
    let uv = stored.uv.as_ref().unwrap();
    assert_eq!((uv.uv.clone(), uv.uv_size.clone()), (None, None));
    assert_eq!(uv.uv(), [number(0.0), number(0.0)]);
    assert_eq!(uv.uv_size(), [number(1.0), number(1.0)]);
    let flipbook = loaded(&billboard(
        r#"{"texture_width": 16, "texture_height": 16, "flipbook": {"base_UV": [0, 0], "max_frame": 4}}"#,
    ));
    assert_eq!(findings(flipbook.diagnostics()), [] as [&str; 0]);
    // One of the two alone.
    let no_size = loaded(&billboard(r#"{"uv": [0, 0]}"#));
    assert_eq!(
        findings(no_size.diagnostics()),
        [format!("Error {at_uv} InvalidValue: missing uv_size field")]
    );
    let no_uv = loaded(&billboard(r#"{"uv_size": [1, 1]}"#));
    assert_eq!(
        findings(no_uv.diagnostics()),
        [format!("Error {at_uv} InvalidValue: missing uv field")]
    );
}

/// `p22_color_rgb_array`: three channels are a colour with alpha 1; a string that is no
/// `#` colour gets vanilla's line and the document loads.
#[test]
fn p22_color_rgb_array() {
    let tinting = |color: &str| {
        effect(
            CEREAL,
            &format!(r#""minecraft:particle_appearance_tinting": {{"color": {color}}}"#),
        )
    };
    let rgb = loaded(&tinting("[1, 0.5, 0]"));
    assert_eq!(findings(rgb.diagnostics()), [] as [&str; 0]);
    assert_eq!(
        rgb.components
            .get::<ParticleAppearanceTinting>()
            .unwrap()
            .color,
        TintColor::Color(ColorExpr::Rgb([number(1.0), number(0.5), number(0.0)]))
    );
    let red = loaded(&tinting(r#""red""#));
    assert_eq!(
        findings(red.diagnostics()),
        [
            "Error particle_effect/components/minecraft:particle_appearance_tinting/color InvalidValue: Color value string has problems!"
        ]
    );
    let hex = loaded(&tinting(r##""#80FF0000""##));
    assert_eq!(findings(hex.diagnostics()), [] as [&str; 0]);
    let TintColor::Color(color) = &hex
        .components
        .get::<ParticleAppearanceTinting>()
        .unwrap()
        .color
    else {
        panic!();
    };
    assert_eq!(color.argb(), Some([0x80, 0xFF, 0, 0]));
}

/// `p23_legacy_array_length`: the legacy parser takes an array of the exact length or none
/// of it; the 1.20.80 reader's array has a fixed size.
#[test]
fn p23_legacy_array_length() {
    let shape = r#""minecraft:emitter_shape_box": {"half_dimensions": [1, 2]}"#;
    let at_member = "particle_effect/components/minecraft:emitter_shape_box/half_dimensions";
    let legacy = loaded(&effect(LEGACY, shape));
    assert_eq!(
        findings(legacy.diagnostics()),
        [format!(
            "Error {at_member} InvalidValue: error reading array"
        )]
    );
    assert_eq!(
        legacy
            .components
            .get::<EmitterShapeBox>()
            .unwrap()
            .half_dimensions,
        [number(0.0), number(0.0), number(0.0)]
    );
    assert_eq!(
        texts(&rejected(&effect(CEREAL, shape))),
        [format!("Error {at_member} InvalidType: wrong JSON type")]
    );
    // One too many is as wrong as one too few.
    let four = r#""minecraft:emitter_shape_box": {"half_dimensions": [1, 2, 3, 4]}"#;
    assert_eq!(
        findings(loaded(&effect(LEGACY, four)).diagnostics()).len(),
        1
    );
    assert_eq!(errors(&rejected(&effect(CEREAL, four))).len(), 1);
}

/// `p24_curve_nodes_1_20_80`: the 1.20.80 conversion keeps the nodes it was given and appends
/// the least number of zero nodes; a chain node needs its value on both sides.
#[test]
fn p24_curve_nodes_1_20_80() {
    let bezier = loaded(&effect_with(
        CEREAL,
        r#""curves": {"variable.c": {"type": "bezier", "input": 0, "nodes": [1, 2, 3]}}"#,
    ));
    assert_eq!(
        findings(bezier.diagnostics()),
        [
            "Error particle_effect/curves/variable.c/nodes InvalidValue: Invalid number of control nodes in curve!  Defaulting curve to 0.0"
        ]
    );
    let Some(Curve::Simple(curve)) = bezier.curves.as_ref().unwrap().get("variable.c") else {
        panic!();
    };
    assert_eq!(curve.nodes, [number(1.0), number(2.0), number(3.0)]);
    assert_eq!(
        curve.effective_nodes(),
        [1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0].map(number)
    );
    let chain = loaded(&effect_with(
        CEREAL,
        r#""curves": {"variable.c": {"type": "bezier_chain", "input": 0, "nodes": {"0.0": {"left_value": 1, "slope": 0}}}}"#,
    ));
    assert_eq!(
        findings(chain.diagnostics()),
        [
            "Error particle_effect/curves/variable.c/nodes/0.0 InvalidValue: Particle curve bezier chain node is missing required value field"
        ]
    );
}

/// The rows' documents are the loader's own: `load` and `loaded` agree on a plain effect.
#[test]
fn a_plain_effect_loads_in_both_eras_without_a_finding() {
    for version in [LEGACY, "1.20.79", "1.20.80", CEREAL, "1.26.30"] {
        let document = load(&effect(version, ""), &context()).expect(version);
        assert_eq!(
            findings(document.diagnostics()),
            [] as [&str; 0],
            "{version}"
        );
        assert_eq!(document.description.identifier.as_str(), "demo:spark");
    }
    assert_lacks(loaded(&effect(LEGACY, "")).diagnostics(), "component!");
    // By legacy name: the variant and the definitions that were it.
    assert_eq!(
        rejected("{").iter().next().map(|d| d.code.name()),
        Some("MalformedInput")
    );
}

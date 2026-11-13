//! The validation rules and the reader lines of the schema: each rule
//! at its boundaries, in both eras, with vanilla's text where it has one.

use bedrock_addon_core::molang::Molang;
use bedrock_addon_family_particle::component::{
    EmitterInitialization, EmitterLifetimeEvents, EmitterRateInstant, EmitterShapeSphere, Gradient,
    ParticleAppearanceBillboard, ParticleAppearanceTinting, ParticleExpireIfInBlocks,
    ParticleKillPlane, ParticleMotionCollision, TintColor,
};
use bedrock_addon_family_particle::{Curve, ParticleEffect};

use super::common::{
    BASE, CEREAL, DESCRIPTION, LEGACY, assert_has, component, context, document, effect,
    effect_with, errors, findings, json, load, loaded, number, object, rejected, texts, written,
};

const COMPONENTS: &str = "particle_effect/components";

/// The findings of a document that loads.
fn found(text: &str) -> Vec<String> {
    findings(loaded(text).diagnostics())
}

/// The one finding of a document that is rejected.
#[track_caller]
fn refusal(text: &str) -> String {
    let d = rejected(text);
    let all = texts(&d);
    assert_eq!(errors(&d).len(), 1, "{all:#?}");
    let refusing = super::common::text(errors(&d)[0]);
    all.into_iter().find(|t| *t == refusing).unwrap()
}

// ---- rule 1 and 2: versions and the envelope -------------------------------------------------

#[test]
fn rule_1_the_range_is_the_floor_to_the_engine() {
    for version in ["1.9.99", "1.0.0", "0.0.0"] {
        assert!(
            refusal(&effect(version, "")).contains("core/no_parser_for_version"),
            "{version}"
        );
    }
    for version in [
        "1.10.0", "1.10.1", "1.19.0", "1.20.79", "1.20.80", "1.21.0", "1.26.50", "1.26.60",
    ] {
        loaded(&effect(version, ""));
    }
    assert!(refusal(&effect("1.26.61", "")).contains("core/no_parser_for_version"));
    // The struct follows the version.
    assert_eq!(
        loaded(&effect("1.20.79", ""))
            .version()
            .parsed_as
            .to_string(),
        "1.10.0"
    );
    assert_eq!(
        loaded(&effect("1.20.80", ""))
            .version()
            .parsed_as
            .to_string(),
        "1.20.80"
    );
}

/// Ours: vanilla refuses a document without a version ("no valid version passed by the
/// caller or found in the input"); we read it at the floor with a warning.
#[test]
fn rule_2_a_missing_format_version_is_the_floor_with_a_warning_deviation() {
    let text = effect(LEGACY, "").replace(r#""format_version": "1.10.0", "#, "");
    let effect = loaded(&text);
    assert_eq!(
        findings(effect.diagnostics()),
        ["Info  core/missing_format_version: no format_version"]
    );
    assert_eq!(effect.version().parsed_as.to_string(), "1.10.0");
    assert_eq!(effect.version().written, None);
}

#[test]
fn rule_2_an_unreadable_format_version_is_malformed() {
    // `initAsJson` reads a number as its text: `1` is 1.0.0,
    // below the floor, which has no parser.
    let text = effect(LEGACY, "").replace(r#""1.10.0""#, "1");
    assert!(
        refusal(&text).contains("no parser available for version 1.0.0"),
        "{}",
        refusal(&text)
    );
    for version in [r#""x""#, "true", "{}", "null", "[1, 2]", r#"[1, 2, "3"]"#] {
        let text = effect(LEGACY, "").replace(r#""1.10.0""#, version);
        assert!(
            refusal(&text).contains("core/malformed_format_version"),
            "{version}: {}",
            refusal(&text)
        );
    }
    // Three integers are a version.
    let text = effect(LEGACY, "").replace(r#""1.10.0""#, "[1, 26, 10]");
    assert_eq!(loaded(&text).version().parsed_as.to_string(), "1.20.80");
}

/// A member written twice keeps its last value, whole, before anything reads the document:
/// the version, the identifier, a component, a member of one.
#[test]
fn a_member_written_twice_keeps_its_last_value() {
    // The version alone is the first textual `"format_version"` of the file, before any
    // member is blanked (`initAsJson`): a first copy below the floor has no parser.
    let twice = |version: &str| {
        format!(
            r#"{{"format_version": "1.0.0", "format_version": "{version}", "particle_effect": {{{DESCRIPTION}, "components": {{{BASE}}}}}}}"#
        )
    };
    for version in [LEGACY, CEREAL] {
        assert!(
            refusal(&twice(version)).contains("no parser available for version 1.0.0"),
            "{version}: {}",
            refusal(&twice(version))
        );
    }
    for version in [LEGACY, CEREAL] {
        let text = format!(
            r#"{{"format_version": "{version}", "particle_effect": {{"description": {{"identifier": "demo:first"}}, {DESCRIPTION}, "components": {{{BASE}, "minecraft:emitter_rate_instant": {{"num_particles": 2, "num_particles": 3}}, "minecraft:emitter_shape_sphere": {{"radius": [1]}}, "minecraft:emitter_shape_sphere": {{"radius": 4}}}}}}}}"#
        );
        let effect = loaded(&text);
        assert_eq!(findings(effect.diagnostics()), [] as [&str; 0], "{version}");
        assert_eq!(effect.description.identifier.as_str(), "demo:spark");
        let rate = effect.components.get::<EmitterRateInstant>().unwrap();
        assert_eq!(rate.num_particles, number(3.0), "{version}");
        let sphere = effect.components.get::<EmitterShapeSphere>().unwrap();
        assert_eq!(sphere.radius, Some(number(4.0)), "{version}");
    }
}

// ---- rule 3: the legacy structure ------------------------------------------------------------

/// The legacy parser stops before it has an identifier; the game registers an effect named
/// `""`, ours rejects the document.
#[test]
fn rule_3_a_legacy_document_without_a_description_or_an_identifier_is_rejected_deviation() {
    let payload = |members: &str| {
        format!(r#"{{"format_version": "1.10.0", "particle_effect": {{{members}}}}}"#)
    };
    let components = format!(r#""components": {{{BASE}}}"#);
    for (members, expected) in [
        (
            format!(r#""description": 1, {components}"#),
            "Error particle_effect/description particle/no_description: Particle effects file has no description block!",
        ),
        (
            format!(
                r#""description": {{"basic_render_parameters": {{"material": "m", "texture": "t"}}}}, {components}"#
            ),
            "Error particle_effect/description/identifier particle/no_identifier: Particle effect description has no identifier field!",
        ),
        (
            format!(
                r#""description": {{"identifier": 1, "basic_render_parameters": {{"material": "m", "texture": "t"}}}}, {components}"#
            ),
            "Error particle_effect/description/identifier particle/no_identifier: Particle effect description has no identifier field!",
        ),
    ] {
        assert_eq!(refusal(&payload(&members)), expected);
    }
}

/// Without `basic_render_parameters` or `components` the legacy parser stops as well, with
/// the identifier read: the effect is what it had by then.
#[test]
fn rule_3_missing_render_parameters_or_components_leave_the_effect_as_far_as_it_was_read() {
    let curves = r#""curves": {"variable.c": {"type": "linear", "input": 0, "nodes": [0]}}, "events": {"e": {"log": "x"}}"#;
    let payload = |members: &str| {
        format!(r#"{{"format_version": "1.10.0", "particle_effect": {{{members}, {curves}}}}}"#)
    };
    let components = format!(r#""components": {{{BASE}}}"#);
    // No render parameters: nothing after the description is read.
    for description in [
        r#"{"identifier": "demo:x"}"#,
        r#"{"identifier": "demo:x", "basic_render_parameters": 1}"#,
    ] {
        let effect = loaded(&payload(&format!(
            r#""description": {description}, {components}"#
        )));
        assert_eq!(
            findings(effect.diagnostics()),
            [
                "Error particle_effect/description/basic_render_parameters particle/no_render_parameters: basic_render_parameters not found when reading in particle effect",
                // What the parser does not get to is noted where the file has it.
                "Info particle_effect/curves core/dropped_field: field dropped",
                "Info particle_effect/events core/dropped_field: field dropped",
                "Info particle_effect/components core/dropped_field: field dropped",
            ],
            "{description}"
        );
        assert_eq!(effect.description.identifier.as_str(), "demo:x");
        assert_eq!(effect.description.basic_render_parameters.material, "");
        assert_eq!(effect.description.basic_render_parameters.texture, "");
        assert_eq!(effect.components.len(), 0, "{description}");
        assert_eq!(effect.curves, None, "{description}");
        assert_eq!(effect.events, None, "{description}");
    }
    // No components: the curves and the events were read before.
    for members in [
        DESCRIPTION.to_owned(),
        format!(r#"{DESCRIPTION}, "components": []"#),
    ] {
        let effect = loaded(&payload(&members));
        assert_eq!(
            findings(effect.diagnostics()),
            [
                "Error particle_effect/components particle/no_components: no components found when reading in particle effect"
            ],
            "{members}"
        );
        assert_eq!(effect.components.len(), 0, "{members}");
        assert_eq!(effect.curves.as_ref().map(indexmap::IndexMap::len), Some(1));
        assert_eq!(effect.events.as_ref().map(indexmap::IndexMap::len), Some(1));
        // What is registered is a 1.20.80 effect whose text loads again.
        let again =
            loaded(&bedrock_addon_core::document::Content::to_json(effect.value()).unwrap());
        assert_eq!(again.value(), effect.value(), "{members}");
    }
    // From 1.20.80 both are required members, and the document is refused.
    for (members, at) in [
        (
            format!(r#""description": {{"identifier": "demo:x"}}, {components}"#),
            "particle_effect/description/basic_render_parameters",
        ),
        (DESCRIPTION.to_owned(), "particle_effect/components"),
    ] {
        let text = payload(&members).replace("1.10.0", CEREAL);
        assert_eq!(
            refusal(&text),
            format!("Error {at} core/missing_required_field: missing required field")
        );
    }
}

/// A legacy effect without a material or a texture is registered as far as the parser got:
/// no component. Each line is vanilla's, as a warning.
#[test]
fn rule_3_missing_render_parameters_stop_the_legacy_parser() {
    let brp = "particle_effect/description/basic_render_parameters";
    let with = |parameters: &str| {
        format!(
            r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "demo:x", "basic_render_parameters": {parameters}}}, "components": {{{BASE}}}, "curves": {{"variable.c": {{"type": "linear", "input": 0, "nodes": [0]}}}}}}}}"#
        )
    };
    for (parameters, expected) in [
        (
            r#"{"texture": "t"}"#,
            format!("Error {brp}/material particle/no_material: particle has no material!"),
        ),
        (
            r#"{"material": 1, "texture": "t"}"#,
            format!(
                "Error {brp}/material particle/material_not_string: particle material field isn't a string"
            ),
        ),
        (
            r#"{"material": "m"}"#,
            format!("Error {brp}/texture particle/no_texture: particle has no texture!"),
        ),
    ] {
        let effect = loaded(&with(parameters));
        assert_eq!(
            findings(effect.diagnostics()),
            [
                expected,
                // What the parser does not get to is noted where the file has it.
                "Info particle_effect/curves core/dropped_field: field dropped".to_owned(),
                "Info particle_effect/components core/dropped_field: field dropped".to_owned(),
            ],
            "{parameters}"
        );
        assert_eq!(effect.components.len(), 0, "{parameters}");
        assert_eq!(effect.curves, None, "{parameters}");
    }
    // A texture that is no string does not stop the parser; the effect then has no texture.
    let effect = loaded(&with(r#"{"material": "m", "texture": 1}"#));
    assert_eq!(
        findings(effect.diagnostics()),
        [
            format!(
                "Error {brp}/texture particle/texture_not_string: particle texture field isn't a string"
            ),
            format!(
                "Error {brp}/texture particle/texture_not_found: Unable to find a texture when reading in particle effect .json!"
            ),
        ]
    );
    assert_eq!(effect.components.len(), 2);
    assert!(effect.curves.is_some());
    // An empty material is found missing after the components.
    let effect = loaded(&with(r#"{"material": "", "texture": "t"}"#));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Error {brp}/material particle/material_not_found: Unable to find a material when reading in particle effect .json!"
        )]
    );
    // The 1.20.80 reader requires both members and has no such lines.
    let cereal = |parameters: &str| with(parameters).replace("1.10.0", CEREAL);
    assert_eq!(
        refusal(&cereal(r#"{"texture": "t"}"#)),
        format!("Error {brp}/material core/missing_required_field: missing required field")
    );
    assert_eq!(
        refusal(&cereal(r#"{"material": "m"}"#)),
        format!("Error {brp}/texture core/missing_required_field: missing required field")
    );
    assert_eq!(
        found(&cereal(r#"{"material": "", "texture": ""}"#)),
        [] as [&str; 0]
    );
}

#[test]
fn rule_3_a_legacy_effect_needs_a_lifetime_and_a_rate_component() {
    let lifetime = "Error particle_effect/components particle/needs_lifetime: Particle effect needs an emitter lifetime component!";
    let rate = "Error particle_effect/components particle/needs_rate: Particle effect needs an emitter rate component!";
    let with = |version: &str, components: &str| {
        document(version, &format!(r#""components": {{{components}}}"#))
    };
    assert_eq!(found(&with(LEGACY, "")), [lifetime, rate]);
    // Each of the lifetime components, and each of the rate components, is one.
    for name in ["once", "looping", "expression", "events"] {
        let components = format!(r#""minecraft:emitter_lifetime_{name}": {{}}"#);
        assert_eq!(found(&with(LEGACY, &components)), [rate], "{name}");
    }
    for (name, member) in [("instant", "num_particles"), ("manual", "max_particles")] {
        let components = format!(r#""minecraft:emitter_rate_{name}": {{"{member}": 1}}"#);
        assert_eq!(found(&with(LEGACY, &components)), [lifetime], "{name}");
    }
    let steady = r#""minecraft:emitter_rate_steady": {"spawn_rate": 1, "max_particles": 1}"#;
    assert_eq!(found(&with(LEGACY, steady)), [lifetime]);
    // The lines are the legacy parser's.
    assert_eq!(found(&with(CEREAL, "")), [] as [&str; 0]);
}

// ---- rules 4 and 5: unknown components and members -------------------------------------------

#[test]
fn rule_4_a_legacy_component_without_a_name_is_skipped() {
    let effect = loaded(&effect(LEGACY, r#""": {}"#));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Error {COMPONENTS}/ particle/component_without_name: Attempting to extract a particle effect component that has no name"
        )]
    );
    assert_eq!(effect.components.len(), 2);
}

/// Rule 5 at 1.26.60: an unknown member refuses a 1.20.80 document (the strict mask);
/// the legacy parser reads past it with a warning and the game stores nothing of it.
#[test]
fn rule_5_an_unknown_member_refuses_a_cereal_document_and_is_dropped_from_a_legacy_one() {
    let sphere = r#""minecraft:emitter_shape_sphere": {"radius": 1, "radious": 2}"#;
    let line = format!(
        "{COMPONENTS}/minecraft:emitter_shape_sphere/radious core/unrecognized_field: unrecognized field"
    );
    // 1.20.80 and later: the strict mask 0xfffc, an error.
    let refused = rejected(&effect(CEREAL, sphere));
    assert!(
        findings(&refused).contains(&format!("Error {line}")),
        "{:?}",
        findings(&refused)
    );
    // At the top of the payload too.
    let refused = rejected(&effect_with(CEREAL, r#""zz": 1"#));
    assert!(
        findings(&refused).contains(
            &"Error particle_effect/zz core/unrecognized_field: unrecognized field".to_owned()
        ),
        "{:?}",
        findings(&refused)
    );
    // The legacy parser reads past it without a word: an information, and the member
    // is not kept.
    let text = effect(LEGACY, sphere);
    let effect = loaded(&text);
    assert_eq!(findings(effect.diagnostics()), [format!("Info {line}")]);
    assert_eq!(
        component(&effect, "minecraft:emitter_shape_sphere"),
        json(r#"{"radius": 1}"#)
    );
}

/// The cereal slice reads through the strict document input, so
/// a member written `null` is present and fails its type; the legacy parser keeps its own
/// reading (here its Molang reader's line and the field's default).
#[test]
fn a_null_member_refuses_a_cereal_document() {
    let rate = r#""minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": null}"#;
    let refused = rejected(&effect(CEREAL, rate));
    assert!(
        errors(&refused)
            .iter()
            .any(|f| f.code.id() == "core/invalid_type"
                && f.location.json_path.to_string()
                    == format!("{COMPONENTS}/minecraft:emitter_rate_instant/num_particles")),
        "{:?}",
        findings(&refused)
    );
    let refused = rejected(&effect_with(CEREAL, r#""curves": null"#));
    assert!(
        errors(&refused)
            .iter()
            .any(|f| f.code.id() == "core/invalid_type"
                && f.location.json_path.to_string() == "particle_effect/curves"),
        "{:?}",
        findings(&refused)
    );
    assert!(load(&effect(LEGACY, rate), &context()).is_ok());
}

// ---- rules 6, 12 and the Molang reader -------------------------------------------------------

#[test]
fn rule_6_the_legacy_molang_reader_takes_a_number_or_a_string() {
    let sphere = |version: &str, radius: &str| {
        effect(
            version,
            &format!(r#""minecraft:emitter_shape_sphere": {{"radius": {radius}}}"#),
        )
    };
    let at_radius = format!("{COMPONENTS}/minecraft:emitter_shape_sphere/radius");
    let radius = |effect: &ParticleEffect| {
        effect
            .components
            .get::<EmitterShapeSphere>()
            .unwrap()
            .radius
            .clone()
    };
    for bad in ["[1]", "null", "{}", r#"{"expression": "1", "version": 1}"#] {
        let effect = loaded(&sphere(LEGACY, bad));
        assert_eq!(
            findings(effect.diagnostics()),
            [format!(
                "Error {at_radius} particle/not_molang: field is not a number or a Molang expression"
            )],
            "{bad}"
        );
        // The helper's default for the field, not 0.
        assert_eq!(radius(&effect), Some(number(1.0)), "{bad}");
        if bad.contains("expression") {
            // The object form is the 1.20.80 reader's own.
            let cereal = loaded(&sphere(CEREAL, bad));
            assert_eq!(radius(&cereal), Some(object("1", 1)));
        } else {
            let refused = refusal(&sphere(CEREAL, bad));
            assert!(
                refused.starts_with(&format!("Error {at_radius} core/invalid_type")),
                "{bad}"
            );
        }
    }
    // A bool passes the legacy reader as its number, and the 1.20.80 reader refuses it.
    // The effect is written with the number: noted at the member (`prt4_` of
    // `writable_legacy/candidates.rs`).
    let effect = loaded(&sphere(LEGACY, "true"));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Info {at_radius} core/value_rewritten: value held in another form than it is written in"
        )]
    );
    assert_eq!(radius(&effect), Some(number(1.0)));
    assert_eq!(radius(&loaded(&sphere(LEGACY, "false"))), Some(number(0.0)));
    assert!(refusal(&sphere(CEREAL, "true")).contains("core/invalid_type"));
    // A present but unusable `creation_expression` is 1, an absent one 0.
    let init = loaded(&effect_legacy(
        r#""minecraft:emitter_initialization": {"creation_expression": []}"#,
    ));
    let init = init.components.get::<EmitterInitialization>().unwrap();
    assert_eq!(init.creation_expression, Some(number(1.0)));
    assert_eq!(init.per_update_expression, None);
    assert_eq!(init.per_update_expression(), number(0.0));
}

#[test]
fn rule_6_a_legacy_required_molang_member_that_is_missing_is_its_default() {
    for (components, at, default) in [
        (
            r#""minecraft:emitter_rate_steady": {"max_particles": 1}"#,
            "minecraft:emitter_rate_steady/spawn_rate",
            "0",
        ),
        (
            r#""minecraft:emitter_rate_steady": {"spawn_rate": 1}"#,
            "minecraft:emitter_rate_steady/max_particles",
            "0",
        ),
        (
            r#""minecraft:emitter_rate_manual": {}"#,
            "minecraft:emitter_rate_manual/max_particles",
            "0",
        ),
        (
            r#""minecraft:emitter_shape_box": {}"#,
            "minecraft:emitter_shape_box/half_dimensions",
            "[0, 0, 0]",
        ),
        (
            r#""minecraft:particle_appearance_billboard": {}"#,
            "minecraft:particle_appearance_billboard/size",
            "[1, 1]",
        ),
    ] {
        // The box's own line is a content-log error after which the effect
        // loads: a kept error, `particle/texts.rs`.
        if at.ends_with("half_dimensions") {
            assert_eq!(
                findings(loaded(&effect(LEGACY, components)).diagnostics()),
                [format!(
                    "Error {COMPONENTS}/{at} particle/offset_required: offset field required!"
                )],
                "{components}"
            );
            continue;
        }
        let effect = loaded(&effect(LEGACY, components));
        assert_eq!(
            findings(effect.diagnostics()),
            [format!(
                "Error {COMPONENTS}/{at} particle/required_missing: required field does not exist"
            )],
            "{components}"
        );
        let (name, member) = at.split_once('/').unwrap();
        let written = component(&effect, name);
        assert_eq!(
            written.as_object().unwrap().get(member),
            Some(&json(default)),
            "{at}"
        );
    }
}

#[test]
fn rule_12_a_legacy_array_has_its_exact_length_and_usable_elements() {
    let billboard = |size: &str| {
        effect(
            LEGACY,
            &format!(r#""minecraft:particle_appearance_billboard": {{"size": {size}}}"#),
        )
    };
    let at_size = format!("{COMPONENTS}/minecraft:particle_appearance_billboard/size");
    let size = |effect: &ParticleEffect| {
        effect
            .components
            .get::<ParticleAppearanceBillboard>()
            .unwrap()
            .size
            .clone()
    };
    for bad in ["[1]", "[1, 2, 3]", "1", r#""x""#, "{}"] {
        let effect = loaded(&billboard(bad));
        assert_eq!(
            findings(effect.diagnostics()),
            [format!(
                "Error {at_size} particle/array_error: error reading array"
            )],
            "{bad}"
        );
        assert_eq!(size(&effect), [number(1.0), number(1.0)], "{bad}");
    }
    let effect = loaded(&billboard("[2, 3]"));
    assert_eq!(size(&effect), [number(2.0), number(3.0)]);
    let effect = loaded(&billboard("[null, {}]"));
    assert_eq!(
        findings(effect.diagnostics()),
        [
            format!("Error {at_size}/0 particle/array_element_missing: field does not exist"),
            format!(
                "Error {at_size}/1 particle/array_element_not_molang: array field is not a number or a Molang expression"
            ),
        ]
    );
    assert_eq!(size(&effect), [number(1.0), number(1.0)]);
    // The 1.20.80 reader: a bad element is reported at its own index.
    let cereal = effect_cereal_billboard("[1, true]");
    assert_eq!(
        refusal(&cereal),
        format!("Error {at_size}/1 core/invalid_type: wrong JSON type")
    );
}

fn effect_cereal_billboard(size: &str) -> String {
    effect(
        CEREAL,
        &format!(r#""minecraft:particle_appearance_billboard": {{"size": {size}}}"#),
    )
}

/// Ours (the settings precedent): a number beyond `f32` is read as the largest finite float
/// of its sign with a warning, in a Molang member and in a plain float, in both eras; it is
/// never infinity, and the document writes back.
#[test]
fn a_number_beyond_f32_is_the_largest_finite_float_with_a_warning_deviation() {
    for version in [LEGACY, CEREAL] {
        let effect = loaded(&effect(
            version,
            r#""minecraft:emitter_shape_sphere": {"radius": 1e39}, "minecraft:particle_kill_plane": [0, -1e39, 0, 1e39], "minecraft:particle_motion_collision": {"collision_radius": 0.1, "collision_drag": -1e300}"#,
        ));
        let at = |rest: &str| format!("Info {COMPONENTS}/{rest} core/invalid_value: invalid value");
        assert_eq!(
            findings(effect.diagnostics()),
            [
                at("minecraft:emitter_shape_sphere/radius"),
                at("minecraft:particle_kill_plane/1"),
                at("minecraft:particle_kill_plane/3"),
                at("minecraft:particle_motion_collision/collision_drag"),
            ],
            "{version}"
        );
        let sphere = effect.components.get::<EmitterShapeSphere>().unwrap();
        assert_eq!(sphere.radius, Some(Molang::Const(f32::MAX)));
        let plane = effect.components.get::<ParticleKillPlane>().unwrap();
        assert_eq!(plane.0, [0.0, f32::MIN, 0.0, f32::MAX]);
        let collision = effect.components.get::<ParticleMotionCollision>().unwrap();
        assert_eq!(collision.collision_drag, Some(f32::MIN));
        // Written and read again: the same value, without the warning.
        let again =
            loaded(&bedrock_addon_core::document::Content::to_json(effect.value()).unwrap());
        assert_eq!(again.value(), effect.value(), "{version}");
        assert_eq!(findings(again.diagnostics()), [] as [&str; 0], "{version}");
    }
    // A number no double holds refuses the document, as the game's JSON parser does.
    for version in [LEGACY, CEREAL] {
        let refused = rejected(&effect(
            version,
            r#""minecraft:particle_kill_plane": [0, 1e999, 0, 1]"#,
        ));
        assert_eq!(
            texts(&refused),
            [
                "Error  core/document_json: Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
            ],
            "{version}"
        );
    }
    // The largest finite float itself is no finding.
    let exact = effect(
        CEREAL,
        r#""minecraft:emitter_shape_sphere": {"radius": 3.4028235e38}"#,
    );
    assert_eq!(found(&exact), [] as [&str; 0]);
}

/// An integer literal too long for 64 bits is the float it spells, in a plain float member as
/// in a Molang one and as `1e20` is, in both eras: one rule for every number of the family.
#[test]
fn an_integer_literal_beyond_64_bits_is_read_as_the_float_it_is() {
    for (literal, value) in [
        ("100000000000000000000", 1e20_f32),
        ("18446744073709551616", 18_446_744_073_709_551_616.0),
        ("-9223372036854775809", -9_223_372_036_854_775_809.0),
        ("18446744073709551615", 18_446_744_073_709_551_615.0),
        ("-9223372036854775808", -9_223_372_036_854_775_808.0),
        ("1e20", 1e20),
        ("100000000000000000000.0", 1e20),
    ] {
        for version in [LEGACY, CEREAL] {
            let effect = loaded(&effect_with(
                version,
                &format!(
                    r#""curves": {{"variable.c": {{"type": "bezier_chain", "input": {literal}, "nodes": {{"0.0": {{"value": {literal}, "slope": 0}}}}}}}}, "events": {{"e": {{"randomize": [{{"weight": {}}}]}}}}"#,
                    literal.trim_start_matches('-')
                ),
            ));
            assert_eq!(
                findings(effect.diagnostics()),
                [] as [&str; 0],
                "{literal} at {version}"
            );
            let Some(Curve::BezierChain(curve)) = effect.curves.as_ref().unwrap().get("variable.c")
            else {
                panic!("{literal} at {version}: no chain");
            };
            // A Molang member, a plain float in an object, and one read directly.
            assert_eq!(curve.input, number(value), "{literal} at {version}");
            assert_eq!(
                curve.nodes["0.0"].value,
                Some(value),
                "{literal} at {version}"
            );
            let options = effect.events.as_ref().unwrap()["e"]
                .randomize
                .as_ref()
                .unwrap();
            assert_eq!(options[0].weight, value.abs(), "{literal} at {version}");
            // A fixed array of floats and a member of a component.
            let effect = loaded(&super::common::effect(
                version,
                &format!(
                    r#""minecraft:particle_kill_plane": [0, {literal}, 0, 1], "minecraft:particle_motion_collision": {{"collision_radius": 0.1, "collision_drag": {literal}}}"#
                ),
            ));
            assert_eq!(
                findings(effect.diagnostics()),
                [] as [&str; 0],
                "{literal} at {version}"
            );
            let plane = effect.components.get::<ParticleKillPlane>().unwrap();
            assert_eq!(plane.0, [0.0, value, 0.0, 1.0], "{literal} at {version}");
            let collision = effect.components.get::<ParticleMotionCollision>().unwrap();
            assert_eq!(
                collision.collision_drag,
                Some(value),
                "{literal} at {version}"
            );
            // Written and read again: the same value.
            let again =
                loaded(&bedrock_addon_core::document::Content::to_json(effect.value()).unwrap());
            assert_eq!(again.value(), effect.value(), "{literal} at {version}");
        }
    }
    // Beyond `f32` an integer literal is what `1e39` is: the largest finite float, warned of.
    // 128 bits are just past the largest float; a literal longer than that, and one no double
    // holds, are read the same way.
    for (huge, version) in [u128::MAX.to_string(), "9".repeat(40)]
        .iter()
        .flat_map(|huge| [(huge, LEGACY), (huge, CEREAL)])
    {
        let effect = loaded(&super::common::effect(
            version,
            &format!(r#""minecraft:particle_kill_plane": [0, {huge}, 0, 1]"#),
        ));
        assert_eq!(
            findings(effect.diagnostics()),
            [format!(
                "Info {COMPONENTS}/minecraft:particle_kill_plane/1 core/invalid_value: invalid value"
            )],
            "{version}"
        );
        let plane = effect.components.get::<ParticleKillPlane>().unwrap();
        assert_eq!(plane.0, [0.0, f32::MAX, 0.0, 1.0], "{version}");
        // In a member the family does not know, the number is told with the member and not
        // kept; from 1.20.80 the member refuses the document.
        let unknown_text = effect_with(version, r#""x": 100000000000000000000"#);
        if version == CEREAL {
            assert!(load(&unknown_text, &context()).is_err());
            continue;
        }
        let unknown = loaded(&unknown_text);
        assert_eq!(
            findings(unknown.diagnostics()),
            ["Info particle_effect/x core/unrecognized_field: unrecognized field"]
        );
        assert_eq!(
            super::common::at(&written(&unknown), "particle_effect/x"),
            None
        );
    }
}

/// Only a number token is a number: a run of digits as long as a wide integer in a string, a
/// key or a comment stays what it is, beside a literal that is read as its float,
/// as the crate's JSON reader reads it.
#[test]
fn a_long_run_of_digits_in_a_string_a_key_or_a_comment_is_no_number() {
    const DIGITS: &str = "100000000000000000000";
    // Unknown members: told and not kept in the legacy era; from 1.20.80 they refuse
    // the document. A run of digits in a string, a key or a comment is no number: the
    // only findings are the three members, none a clamped number.
    for version in [LEGACY, CEREAL] {
        let text = effect_with(
            version,
            &format!(
                "\"{DIGITS}\": \"{DIGITS}\", /* {DIGITS} */ \"escaped\": \"a\\\"{DIGITS}\", // {DIGITS}\n \"x\": {DIGITS}"
            ),
        );
        if version == CEREAL {
            assert!(load(&text, &context()).is_err());
            continue;
        }
        let effect = loaded(&text);
        assert_eq!(
            findings(effect.diagnostics()),
            [
                format!(
                    "Info particle_effect/{DIGITS} core/unrecognized_field: unrecognized field"
                ),
                "Info particle_effect/escaped core/unrecognized_field: unrecognized field"
                    .to_owned(),
                "Info particle_effect/x core/unrecognized_field: unrecognized field".to_owned(),
            ],
            "{version}"
        );
        assert!(effect.extra.is_empty());
    }
}

/// Content after the root value refuses the document in both eras: the particle effect is a
/// `MinecraftDocumentInput`, whose JSON parse stops at it.
#[test]
fn content_after_the_root_value_refuses_the_document() {
    for version in [LEGACY, CEREAL] {
        let plain = effect(version, "");
        for tail in [
            " garbage",
            "\n{\"format_version\": \"1.0.0\"}",
            " ] 100000000000000000000",
        ] {
            let refused = rejected(&format!("{plain}{tail}"));
            assert_eq!(
                texts(&refused),
                [
                    "Error  core/document_json: Failed to parse the Minecraft Document JSON: The document root must not be followed by other values."
                ],
                "{version}{tail}"
            );
            let span = refused.iter().next().unwrap().location.span;
            let span = span.expect("the finding is on the content");
            assert_eq!(span.offset as usize, plain.len() + 1, "{version}{tail}");
            assert_eq!(span.len as usize, tail.trim().len(), "{version}{tail}");
        }
        // White space and a comment after the root are no content.
        let effect = loaded(&format!("{plain} \n// the end\n"));
        assert_eq!(findings(effect.diagnostics()), [] as [&str; 0], "{version}");
    }
}

/// A run of digits too long for 64 bits that is no number token stays what it is: in a
/// string value, a key, a comment and a string of a member the family does not know, in both
/// eras, as read and as written back.
#[test]
fn a_long_run_of_digits_in_a_string_a_key_or_a_comment_is_not_a_number() {
    const DIGITS: &str = "100000000000000000000";
    for version in [LEGACY, CEREAL] {
        // Two unknown members in the legacy era: told and not kept; at 1.20.80 they
        // would refuse the document, so it has none.
        let unknown = if version == LEGACY {
            format!(
                r#", "{DIGITS}": "{DIGITS}", "unknown": {{"text": "a \" {DIGITS}", "{DIGITS}": [{DIGITS}]}}"#
            )
        } else {
            String::new()
        };
        let text = effect_with(
            version,
            &format!(
                r#"// {DIGITS}
                "curves": {{"{DIGITS}": {{"type": "linear", "input": "{DIGITS}", "nodes": [0, 1]}}}},
                /* {DIGITS} */ "events": {{"{DIGITS}": {{"log": "{DIGITS}", "sound_effect": {{"event_name": "-{DIGITS}"}}}}}}{unknown}"#
            ),
        );
        let effect = loaded(&text);
        let Some(Curve::Simple(curve)) = effect.curves.as_ref().unwrap().get(DIGITS) else {
            panic!("{version}: no curve named by the digits");
        };
        // A Molang string of the digits, not a constant.
        assert_eq!(
            curve
                .input
                .source()
                .map(bedrock_addon_core::molang::MolangSource::as_str),
            Some(DIGITS),
            "{version}"
        );
        let event = &effect.events.as_ref().unwrap()[DIGITS];
        assert_eq!(event.log(), DIGITS, "{version}");
        let sound = event.sound_effect.as_ref().unwrap();
        assert_eq!(sound.event_name, format!("-{DIGITS}"), "{version}");
        // Written back: the strings are strings; the unknown members are gone.
        let written = super::common::written(&effect);
        let at = |pointer: &str| super::common::at(&written, &format!("particle_effect/{pointer}"));
        assert_eq!(
            at(&format!("events/{DIGITS}/log"))
                .and_then(|value| value.as_string())
                .map(|text| text.as_str().to_owned())
                .as_deref(),
            Some(DIGITS),
            "{version}"
        );
        assert_eq!(at(DIGITS), None, "{version}");
        assert_eq!(at("unknown"), None, "{version}");
    }
}

// ---- rule 7 is fixture p11; rule 8: the collision radius -------------------------------------

#[test]
fn rule_8_the_collision_radius_is_0_to_one_half() {
    let collision = |version: &str, radius: &str| {
        effect(
            version,
            &format!(r#""minecraft:particle_motion_collision": {{"collision_radius": {radius}}}"#),
        )
    };
    let at_radius = format!("{COMPONENTS}/minecraft:particle_motion_collision/collision_radius");
    let radius = |effect: &ParticleEffect| {
        effect
            .components
            .get::<ParticleMotionCollision>()
            .unwrap()
            .collision_radius
    };
    for version in [LEGACY, CEREAL] {
        for inside in ["0", "0.5", "0.25"] {
            assert_eq!(
                found(&collision(version, inside)),
                [] as [&str; 0],
                "{version} {inside}"
            );
        }
    }
    for (outside, clamped, id, line) in [
        (
            "-0.001",
            0.0,
            "radius_too_small",
            "radius is too small, clamping to 0",
        ),
        (
            "-7",
            0.0,
            "radius_too_small",
            "radius is too small, clamping to 0",
        ),
        (
            "0.501",
            0.5,
            "radius_too_large",
            "radius is too large, clamping to 0.5",
        ),
        (
            "9",
            0.5,
            "radius_too_large",
            "radius is too large, clamping to 0.5",
        ),
    ] {
        let effect = loaded(&collision(LEGACY, outside));
        assert_eq!(
            findings(effect.diagnostics()),
            [format!("Error {at_radius} particle/{id}: {line}")]
        );
        assert_eq!(radius(&effect), clamped, "{outside}");
        assert_eq!(
            refusal(&collision(CEREAL, outside)),
            format!("Error {at_radius} core/invalid_value: invalid value"),
            "{outside}"
        );
    }
    // The legacy parser's lines for a radius that is missing or no number.
    let effect = loaded(&effect(
        LEGACY,
        r#""minecraft:particle_motion_collision": {}"#,
    ));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Error {at_radius} particle/required_missing: required field does not exist"
        )]
    );
    assert_eq!(radius(&effect), 0.0);
    let effect = loaded(&collision(LEGACY, r#""wide""#));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Error {at_radius} particle/radius_not_float: value is not a float"
        )]
    );
    assert_eq!(
        refusal(&effect_cereal_collision("{}")),
        format!("Error {at_radius} core/missing_required_field: missing required field")
    );
}

fn effect_cereal_collision(value: &str) -> String {
    effect(
        CEREAL,
        &format!(r#""minecraft:particle_motion_collision": {value}"#),
    )
}

#[test]
fn the_legacy_collision_reader_logs_its_lines_and_keeps_the_defaults() {
    let at = format!("{COMPONENTS}/minecraft:particle_motion_collision");
    let effect = loaded(&effect(
        LEGACY,
        r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "collision_drag": "x", "coefficient_of_restitution": [], "expire_on_contact": 1, "events": 3}"#,
    ));
    assert_eq!(
        findings(effect.diagnostics()),
        [
            format!(
                "Error {at}/collision_drag particle/drag_not_float: collision_drag is not a float"
            ),
            format!(
                "Error {at}/coefficient_of_restitution particle/restitution_not_float: coefficient_of_restitution value is not a float"
            ),
            format!(
                "Error {at}/expire_on_contact particle/expire_not_bool: expire_on_contact needs to be true/false"
            ),
            format!(
                "Error {at}/events particle/events_not_pairs: is not an array of min-speed/event pairs"
            ),
        ]
    );
    let stored = effect.components.get::<ParticleMotionCollision>().unwrap();
    assert_eq!(
        (
            stored.collision_drag(),
            stored.coefficient_of_restitution(),
            stored.expire_on_contact()
        ),
        (0.0, 0.0, false)
    );
    assert_eq!(stored.events, None);
    let effect = loaded(&super::common::effect(
        LEGACY,
        r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [1, {"min_speed": 3}, {"event": "kept"}, {"event": "floor", "min_speed": 2}, {"event": "raised", "min_speed": 1.999}, {"event": "text", "min_speed": "x"}]}"#,
    ));
    assert_eq!(
        findings(effect.diagnostics()),
        [
            format!(
                "Error {at}/events/0 particle/event_not_pair: event is not a min-speed/event pair"
            ),
            format!("Error {at}/events/1/event particle/no_event_field: no 'event' field!"),
            format!("Info {at}/events/4/min_speed core/invalid_value: invalid value"),
            format!("Info {at}/events/5/min_speed core/invalid_value: invalid value"),
            format!("Info {at}/events/5/min_speed core/invalid_value: invalid value"),
            // The entry without an `event` goes whole.
            format!("Info {at}/events/1 core/dropped_field: field dropped"),
        ]
    );
    let stored = effect.components.get::<ParticleMotionCollision>().unwrap();
    let speeds: Vec<_> = stored
        .events
        .as_ref()
        .unwrap()
        .iter()
        .map(|e| (e.event.as_str(), e.min_speed))
        .collect();
    // Exactly 2 stays as written; below it, and a value that is no number, become 2.
    assert_eq!(
        speeds,
        [
            ("kept", None),
            ("floor", Some(2.0)),
            ("raised", Some(2.0)),
            ("text", Some(2.0))
        ]
    );
}

// ---- rule 10: the members the 1.20.80 reader requires ----------------------------------------

/// Every component member the 1.20.80 reader requires is refused when it is missing.
fn required_components_are_refused() {
    let billboard = |rest: &str| {
        format!(r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], {rest}}}"#)
    };
    let cases: Vec<(String, String)> = vec![
        (r#""minecraft:emitter_rate_instant": {}"#.into(), "components/minecraft:emitter_rate_instant/num_particles".into()),
        (r#""minecraft:emitter_rate_steady": {"max_particles": 1}"#.into(), "components/minecraft:emitter_rate_steady/spawn_rate".into()),
        (r#""minecraft:emitter_rate_steady": {"spawn_rate": 1}"#.into(), "components/minecraft:emitter_rate_steady/max_particles".into()),
        (r#""minecraft:emitter_rate_manual": {}"#.into(), "components/minecraft:emitter_rate_manual/max_particles".into()),
        (r#""minecraft:emitter_shape_box": {}"#.into(), "components/minecraft:emitter_shape_box/half_dimensions".into()),
        (r#""minecraft:particle_appearance_billboard": {}"#.into(), "components/minecraft:particle_appearance_billboard/size".into()),
        (billboard(r#""direction": {}"#), "components/minecraft:particle_appearance_billboard/direction/mode".into()),
        (billboard(r#""uv": {"flipbook": {"max_frame": 1}}"#), "components/minecraft:particle_appearance_billboard/uv/flipbook/base_UV".into()),
        (billboard(r#""uv": {"flipbook": {"base_UV": [0, 0]}}"#), "components/minecraft:particle_appearance_billboard/uv/flipbook/max_frame".into()),
        (r#""minecraft:particle_appearance_tinting": {}"#.into(), "components/minecraft:particle_appearance_tinting/color".into()),
        (r#""minecraft:particle_appearance_tinting": {"color": {"interpolant": 0}}"#.into(), "components/minecraft:particle_appearance_tinting/color/gradient".into()),
        (r#""minecraft:particle_appearance_tinting": {"color": {"gradient": []}}"#.into(), "components/minecraft:particle_appearance_tinting/color/interpolant".into()),
        (r#""minecraft:particle_motion_collision": {}"#.into(), "components/minecraft:particle_motion_collision/collision_radius".into()),
        (r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [{}]}"#.into(), "components/minecraft:particle_motion_collision/events/0/event".into()),
        (r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": {"min_speed": 1}}"#.into(), "components/minecraft:particle_motion_collision/events/event".into()),
        (r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [{"effects": "e"}]}"#.into(), "components/minecraft:emitter_lifetime_events/looping_travel_distance_events/0/distance".into()),
        (r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [{"distance": 1}]}"#.into(), "components/minecraft:emitter_lifetime_events/looping_travel_distance_events/0/effects".into()),
    ];
    for (components, at) in cases {
        assert_eq!(
            refusal(&effect(CEREAL, &components)),
            format!(
                "Error particle_effect/{at} core/missing_required_field: missing required field"
            ),
            "{components}"
        );
    }
}

/// Every member of a curve or an event the 1.20.80 reader requires is refused when it is
/// missing.
fn required_members_of_curves_and_events_are_refused() {
    let members: Vec<(&str, &str)> = vec![
        (
            r#""curves": {"variable.c": {"input": 0, "nodes": [0]}}"#,
            "curves/variable.c/type",
        ),
        (
            r#""curves": {"variable.c": {"type": "linear", "input": 0}}"#,
            "curves/variable.c/nodes",
        ),
        (
            r#""curves": {"variable.c": {"type": "linear", "nodes": [0]}}"#,
            "curves/variable.c/input",
        ),
        (
            r#""curves": {"variable.c": {"type": "bezier", "nodes": [0, 0, 0, 0]}}"#,
            "curves/variable.c/input",
        ),
        (
            r#""curves": {"variable.c": {"type": "catmull_rom", "input": 0}}"#,
            "curves/variable.c/nodes",
        ),
        (
            r#""curves": {"variable.c": {"type": "bezier_chain", "input": 0}}"#,
            "curves/variable.c/nodes",
        ),
        (
            r#""curves": {"variable.c": {"type": "bezier_chain", "nodes": {}}}"#,
            "curves/variable.c/input",
        ),
        (
            r#""events": {"e": {"randomize": [{}]}}"#,
            "events/e/randomize/0/weight",
        ),
        (
            r#""events": {"e": {"sequence": [{"randomize": [{"weight": 1}, {}]}]}}"#,
            "events/e/sequence/0/randomize/1/weight",
        ),
        (
            r#""events": {"e": {"particle_effect": {"type": "emitter"}}}"#,
            "events/e/particle_effect/effect",
        ),
        (
            r#""events": {"e": {"particle_effect": {"effect": "x"}}}"#,
            "events/e/particle_effect/type",
        ),
        (
            r#""events": {"e": {"sound_effect": {}}}"#,
            "events/e/sound_effect/event_name",
        ),
    ];
    for (members, at) in members {
        assert_eq!(
            refusal(&effect_with(CEREAL, members)),
            format!(
                "Error particle_effect/{at} core/missing_required_field: missing required field"
            ),
            "{members}"
        );
    }
}

#[test]
fn rule_10_every_required_member_of_the_1_20_80_reader() {
    required_components_are_refused();
    required_members_of_curves_and_events_are_refused();
    // The payload's own two.
    assert_eq!(
        refusal(r#"{"format_version": "1.26.10", "particle_effect": {"components": {}}}"#),
        "Error particle_effect/description core/missing_required_field: missing required field"
    );
    assert_eq!(
        refusal(&format!(
            r#"{{"format_version": "1.26.10", "particle_effect": {{{DESCRIPTION}}}}}"#
        )),
        "Error particle_effect/components core/missing_required_field: missing required field"
    );
    // With every required member, each of these loads.
    assert_eq!(
        found(&format!(
            r#"{{"format_version": "1.26.10", "particle_effect": {{{DESCRIPTION}, "components": {{}}}}}}"#
        )),
        [] as [&str; 0]
    );
}

/// A component member of the wrong type or value is refused at its path.
fn components_of_the_wrong_type_are_refused_at_their_path() {
    refused_at_their_paths(&[
        (
            r#""minecraft:emitter_local_space": {"position": 1}"#,
            "minecraft:emitter_local_space/position",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_local_space": 1"#,
            "minecraft:emitter_local_space",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_appearance_lighting": []"#,
            "minecraft:particle_appearance_lighting",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_lifetime_once": []"#,
            "minecraft:emitter_lifetime_once",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_shape_sphere": {"direction": "up"}"#,
            "minecraft:emitter_shape_sphere/direction",
            "core/invalid_value: invalid value",
        ),
        (
            r#""minecraft:emitter_shape_sphere": {"direction": 1}"#,
            "minecraft:emitter_shape_sphere/direction",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_shape_disc": {"plane_normal": "w"}"#,
            "minecraft:emitter_shape_disc/plane_normal",
            "core/invalid_value: invalid value",
        ),
        (
            r#""minecraft:emitter_shape_sphere": {"surface_only": "yes"}"#,
            "minecraft:emitter_shape_sphere/surface_only",
            "core/invalid_type: wrong JSON type",
        ),
    ]);
}

/// The same, for the members of the particle components.
fn particle_components_of_the_wrong_type_are_refused_at_their_path() {
    refused_at_their_paths(&[
        (
            r#""minecraft:particle_kill_plane": [0, 1, 0]"#,
            "minecraft:particle_kill_plane",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_kill_plane": [0, 1, 0, "x"]"#,
            "minecraft:particle_kill_plane/3",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_expire_if_in_blocks": "minecraft:air""#,
            "minecraft:particle_expire_if_in_blocks",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_expire_if_not_in_blocks": ["a", 1]"#,
            "minecraft:particle_expire_if_not_in_blocks/1",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_initial_speed": [1, 2]"#,
            "minecraft:particle_initial_speed",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_initial_speed": true"#,
            "minecraft:particle_initial_speed",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_lifetime_events": {"creation_event": 1}"#,
            "minecraft:emitter_lifetime_events/creation_event",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_lifetime_events": {"creation_event": ["a", 1]}"#,
            "minecraft:emitter_lifetime_events/creation_event/1",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_lifetime_events": {"timeline": []}"#,
            "minecraft:emitter_lifetime_events/timeline",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [{"distance": -0.5, "effects": "e"}]}"#,
            "minecraft:emitter_lifetime_events/looping_travel_distance_events/0/distance",
            "core/invalid_value: invalid value",
        ),
        (
            r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "direction": {"mode": "sideways"}}"#,
            "minecraft:particle_appearance_billboard/direction/mode",
            "core/invalid_value: invalid value",
        ),
        (
            r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "uv": {"texture_width": 16.5}}"#,
            "minecraft:particle_appearance_billboard/uv/texture_width",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "uv": {"flipbook": {"base_UV": [0, 0], "max_frame": 1, "size_UV": [1]}}}"#,
            "minecraft:particle_appearance_billboard/uv/flipbook/size_UV",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_appearance_tinting": {"color": [1, 1]}"#,
            "minecraft:particle_appearance_tinting/color",
            "core/invalid_type: wrong JSON type",
        ),
        (
            r#""minecraft:particle_appearance_tinting": {"color": 1}"#,
            "minecraft:particle_appearance_tinting/color",
            "core/invalid_type: wrong JSON type",
        ),
    ]);
}

/// Each `(components, path, finding)` is refused with the finding at the path.
fn refused_at_their_paths(cases: &[(&str, &str, &str)]) {
    for &(components, at, code) in cases {
        assert_eq!(
            refusal(&effect(CEREAL, components)),
            format!("Error {COMPONENTS}/{at} {code}"),
            "{components}"
        );
    }
}

#[test]
fn the_1_20_80_reader_refuses_a_value_of_the_wrong_type_at_its_path() {
    components_of_the_wrong_type_are_refused_at_their_path();
    particle_components_of_the_wrong_type_are_refused_at_their_path();
    // A distance of exactly 0 is inside the constraint.
    let zero = r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [{"distance": 0, "effects": "e"}]}"#;
    assert_eq!(found(&effect(CEREAL, zero)), [] as [&str; 0]);
    for (members, at) in [
        (r#""curves": []"#, "curves"),
        (r#""events": []"#, "events"),
        (r#""events": {"e": 1}"#, "events/e"),
        (r#""events": {"e": {"sequence": {}}}"#, "events/e/sequence"),
        (r#""events": {"e": {"log": 1}}"#, "events/e/log"),
        (
            r#""events": {"e": {"expression": true}}"#,
            "events/e/expression",
        ),
        (r#""curves": {"variable.c": 1}"#, "curves/variable.c"),
        (
            r#""curves": {"variable.c": {"type": 1, "input": 0, "nodes": []}}"#,
            "curves/variable.c/type",
        ),
        (
            r#""curves": {"variable.c": {"type": "linear", "input": 0, "nodes": {}}}"#,
            "curves/variable.c/nodes",
        ),
        (
            r#""curves": {"variable.c": {"type": "bezier_chain", "input": 0, "nodes": []}}"#,
            "curves/variable.c/nodes",
        ),
        (
            r#""curves": {"variable.c": {"type": "bezier_chain", "input": 0, "nodes": {"0": 1}}}"#,
            "curves/variable.c/nodes/0",
        ),
    ] {
        assert_eq!(
            refusal(&effect_with(CEREAL, members)),
            format!("Error particle_effect/{at} core/invalid_type: wrong JSON type"),
            "{members}"
        );
    }
    assert_eq!(
        refusal(&effect_with(
            CEREAL,
            r#""events": {"e": {"particle_effect": {"effect": "x", "type": "none"}}}"#
        )),
        "Error particle_effect/events/e/particle_effect/type core/invalid_value: invalid value"
    );
}

#[test]
fn the_texture_is_at_most_256_characters_from_1_20_80() {
    let with = |version: &str, length: usize| {
        effect(version, "").replace("textures/particle/particles", &"t".repeat(length))
    };
    assert_eq!(found(&with(CEREAL, 256)), [] as [&str; 0]);
    assert_eq!(
        refusal(&with(CEREAL, 257)),
        "Error particle_effect/description/basic_render_parameters/texture core/invalid_value: invalid value"
    );
    assert_eq!(found(&with(LEGACY, 257)), [] as [&str; 0]);
}

// ---- rule 13 and the legacy readers' lines ---------------------------------------------------

#[test]
fn rule_13_a_custom_direction_mode_needs_its_direction() {
    let at = format!("{COMPONENTS}/minecraft:particle_appearance_billboard/direction");
    for version in [LEGACY, CEREAL] {
        let custom = effect(
            version,
            r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "direction": {"mode": "custom"}}"#,
        );
        // The legacy billboard adds its own line when `_parseDirection` fails,
        // a content-log error after which the effect loads: a kept error.
        let first = format!(
            "Error {at} particle/direction_custom: The 'direction' sub section is not defined as a proper json object!"
        );
        if version == LEGACY {
            assert_eq!(
                findings(loaded(&custom).diagnostics()),
                [
                    first,
                    format!(
                        "Error {at} particle/direction_not_properly_defined: The direction sub section is not properly defined!"
                    )
                ]
            );
        } else {
            assert_eq!(found(&custom), [first], "{version}");
        }
        let derived = effect(
            version,
            r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "direction": {"mode": "derive_from_velocity"}}"#,
        );
        assert_eq!(found(&derived), [] as [&str; 0], "{version}");
    }
    // The legacy parser's own lines for the section.
    let legacy = |direction: &str| {
        effect(
            LEGACY,
            &format!(
                r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "direction": {direction}, "uv": 1}}"#
            ),
        )
    };
    // The failed direction ends the parse before `uv`: no "bad uv data!"; the
    // effect loads (a kept error).
    assert_eq!(
        findings(loaded(&legacy("1")).diagnostics()),
        [
            format!(
                "Error {at} particle/direction_not_object: The direction subsection is not defined as a proper json object!"
            ),
            format!(
                "Error {at} particle/direction_not_properly_defined: The direction sub section is not properly defined!"
            ),
            format!(
                "Info {COMPONENTS}/minecraft:particle_appearance_billboard/uv core/invalid_value: invalid value"
            ),
        ]
    );
    assert_eq!(
        found(&legacy(r#"{"mode": "sideways"}"#))[0],
        format!(
            "Error {at}/mode particle/direction_mode: The 'direction' sub section is not using a valid mode!"
        )
    );
    let uv = effect(
        LEGACY,
        r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "uv": {"texturewidth": 16, "textureheight": 16, "flipbook": 1}}"#,
    );
    let at_uv = format!("{COMPONENTS}/minecraft:particle_appearance_billboard/uv");
    let lines = found(&uv);
    for line in [
        format!("Error {at_uv}/flipbook particle/bad_flipbook: bad data!"),
        format!(
            "Error {at_uv}/texturewidth particle/texture_width_name: texture width field needs to be named 'texture_width'"
        ),
        format!(
            "Error {at_uv}/textureheight particle/texture_height_name: texture height field needs to be named 'texture_height'"
        ),
    ] {
        assert!(lines.contains(&line), "{line} in {lines:#?}");
    }
    // The legacy parser has no "missing uv" lines.
    let bare = effect(
        LEGACY,
        r#""minecraft:particle_appearance_billboard": {"size": [1, 1], "uv": {}}"#,
    );
    assert_eq!(found(&bare), [] as [&str; 0]);
}

#[test]
fn rule_13_curves_of_an_unknown_type_are_left_out_in_both_eras() {
    for version in [LEGACY, CEREAL] {
        let effect = loaded(&effect_with(
            version,
            r#""curves": {"variable.a": {"type": "spiral", "input": 0, "nodes": [0]}, "variable.b": {"type": "linear", "input": 0, "nodes": [0]}}"#,
        ));
        assert_eq!(
            findings(effect.diagnostics()),
            [
                "Error particle_effect/curves/variable.a/type particle/curve_type: Particle effect curve has invalid type",
                // The curve goes with its `input` and `nodes`.
                "Info particle_effect/curves/variable.a core/dropped_field: field dropped",
            ],
            "{version}"
        );
        let names: Vec<_> = effect.curves.as_ref().unwrap().keys().collect();
        assert_eq!(names, ["variable.b"], "{version}");
    }
    // The match is case-sensitive.
    let upper = effect_with(
        CEREAL,
        r#""curves": {"variable.a": {"type": "Linear", "input": 0, "nodes": [0]}}"#,
    );
    // The line at `type` and the note at the curve.
    assert_eq!(found(&upper).len(), 2);
}

#[test]
fn rule_13_a_chain_node_needs_a_value_and_a_slope_on_both_sides() {
    let node = |members: &str| {
        effect_with(
            CEREAL,
            &format!(
                r#""curves": {{"variable.c": {{"type": "bezier_chain", "input": 0, "nodes": {{"0.5": {{{members}}}}}}}}}"#
            ),
        )
    };
    let at = "particle_effect/curves/variable.c/nodes/0.5";
    let value = format!(
        "Error {at} particle/chain_value: Particle curve bezier chain node is missing required value field"
    );
    let slope = format!(
        "Error {at} particle/chain_slope: Particle curve bezier chain node is missing required slope field"
    );
    for (members, expected) in [
        (r#""value": 1, "slope": 0"#, vec![]),
        (
            r#""left_value": 1, "right_value": 2, "left_slope": 0, "right_slope": 1"#,
            vec![],
        ),
        (r#""value": 1, "left_slope": 0, "right_slope": 1"#, vec![]),
        (r#""left_value": 1, "slope": 0"#, vec![&value]),
        (r#""right_value": 1, "slope": 0"#, vec![&value]),
        (r#""value": 1, "left_slope": 0"#, vec![&slope]),
        (r#""value": 1, "right_slope": 0"#, vec![&slope]),
        ("", vec![&value, &slope]),
    ] {
        assert_eq!(
            found(&node(members)).iter().collect::<Vec<_>>(),
            expected,
            "{members}"
        );
    }
}

#[test]
fn the_legacy_curve_reader_logs_its_lines_and_leaves_the_curve_out() {
    let curves = |entries: &str| effect_with(LEGACY, &format!(r#""curves": {entries}"#));
    let at = "particle_effect/curves";
    assert_eq!(
        found(&curves("1")),
        [format!(
            "Error {at} particle/no_curves: No curves found in curves section!"
        )]
    );
    assert_eq!(loaded(&curves("1")).curves, None);
    for (entries, expected) in [
        (
            r#"{"": {"type": "linear", "input": 0, "nodes": [0]}}"#,
            format!(
                "Error {at}/ particle/curve_without_name: Attempting to extract a curve with an empty name!"
            ),
        ),
        (
            r#"{"variable.c": 1}"#,
            format!(
                "Error {at}/variable.c particle/curve_no_data: Particle effect curve has no valid data!"
            ),
        ),
        (
            r#"{"variable.c": {"input": 0, "nodes": [0]}}"#,
            format!(
                "Error {at}/variable.c/type particle/curve_type: Particle effect curve has invalid type"
            ),
        ),
        (
            r#"{"variable.c": {"type": 7, "input": 0, "nodes": [0]}}"#,
            format!(
                "Error {at}/variable.c/type particle/curve_type: Particle effect curve has invalid type"
            ),
        ),
        (
            r#"{"variable.c": {"type": "linear", "input": 0, "nodes": 1}}"#,
            format!(
                "Error {at}/variable.c/nodes particle/curve_nodes: Particle curve declared with invalid nodes array"
            ),
        ),
        (
            r#"{"variable.c": {"type": "linear", "input": 0}}"#,
            format!(
                "Error {at}/variable.c/nodes particle/curve_nodes: Particle curve declared with invalid nodes array"
            ),
        ),
        (
            r#"{"variable.c": {"type": "bezier_chain", "input": 0, "nodes": []}}"#,
            format!(
                "Error {at}/variable.c/nodes particle/curve_node_data: Particle curve declared with invalid node data"
            ),
        ),
    ] {
        let effect = loaded(&curves(entries));
        // The parser's line, and the note of a curve that is an object and goes whole.
        let mut lines = vec![expected];
        if entries.contains(r#""variable.c": {"#) {
            lines.push(format!(
                "Info {at}/variable.c core/dropped_field: field dropped"
            ));
        }
        assert_eq!(findings(effect.diagnostics()), lines, "{entries}");
        assert_eq!(
            effect.curves.as_ref().map(indexmap::IndexMap::len),
            Some(0),
            "{entries}"
        );
    }
    // A chain node that is no object is left out of its curve.
    let effect = loaded(&curves(
        r#"{"variable.c": {"type": "bezier_chain", "input": 0, "nodes": {"0": 1, "1": {"value": 0, "slope": 0}}}}"#,
    ));
    assert_eq!(
        findings(effect.diagnostics()),
        [format!(
            "Error {at}/variable.c/nodes/0 particle/curve_node_data: Particle curve declared with invalid node data"
        )]
    );
}

/// The legacy event sections that log a line, each with the line.
fn legacy_event_cases(at: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            r#"{"": {}}"#,
            format!(
                "{at}/ particle/event_without_name: Attempting to extract an event with an empty name!"
            ),
        ),
        (
            r#"{"e": 1}"#,
            format!("{at}/e particle/event_no_data: Particle effect event has no valid data!"),
        ),
        (
            r#"{"e": {"sequence": 1}}"#,
            format!("{at}/e/sequence particle/sequence: Error reading sequence field!"),
        ),
        (
            r#"{"e": {"randomize": 1}}"#,
            format!("{at}/e/randomize particle/random: Error reading random field!"),
        ),
        (
            r#"{"e": {"randomize": [{"weight": "x"}]}}"#,
            format!(
                "{at}/e/randomize/0/weight particle/weight: Error reading weight field!  Ignoring random option"
            ),
        ),
        (
            r#"{"e": {"randomize": [{"weight": -0.5}]}}"#,
            format!(
                "{at}/e/randomize/0/weight particle/weight_negative: Weight field must be zero or greater.  Ignoring random option"
            ),
        ),
        (
            r#"{"e": {"particle_effect": 1}}"#,
            format!(
                "{at}/e/particle_effect particle/effect_section: particle_effect section is invalid"
            ),
        ),
        (
            r#"{"e": {"particle_effect": {"type": "emitter"}}}"#,
            format!("{at}/e/particle_effect/effect particle/effect_missing: Effect field missing!"),
        ),
        (
            r#"{"e": {"particle_effect": {"effect": "x"}}}"#,
            format!("{at}/e/particle_effect/type particle/type_missing: Type field missing!"),
        ),
        (
            r#"{"e": {"particle_effect": {"effect": "x", "type": "none"}}}"#,
            format!("{at}/e/particle_effect/type particle/type_missing: Type field missing!"),
        ),
        (
            r#"{"e": {"sound_effect": 1}}"#,
            format!(
                "{at}/e/sound_effect particle/sound_section: sound_effect section should be an object"
            ),
        ),
        (
            r#"{"e": {"sound_effect": {"event_name": 1}}}"#,
            format!(
                "{at}/e/sound_effect/event_name particle/sound_name: sound_effect.event_name should be a string"
            ),
        ),
    ]
}

#[test]
fn the_legacy_event_reader_logs_its_lines_and_leaves_the_part_out() {
    let events = |entries: &str| effect_with(LEGACY, &format!(r#""events": {entries}"#));
    let at = "particle_effect/events";
    assert_eq!(
        found(&events("[]")),
        [format!(
            "Error {at} particle/no_events: No events found in events section!"
        )]
    );
    for (entries, expected) in legacy_event_cases(at) {
        let effect = loaded(&events(entries));
        // The parser's line, and the note of a section that is an object and goes whole.
        let mut lines = vec![format!("Error {expected}")];
        for section in ["particle_effect", "sound_effect"] {
            if entries.contains(&format!(r#""{section}": {{"#)) {
                lines.push(format!(
                    "Info {at}/e/{section} core/dropped_field: field dropped"
                ));
            }
        }
        // A random option that is skipped for its weight.
        if entries.contains(r#""randomize": [{"#) {
            lines.push(format!(
                "Info {at}/e/randomize/0 core/dropped_field: field dropped"
            ));
        }
        assert_eq!(findings(effect.diagnostics()), lines, "{entries}");
        let node = effect.events.as_ref().unwrap().get("e");
        assert!(
            node.is_none_or(|node| node.particle_effect.is_none() && node.sound_effect.is_none()),
            "{entries}"
        );
        assert!(
            node.is_none_or(|node| node.randomize.as_ref().is_none_or(Vec::is_empty)),
            "{entries}"
        );
    }
    // A weight of exactly 0 is an option; from 1.20.80 a negative one is too.
    let zero = loaded(&events(
        r#"{"e": {"randomize": [{"weight": 0, "log": "a"}, {"weight": 2, "log": "b"}]}}"#,
    ));
    assert_eq!(findings(zero.diagnostics()), [] as [&str; 0]);
    assert_eq!(
        zero.events.as_ref().unwrap()["e"]
            .randomize
            .as_ref()
            .unwrap()
            .len(),
        2
    );
    let negative = effect_with(
        CEREAL,
        r#""events": {"e": {"randomize": [{"weight": -0.5}]}}"#,
    );
    assert_eq!(found(&negative), [] as [&str; 0]);
    // The legacy parser reads an event's Molang only from a string; a number is nothing,
    // without a line of the game's, and the upgrade gives the node its 0. Noted where it
    // is.
    let silent = loaded(&events(
        r#"{"e": {"expression": 5, "particle_effect": {"effect": "x", "type": "emitter", "pre_effect_expression": 7}}}"#,
    ));
    assert_eq!(
        findings(silent.diagnostics()),
        [
            format!("Info {at}/e/expression core/invalid_type: wrong JSON type"),
            format!(
                "Info {at}/e/particle_effect/pre_effect_expression core/invalid_type: wrong JSON type"
            ),
        ]
    );
    let node = &silent.events.as_ref().unwrap()["e"];
    assert_eq!(node.expression, Some(number(0.0)));
    assert_eq!(
        node.particle_effect.as_ref().unwrap().pre_effect_expression,
        Some(number(0.0))
    );
    let text = loaded(&events(r#"{"e": {"expression": "variable.a = 1;"}}"#));
    assert_eq!(
        text.events.as_ref().unwrap()["e"].expression,
        Some(object("variable.a = 1;", 13))
    );
    // From 1.20.80 a number is a constant and an absent member stays absent.
    let cereal = loaded(&effect_with(
        CEREAL,
        r#""events": {"e": {"expression": 5}, "f": {}}"#,
    ));
    let events = cereal.events.as_ref().unwrap();
    assert_eq!(events["e"].expression, Some(number(5.0)));
    assert_eq!(events["f"].expression, None);
    assert_eq!(events["f"].expression(), number(0.0));
}

/// A component that is no object, and the ones that say nothing in the game.
fn components_without_data_keep_their_defaults() {
    // A component whose value is no object: one line, every member at its default.
    let effect = loaded(&effect(
        LEGACY,
        r#""minecraft:emitter_shape_sphere": 1, "minecraft:emitter_rate_manual": []"#,
    ));
    assert_eq!(
        findings(effect.diagnostics()),
        [
            format!(
                "Error {COMPONENTS}/minecraft:emitter_shape_sphere particle/no_data_fields: Component has no data fields!"
            ),
            format!(
                "Error {COMPONENTS}/minecraft:emitter_rate_manual particle/no_data_fields: Component has no data fields!"
            ),
        ]
    );
    assert_eq!(
        component(&effect, "minecraft:emitter_shape_sphere"),
        json("{}")
    );
    assert_eq!(
        component(&effect, "minecraft:emitter_rate_manual"),
        json(r#"{"max_particles": 0}"#)
    );
    // Local space and lighting say nothing in the game; a value that is no object is noted
    // (INFO), and the component keeps its defaults. An object says nothing.
    let quiet = loaded(&effect_legacy(
        r#""minecraft:emitter_local_space": 1, "minecraft:particle_appearance_lighting": "x""#,
    ));
    assert_eq!(
        findings(quiet.diagnostics()),
        [
            format!(
                "Info {COMPONENTS}/minecraft:emitter_local_space core/invalid_type: wrong JSON type"
            ),
            format!(
                "Info {COMPONENTS}/minecraft:particle_appearance_lighting core/invalid_type: wrong JSON type"
            ),
        ]
    );
    assert_eq!(
        component(&quiet, "minecraft:emitter_local_space"),
        json("{}")
    );
    let quiet = effect_legacy(
        r#""minecraft:emitter_local_space": {}, "minecraft:particle_appearance_lighting": {}"#,
    );
    assert_eq!(found(&quiet), [] as [&str; 0]);
}

/// The block lists and the shapes of the legacy reader.
fn block_lists_and_shapes_log_their_lines() {
    // Block lists.
    let blocks = loaded(&effect_legacy(
        r#""minecraft:particle_expire_if_in_blocks": ["a", 1, "b"], "minecraft:particle_expire_if_not_in_blocks": 1"#,
    ));
    assert_eq!(
        findings(blocks.diagnostics()),
        [
            format!(
                "Error {COMPONENTS}/minecraft:particle_expire_if_in_blocks/1 particle/block_not_string: Block name is not a string!"
            ),
            format!(
                "Error {COMPONENTS}/minecraft:particle_expire_if_not_in_blocks particle/no_block_list: Component has no block list!"
            ),
        ]
    );
    assert_eq!(
        blocks
            .components
            .get::<ParticleExpireIfInBlocks>()
            .unwrap()
            .0,
        ["a", "b"]
    );
    // Shapes.
    let shapes = effect_legacy(
        r#""minecraft:emitter_shape_sphere": {"direction": "up"}, "minecraft:emitter_shape_disc": {"plane_normal": "w"}"#,
    );
    assert_eq!(
        found(&shapes),
        [
            format!(
                "Error {COMPONENTS}/minecraft:emitter_shape_sphere/direction particle/direction_invalid: direction has invalid value"
            ),
            format!(
                "Error {COMPONENTS}/minecraft:emitter_shape_disc/plane_normal particle/plane_normal_invalid: plane_normal has invalid value"
            ),
        ]
    );
}

/// The lifetime events of the legacy reader.
fn lifetime_events_log_their_lines() {
    // Lifetime events.
    let at = format!("{COMPONENTS}/minecraft:emitter_lifetime_events");
    let events = loaded(&effect_legacy(
        r#""minecraft:emitter_lifetime_events": {"creation_event": 1, "expiration_event": ["a", 2], "timeline": 1, "travel_distance_events": [], "looping_travel_distance_events": [1, {"distance": -1, "effects": "a"}, {"distance": 1}, {"effects": "b"}, {"distance": 0, "effects": ["c"]}]}"#,
    ));
    assert_eq!(
        findings(events.diagnostics()),
        [
            format!("Error {at}/creation_event particle/event_field: Error reading field"),
            format!(
                "Error {at}/expiration_event/1 particle/event_element: Error reading array element"
            ),
            format!("Error {at}/timeline particle/timeline: Error reading timeline"),
            format!(
                "Error {at}/travel_distance_events particle/travel_distance: Error reading travel_distance_events"
            ),
            format!(
                "Error {at}/looping_travel_distance_events/0 particle/looping: Error reading looping_travel_distance_events"
            ),
            // Each at the index the file has the entry at, behind the element that is no
            // object.
            format!(
                "Error {at}/looping_travel_distance_events/1 particle/looping_entry: Error reading and individual event from looping_travel_distance_events"
            ),
            format!(
                "Error {at}/looping_travel_distance_events/2 particle/looping_entry: Error reading and individual event from looping_travel_distance_events"
            ),
        ]
    );
    let stored = events.components.get::<EmitterLifetimeEvents>().unwrap();
    let kept: Vec<_> = stored
        .looping_travel_distance_events
        .as_ref()
        .unwrap()
        .iter()
        .map(|e| (e.distance, e.effects.as_slice().to_vec()))
        .collect();
    // A missing distance reads as 0; a distance of exactly 0 is kept.
    assert_eq!(
        kept,
        [(0.0, vec!["b".to_owned()]), (0.0, vec!["c".to_owned()])]
    );
    assert_eq!(stored.expiration_event.as_ref().unwrap().as_slice(), ["a"]);
    let looping = effect_legacy(
        r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": {}}"#,
    );
    assert_eq!(
        found(&looping),
        [format!(
            "Error {at}/looping_travel_distance_events particle/looping: Error reading looping_travel_distance_events"
        )]
    );
}

#[test]
fn the_legacy_component_readers_log_their_lines_and_keep_the_defaults() {
    components_without_data_keep_their_defaults();
    block_lists_and_shapes_log_their_lines();
    lifetime_events_log_their_lines();
}

fn effect_legacy(components: &str) -> String {
    effect(LEGACY, components)
}

#[test]
fn the_legacy_colour_reader_logs_its_lines_and_the_tint_is_white() {
    let at = format!("{COMPONENTS}/minecraft:particle_appearance_tinting/color");
    let invalid = format!("Error {at} particle/invalid_color: Component has invalid color field!");
    for (color, line) in [
        (
            "1",
            format!(
                "Error {at} particle/color_shape: Color value isn't a string or array of values"
            ),
        ),
        (
            r#""red""#,
            format!("Error {at} particle/color_string: Color value string has problems!"),
        ),
        (
            "[1, 1]",
            format!("Error {at} particle/color_count: Color value has wrong number of elements"),
        ),
        (
            "[1, 1, 1, 1, 1]",
            format!("Error {at} particle/color_count: Color value has wrong number of elements"),
        ),
        (
            r#"{"gradient": 1, "interpolant": 0}"#,
            format!(
                "Error {at}/gradient particle/gradient: Color gradient area not set up correctly"
            ),
        ),
    ] {
        let effect = loaded(&effect_legacy(&format!(
            r#""minecraft:particle_appearance_tinting": {{"color": {color}}}"#
        )));
        let lines = findings(effect.diagnostics());
        assert_eq!(lines[0], line, "{color}");
        if !color.starts_with('{') {
            assert_eq!(lines[1..], std::slice::from_ref(&invalid)[..], "{color}");
            assert_eq!(
                component(&effect, "minecraft:particle_appearance_tinting"),
                json(r#"{"color": [1, 1, 1, 1]}"#),
                "{color}"
            );
        }
    }
    // A colour of a gradient that cannot be read spoils the gradient.
    let effect = loaded(&effect_legacy(
        r##""minecraft:particle_appearance_tinting": {"color": {"gradient": ["#FFFFFF", 1], "interpolant": 0}}"##,
    ));
    assert_has(
        effect.diagnostics(),
        &format!("Error {at}/gradient particle/gradient: Color gradient area not set up correctly"),
    );
    // The hex forms: `#` and one to eight hexadecimal digits.
    for (text, fine) in [
        ("#F", true),
        ("#12345678", true),
        ("#123456789", false),
        ("#", false),
        ("FF0000", false),
        ("#GG0000", false),
    ] {
        let document = effect_cereal(&format!(
            r#""minecraft:particle_appearance_tinting": {{"color": "{text}"}}"#
        ));
        assert_eq!(found(&document).is_empty(), fine, "{text}");
    }
}

#[test]
fn a_keyed_gradient_is_used_in_the_order_of_its_keys() {
    let effect = loaded(&effect(
        CEREAL,
        r##""minecraft:particle_appearance_tinting": {"color": {"gradient": {"1.0": "#FF0000", "0.0": "#00FF00", "0.5": "#0000FF"}, "interpolant": 0}}"##,
    ));
    let TintColor::Gradient(tint) = &effect
        .components
        .get::<ParticleAppearanceTinting>()
        .unwrap()
        .color
    else {
        panic!();
    };
    let Gradient::Keyed(colors) = &tint.gradient else {
        panic!();
    };
    // As written in the value; sorted for the game.
    assert_eq!(colors.keys().collect::<Vec<_>>(), ["1.0", "0.0", "0.5"]);
    let stops: Vec<f32> = tint.gradient.stops().iter().map(|stop| stop.0).collect();
    assert_eq!(stops, [0.0, 0.5, 1.0]);
    let list = Gradient::List(colors.values().cloned().collect());
    let stops: Vec<f32> = list.stops().iter().map(|stop| stop.0).collect();
    assert_eq!(stops, [0.0, 0.5, 1.0]);
}

fn effect_cereal(components: &str) -> String {
    effect(CEREAL, components)
}

/// The pins of the mutation pass: each rule here had a half no test held.
#[test]
fn the_halves_the_mutation_pass_found_unpinned() {
    // A 1.20.80 curve without `nodes` is one finding, the missing member: the node-count
    // line is about a list that is there.
    let d = rejected(&effect_with(
        CEREAL,
        r#""curves": {"variable.c": {"type": "bezier", "input": 0}}"#,
    ));
    assert_eq!(
        texts(&d),
        [
            "Error particle_effect/curves/variable.c/nodes core/missing_required_field: missing required field"
        ]
    );
    // A sign is no hexadecimal digit.
    for text in ["#+FF", "#-1", "# FF"] {
        let document = effect_cereal(&format!(
            r#""minecraft:particle_appearance_tinting": {{"color": "{text}"}}"#
        ));
        assert_eq!(found(&document).len(), 1, "{text}");
    }
    // From 1.20.80 a component without a name is an unknown component like any other: under
    // the strict mask it refuses the document.
    let unnamed = rejected(&effect_cereal(r#""": {"a": 1}"#));
    assert_eq!(
        findings(&unnamed),
        [format!(
            "Error {COMPONENTS}/ core/unrecognized_component: unrecognized component"
        )]
    );
}

/// What the legacy parser leaves out
/// BESIDE a member it has a line for is noted where the file has it: the curves, events and
/// components behind render parameters that stop it; an event's `particle_effect` /
/// `sound_effect` without its required strings; a collision event without an `event`. And an
/// event's Molang that is no string, which the parser reads as 0 without a line.
#[test]
fn what_the_legacy_parser_leaves_out_beside_a_member_is_noted_where_it_is() {
    let noted = |text: &str| -> Vec<String> {
        loaded(text)
            .diagnostics()
            .iter()
            .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Info)
            .filter(|f| matches!(f.code.id(), "core/dropped_field" | "core/invalid_type"))
            .map(|f| format!("{} {}", f.code.id(), f.location.json_path))
            .collect()
    };
    // The parser stops at the material: the rest of the effect is not read.
    let stopped = format!(
        r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "demo:x", "basic_render_parameters": {{"material": 1, "texture": "t"}}}}, "components": {{{BASE}}}, "curves": {{"variable.c": {{"type": "linear", "input": 0, "nodes": [0]}}}}, "events": {{"e": {{"log": "x"}}}}}}}}"#
    );
    assert_eq!(
        noted(&stopped),
        [
            "core/dropped_field particle_effect/curves",
            "core/dropped_field particle_effect/events",
            "core/dropped_field particle_effect/components",
        ]
    );
    // An event's sections and Molang.
    let events = effect_with(
        LEGACY,
        r#""events": {"e": {"expression": [7], "particle_effect": {"effect": "demo:other", "type": 7, "pre_effect_expression": 7}, "sound_effect": {"event_name": 7, "zz": 1}}, "f": {"expression": "v.x = 1;", "particle_effect": {"effect": "demo:other", "type": "emitter"}}}"#,
    );
    assert_eq!(
        noted(&events),
        [
            "core/invalid_type particle_effect/events/e/expression",
            "core/invalid_type particle_effect/events/e/particle_effect/pre_effect_expression",
            "core/dropped_field particle_effect/events/e/particle_effect",
            "core/dropped_field particle_effect/events/e/sound_effect",
        ]
    );
    // A collision event without its `event`.
    let collision = effect(
        LEGACY,
        r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [{"event": "e", "min_speed": 3}, {"event": 7, "min_speed": 4}]}"#,
    );
    assert_eq!(
        noted(&collision),
        [format!(
            "core/dropped_field {COMPONENTS}/minecraft:particle_motion_collision/events/1"
        )]
    );
}

/// The legacy parser returns what it has where it
/// stops (the render parameters; a `components` that is missing or no object). An unknown
/// member of the payload is no part of what it stopped before: it is told as in an effect the
/// parser reads to the end, and not kept.
#[test]
fn an_unknown_payload_member_of_an_effect_the_legacy_parser_stopped_in_is_told() {
    for (what, rest) in [
        (
            "no components",
            r#""description": {"identifier": "demo:x", "basic_render_parameters": {"material": "m", "texture": "t"}}, "zz": 3"#,
        ),
        (
            "components of another type",
            r#""description": {"identifier": "demo:x", "basic_render_parameters": {"material": "m", "texture": "t"}}, "components": 7, "zz": 3"#,
        ),
        (
            "render parameters that stop the parser",
            r#""description": {"identifier": "demo:x", "basic_render_parameters": {"material": 7, "texture": "t"}}, "components": {}, "zz": 3"#,
        ),
    ] {
        let text = format!(r#"{{"format_version": "1.10.0", "particle_effect": {{{rest}}}}}"#);
        let document = loaded(&text);
        let unknown: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.code.id() == "core/unrecognized_field")
            .map(|f| f.location.json_path.to_string())
            .collect();
        assert_eq!(
            unknown,
            ["particle_effect/zz"],
            "the unknown member is not told: {what}"
        );
        let written = super::common::written(&document);
        assert_eq!(
            super::common::at(&written, "particle_effect/zz"),
            None,
            "the unknown member is written: {what}: {written:?}"
        );
    }
}

/// A curve that is not made for
/// its `type` or its `nodes`, and a `randomize` option that is skipped for its `weight`, go
/// with every member they have. The line is at the one member (which the file may not have);
/// the curve or the option is noted where it is. One that is read says nothing.
#[test]
fn a_curve_or_a_random_option_that_goes_whole_for_one_member_is_noted_where_it_is() {
    let dropped = |text: &str| -> Vec<String> {
        loaded(text)
            .diagnostics()
            .iter()
            .filter(|f| f.code.id() == "core/dropped_field")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect()
    };
    let curves = r#""curves": {"variable.no_type": {"input": "v.particle_age", "horizontal_range": 2, "nodes": [0, 1]}, "variable.kind": {"type": "zigzag", "input": "v.particle_age", "nodes": [0, 1]}, "variable.typed": {"type": 7, "input": "v.particle_age", "nodes": [0, 1]}, "variable.nodes": {"type": "linear", "input": "v.particle_age", "nodes": {"a": 1}}, "variable.chain": {"type": "bezier_chain", "input": "v.particle_age", "nodes": [0, 1]}, "variable.good": {"type": "linear", "input": "v.particle_age", "horizontal_range": 2, "nodes": [0, 1]}}"#;
    assert_eq!(
        dropped(&effect_with(LEGACY, curves)),
        [
            "Info particle_effect/curves/variable.no_type",
            "Info particle_effect/curves/variable.kind",
            "Info particle_effect/curves/variable.typed",
            "Info particle_effect/curves/variable.nodes",
            "Info particle_effect/curves/variable.chain",
        ],
        "a curve that is not made is not told at the curve"
    );
    // The cereal conversion leaves a curve of an unknown type out with the same line.
    assert_eq!(
        dropped(&effect_with(
            CEREAL,
            r#""curves": {"variable.kind": {"type": "zigzag", "input": "v.particle_age", "nodes": [0, 1]}}"#
        )),
        ["Info particle_effect/curves/variable.kind"]
    );
    let options = r#""events": {"e": {"randomize": [{"expression": "v.a = 1;"}, {"weight": -1, "expression": "v.b = 2;", "log": "x"}, {"weight": "w", "log": "y"}, {"weight": 1, "expression": "v.c = 3;"}]}}"#;
    assert_eq!(
        dropped(&effect_with(LEGACY, options)),
        [
            "Info particle_effect/events/e/randomize/0",
            "Info particle_effect/events/e/randomize/1",
            "Info particle_effect/events/e/randomize/2",
        ],
        "a random option that is skipped is not told at the option"
    );
}

/// The legacy parser's line for a looping entry it drops (a negative distance, no effects)
/// is at the index the file has the entry at, behind an entry that is no object.
#[test]
fn a_looping_entry_dropped_behind_one_left_out_is_told_at_the_index_of_the_file() {
    let document = loaded(&effect(
        LEGACY,
        r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [7, {"distance": 1, "effects": "e"}, 7, {"distance": -1, "effects": "e"}]}"#,
    ));
    let at = "particle_effect/components/minecraft:emitter_lifetime_events/looping_travel_distance_events";
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.severity >= bedrock_addon_core::diagnostic::Severity::Warning)
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        told,
        [format!("{at}/0"), format!("{at}/2"), format!("{at}/3")]
    );
}

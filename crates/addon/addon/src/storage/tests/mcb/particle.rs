//! `particle_effect` bodies.

use super::{Body, assert_unsupported, document};
use crate::storage::StorageError;
use crate::storage::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 26, 10);

/// An effect with `components` (name, fields), then `tail`: the curves and the events.
fn effect(components: &[(&str, Body)], tail: &[u8]) -> Result<String, StorageError> {
    let mut body = Body::default()
        .string("demo:spark")
        .string("particles_alpha")
        .string("textures/particle/demo")
        .u32(u32::try_from(components.len()).unwrap());
    for (name, fields) in components {
        body = body.component(name).raw(&fields.0);
    }
    to_json_text(&document("particle_effect", VERSION, &body.raw(tail).0))
}

/// What precedes the first component's value in the text of an effect of [`effect`] that has
/// neither curves nor events.
fn before(name: &str) -> String {
    format!(
        concat!(
            r#"{{"format_version":"1.26.10","particle_effect":{{"description":{{"#,
            r#""identifier":"demo:spark","basic_render_parameters":{{"#,
            r#""material":"particles_alpha","texture":"textures/particle/demo"}}}},"#,
            r#""components":{{"{}":"#
        ),
        name
    )
}

/// The JSON of the one component `name` with `fields`.
#[track_caller]
fn component(name: &str, fields: Body) -> String {
    let name = format!("minecraft:{name}");
    let text = effect(&[(&name, fields)], &[0, 0]).unwrap();
    let value = text
        .strip_prefix(&before(&name))
        .and_then(|rest| rest.strip_suffix("}}}"));
    value.unwrap_or_else(|| panic!("{text}")).to_owned()
}

fn fields() -> Body {
    Body::default()
}

const ZERO: [f32; 3] = [0.0; 3];

impl Body {
    /// Expressions that are floats.
    fn numbers(self, values: &[f32]) -> Body {
        values.iter().fold(self, |body, value| body.number(*value))
    }
}

#[test]
fn an_effect_without_components_is_its_description() {
    assert_eq!(
        effect(&[], &[0, 0]).unwrap(),
        concat!(
            r#"{"format_version":"1.26.10","particle_effect":{"description":{"#,
            r#""identifier":"demo:spark","basic_render_parameters":{"#,
            r#""material":"particles_alpha","texture":"textures/particle/demo"}},"#,
            r#""components":{}}}"#
        )
    );
}

#[test]
fn an_expression_is_a_float_or_molang_text() {
    assert_eq!(
        component("emitter_rate_instant", fields().number(3.0)),
        r#"{"num_particles":3.0}"#
    );
    assert_eq!(
        component("emitter_rate_instant", fields().molang("math.random(2, 4)")),
        r#"{"num_particles":"math.random(2, 4)"}"#
    );
    assert_eq!(
        component("emitter_rate_manual", fields().number(50.0)),
        r#"{"max_particles":50.0}"#
    );
    assert_eq!(
        component("emitter_rate_steady", fields().molang("v.r").number(20.0)),
        r#"{"spawn_rate":"v.r","max_particles":20.0}"#
    );
}

#[test]
fn molang_text_with_a_version_is_an_expression_object() {
    // the form of the builds that write the Molang version after the string
    assert_eq!(
        component(
            "emitter_rate_steady",
            fields().molang_at("v.r", 12).number(20.0)
        ),
        r#"{"spawn_rate":{"expression":"v.r","version":12},"max_particles":20.0}"#
    );
}

#[test]
fn a_body_that_reads_to_two_values_is_unsupported() {
    // `color: ["v.r", 0.5, 0.25, 1.0]` in the form without a version. With one, the same
    // bytes are "v.r" at version 1, two empty texts at versions 319 and 16000, and 1.0:
    // each float's `01 00 00` reads as a version's high byte and an empty string. Nothing
    // says which form the body is in.
    let fields = fields().raw(&[0]).molang("v.r").numbers(&[0.5, 0.25, 1.0]);
    assert_unsupported(effect(
        &[("minecraft:particle_appearance_tinting", fields)],
        &[0, 0],
    ));
}

#[test]
fn a_string_of_128_bytes_or_more_has_a_two_byte_length() {
    let text = "v.a = 1;".repeat(30);
    assert_eq!(text.len(), 240);
    assert_eq!(
        component("particle_initial_speed", fields().raw(&[1]).molang(&text)),
        format!(r#""{text}""#)
    );
    // a third length byte, or two bytes for a length under 128, were never seen
    for length in [&[0x80, 0x80, 0x01][..], &[0x83, 0x00]] {
        let fields = fields().raw(&[1, 0]).raw(length).raw(b"abc");
        assert_unsupported(effect(
            &[("minecraft:particle_initial_speed", fields)],
            &[0, 0],
        ));
    }
}

#[test]
fn a_member_at_its_default_is_left_out_as_the_twins_do() {
    for (name, at_default, off_default, text) in [
        (
            "emitter_initialization",
            fields().numbers(&[0.0, 0.0]),
            fields().molang("v.a = 1;").number(0.0),
            r#"{"creation_expression":"v.a = 1;"}"#,
        ),
        (
            "emitter_lifetime_expression",
            fields().numbers(&[1.0, 0.0]),
            fields().number(1.0).molang("q.x"),
            r#"{"expiration_expression":"q.x"}"#,
        ),
        (
            "emitter_lifetime_once",
            fields().number(0.0),
            fields().number(2.5),
            r#"{"active_time":2.5}"#,
        ),
        (
            "emitter_shape_custom",
            fields().numbers(&ZERO).numbers(&ZERO),
            fields().numbers(&[0.0, 1.0, 0.0]).numbers(&ZERO),
            r#"{"offset":[0.0,1.0,0.0]}"#,
        ),
        (
            "particle_initial_spin",
            fields().numbers(&[0.0, 0.0]),
            fields().number(0.0).molang("v.w"),
            r#"{"rotation_rate":"v.w"}"#,
        ),
        (
            "particle_initialization",
            fields().numbers(&[0.0, 0.0]),
            fields().number(0.0).molang("v.z = 2;"),
            r#"{"per_render_expression":"v.z = 2;"}"#,
        ),
        (
            "particle_motion_dynamic",
            fields().numbers(&ZERO).numbers(&ZERO),
            fields()
                .numbers(&[0.0, -9.8, 0.0])
                .numbers(&[0.0, 0.0, 0.1]),
            r#"{"linear_acceleration":[0.0,-9.8,0.0],"rotation_drag_coefficient":0.1}"#,
        ),
    ] {
        assert_eq!(component(name, at_default), "{}", "{name}");
        assert_eq!(component(name, off_default), text, "{name}");
    }
    assert_eq!(component("particle_appearance_lighting", fields()), "{}");
    assert_eq!(
        component("emitter_lifetime_looping", fields().numbers(&[2.0, 0.0])),
        r#"{"active_time":2.0}"#
    );
    assert_eq!(
        component("emitter_lifetime_looping", fields().numbers(&[2.0, 0.5])),
        r#"{"active_time":2.0,"sleep_time":0.5}"#
    );
    // a zero that is not the default's bits is written
    assert_eq!(
        component("emitter_lifetime_once", fields().number(-0.0)),
        r#"{"active_time":-0.0}"#
    );
}

#[test]
fn local_space_has_position_and_rotation_and_a_third_byte_that_is_not_velocity() {
    assert_eq!(
        component("emitter_local_space", fields().raw(&[1, 0, 0])),
        r#"{"position":true,"rotation":false}"#
    );
    let third = fields().raw(&[1, 1, 1]);
    assert_unsupported(effect(&[("minecraft:emitter_local_space", third)], &[0, 0]));
}

#[test]
fn an_emitter_shape_has_an_optional_direction_by_name_or_by_vector() {
    assert_eq!(
        component(
            "emitter_shape_box",
            fields()
                .numbers(&ZERO)
                .numbers(&[1.0, 2.0, 3.0])
                .raw(&[1, 1, 0])
                .string("outwards")
        ),
        r#"{"half_dimensions":[1.0,2.0,3.0],"surface_only":true,"direction":"outwards"}"#
    );
    assert_eq!(
        component(
            "emitter_shape_sphere",
            fields()
                .numbers(&ZERO)
                .molang("v.s")
                .raw(&[0, 1, 1])
                .numbers(&[0.0, 1.0, 0.0])
        ),
        r#"{"radius":"v.s","direction":[0.0,1.0,0.0]}"#
    );
    assert_eq!(
        component("emitter_shape_entity_aabb", fields().raw(&[0, 0])),
        "{}"
    );
    // a point has no named direction: its optional is the vector itself
    assert_eq!(
        component("emitter_shape_point", fields().numbers(&ZERO).raw(&[0])),
        "{}"
    );
    assert_eq!(
        component(
            "emitter_shape_point",
            fields()
                .numbers(&[0.5, 0.0, 0.5])
                .raw(&[1])
                .numbers(&[0.0, 1.0, 0.0])
        ),
        r#"{"offset":[0.5,0.0,0.5],"direction":[0.0,1.0,0.0]}"#
    );
    // a disc ends with its plane normal, left out at [0, 1, 0]
    let disc = |normal: [f32; 3]| {
        fields()
            .numbers(&ZERO)
            .number(1.5)
            .raw(&[0, 0, 1])
            .numbers(&normal)
    };
    assert_eq!(
        component("emitter_shape_disc", disc([0.0, 1.0, 0.0])),
        r#"{"radius":1.5}"#
    );
    assert_eq!(
        component("emitter_shape_disc", disc([1.0, 0.0, 0.0])),
        r#"{"radius":1.5,"plane_normal":[1.0,0.0,0.0]}"#
    );
}

#[test]
fn a_billboard_has_its_optional_facing_direction_and_uv() {
    assert_eq!(
        component(
            "particle_appearance_billboard",
            fields().numbers(&[0.1, 0.2]).raw(&[0, 0, 0])
        ),
        r#"{"size":[0.1,0.2]}"#
    );
    let whole = fields()
        .molang("v.size")
        .number(0.2)
        .raw(&[1])
        .string("lookat_xyz")
        // direction: mode, custom_direction, min_speed_threshold
        .raw(&[1])
        .string("custom")
        .raw(&[1])
        .numbers(&[1.0, 0.0, 0.0])
        .f32(0.01)
        // uv: texture size, uv, uv_size
        .raw(&[1])
        .i32(16)
        .i32(1)
        .raw(&[1])
        .numbers(&[0.0, 0.0])
        .raw(&[1])
        .numbers(&[8.0, 8.0])
        // flipbook: base, size, step, rate, max_frame, stretch, loop
        .raw(&[1])
        .numbers(&[0.0, 0.0])
        .f32s(&[8.0, 8.0, 0.0, 8.0, 4.0])
        .number(4.0)
        .raw(&[1, 0]);
    assert_eq!(
        component("particle_appearance_billboard", whole),
        concat!(
            r#"{"size":["v.size",0.2],"facing_camera_mode":"lookat_xyz","#,
            r#""direction":{"mode":"custom","custom_direction":[1.0,0.0,0.0],"#,
            r#""min_speed_threshold":0.01},"#,
            r#""uv":{"texture_width":16,"uv":[0.0,0.0],"uv_size":[8.0,8.0],"#,
            r#""flipbook":{"base_UV":[0.0,0.0],"size_UV":[8.0,8.0],"step_UV":[0.0,8.0],"#,
            r#""frames_per_second":4.0,"max_frame":4.0,"stretch_to_lifetime":true}}}"#
        )
    );
    // the flipbook's defaults: size 1 × 1, no rate, neither flag
    let plain_flipbook = fields()
        .numbers(&[1.0, 1.0])
        .raw(&[0, 0, 1])
        .i32(1)
        .i32(12)
        .raw(&[0, 0, 1])
        .numbers(&[0.0, 11.0])
        .f32s(&[1.0, 1.0, 0.0, -1.0, 0.0])
        .molang("v.frames")
        .raw(&[0, 1]);
    assert_eq!(
        component("particle_appearance_billboard", plain_flipbook),
        concat!(
            r#"{"size":[1.0,1.0],"uv":{"texture_height":12,"flipbook":{"#,
            r#""base_UV":[0.0,11.0],"step_UV":[0.0,-1.0],"max_frame":"v.frames","#,
            r#""loop":true}}}"#
        )
    );
}

#[test]
fn a_tint_is_a_colour_or_a_gradient_keyed_by_time_text() {
    assert_eq!(
        component(
            "particle_appearance_tinting",
            fields()
                .raw(&[0])
                .molang("v.r")
                .molang("v.g")
                .numbers(&[0.25, 1.0])
        ),
        r#"{"color":["v.r","v.g",0.25,1.0]}"#
    );
    let gradient = fields()
        .raw(&[1, 1, 2])
        .string("0.000000")
        .numbers(&[1.0, 0.0, 0.0, 1.0])
        .string("1.000000")
        .numbers(&[0.0, 0.0, 1.0, 0.5])
        .molang("v.t");
    assert_eq!(
        component("particle_appearance_tinting", gradient),
        concat!(
            r#"{"color":{"gradient":{"0.000000":[1.0,0.0,0.0,1.0],"#,
            r#""1.000000":[0.0,0.0,1.0,0.5]},"interpolant":"v.t"}}"#
        )
    );
    // one time twice is no JSON object
    let twice = fields()
        .raw(&[1, 1, 2])
        .string("0.000000")
        .numbers(&[1.0, 0.0, 0.0, 1.0])
        .string("0.000000")
        .numbers(&[0.0, 0.0, 1.0, 0.5])
        .number(0.0);
    assert_unsupported(effect(
        &[("minecraft:particle_appearance_tinting", twice)],
        &[0, 0],
    ));
}

#[test]
fn the_components_that_are_not_objects() {
    assert_eq!(
        component(
            "particle_expire_if_not_in_blocks",
            fields()
                .raw(&[2])
                .string("minecraft:air")
                .string("minecraft:water")
        ),
        r#"["minecraft:air","minecraft:water"]"#
    );
    assert_eq!(
        component("particle_initial_speed", fields().raw(&[1]).number(10.0)),
        "10.0"
    );
    assert_eq!(
        component(
            "particle_initial_speed",
            fields().raw(&[0]).numbers(&[1.0, 2.0, 3.0])
        ),
        "[1.0,2.0,3.0]"
    );
    assert_eq!(
        component("particle_kill_plane", fields().f32s(&[0.0, 1.0, 0.0, 4.0])),
        "[0.0,1.0,0.0,4.0]"
    );
}

#[test]
fn a_lifetime_and_a_parametric_motion_have_optionals() {
    assert_eq!(
        component(
            "particle_lifetime_expression",
            fields().raw(&[1]).number(3.0).number(0.0)
        ),
        r#"{"max_lifetime":3.0}"#
    );
    assert_eq!(
        component(
            "particle_lifetime_expression",
            fields().raw(&[0]).molang("v.done")
        ),
        r#"{"expiration_expression":"v.done"}"#
    );
    assert_eq!(
        component("particle_motion_parametric", fields().raw(&[0, 0, 0])),
        "{}"
    );
    assert_eq!(
        component(
            "particle_motion_parametric",
            fields()
                .raw(&[1])
                .numbers(&[0.0, 1.0, 0.0])
                .raw(&[0, 1])
                .molang("v.rot")
        ),
        r#"{"relative_position":[0.0,1.0,0.0],"rotation":"v.rot"}"#
    );
}

#[test]
fn a_collision_tells_no_events_from_an_empty_list() {
    let collision = |tail: Body| {
        let fields = fields().number(1.0).f32s(&[0.0, 0.0, 0.25]).raw(&[0]);
        component("particle_motion_collision", fields.raw(&tail.0))
    };
    assert_eq!(
        collision(fields().raw(&[0])),
        r#"{"collision_radius":0.25}"#
    );
    assert_eq!(
        collision(fields().raw(&[1, 0])),
        r#"{"collision_radius":0.25,"events":[]}"#
    );
    assert_eq!(
        component(
            "particle_motion_collision",
            fields()
                .molang("v.on")
                .f32s(&[0.5, 0.1, 0.01])
                .raw(&[1, 1, 2])
                .f32(2.0)
                .string("hit")
                .f32(3.0)
                .string("hard")
        ),
        concat!(
            r#"{"enabled":"v.on","collision_drag":0.5,"coefficient_of_restitution":0.1,"#,
            r#""collision_radius":0.01,"expire_on_contact":true,"#,
            r#""events":[{"event":"hit"},{"min_speed":3.0,"event":"hard"}]}"#
        )
    );
}

#[test]
fn curves_and_events_follow_the_components() {
    let tail = fields()
        // curves: one, linear, two nodes
        .raw(&[1, 1])
        .string("variable.c")
        .raw(&[0])
        .string("linear")
        .raw(&[2])
        .numbers(&[0.0, 1.0])
        .molang("v.age")
        .number(1.0)
        // events: one, a particle effect
        .raw(&[1, 1])
        .string("hit")
        .raw(&[1, 0, 0, 0, 0])
        .string("demo:splash")
        .string("emitter")
        .raw(&[1, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        effect(&[], &tail.0).unwrap(),
        concat!(
            r#"{"format_version":"1.26.10","particle_effect":{"description":{"#,
            r#""identifier":"demo:spark","basic_render_parameters":{"#,
            r#""material":"particles_alpha","texture":"textures/particle/demo"}},"#,
            r#""curves":{"variable.c":{"type":"linear","nodes":[0.0,1.0],"#,
            r#""input":"v.age","horizontal_range":1.0}},"#,
            r#""events":{"hit":{"particle_effect":{"effect":"demo:splash","type":"emitter"}}},"#,
            r#""components":{}}}"#
        )
    );
}

#[test]
fn a_component_vanilla_does_not_bake_is_unsupported() {
    let fields = fields().number(1.0);
    assert_unsupported(effect(
        &[("minecraft:emitter_lifetime_events", fields)],
        &[0, 0],
    ));
}

/// A body with a NaN is read the same in both Molang forms, and two NaNs are never equal: the
/// fault is the float JSON cannot write, not an unknown layout.
#[test]
fn a_nan_in_a_body_both_forms_read_alike_is_the_non_finite_fault() {
    let body = effect(
        &[("minecraft:emitter_rate_instant", fields().number(f32::NAN))],
        &[0, 0],
    );
    assert!(
        matches!(
            body,
            Err(StorageError::Mcb {
                fault: crate::storage::McbFault::NonFiniteFloat
            })
        ),
        "{body:?}"
    );
    // The two forms still have to agree: an infinity is the same fault, a body only one
    // form reads is read in that form.
    let infinite = effect(
        &[(
            "minecraft:emitter_rate_instant",
            fields().number(f32::INFINITY),
        )],
        &[0, 0],
    );
    assert!(matches!(
        infinite,
        Err(StorageError::Mcb {
            fault: crate::storage::McbFault::NonFiniteFloat
        })
    ));
}

/// The JSON text of the MCB fixture `name` (`tests/fixtures/storage/mcb/`).
fn bake(name: &str) -> Result<String, StorageError> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/storage/mcb")
        .join(name);
    to_json_text(&std::fs::read(path).unwrap())
}

/// The Drop 4 freezing effects (`experimental_y_2026_drop_4/particles/`) are baked at 1.26.50;
/// the game has one effect document layout, so their bodies have the layout of every other
/// bake. They decode to the last byte (bytes after the events are unsupported).
#[test]
fn the_freezing_effects_baked_at_1_26_50_decode() {
    for (name, identifier) in [
        (
            "freezing_ambient_1_26_50.json",
            "minecraft:freezing_ambient",
        ),
        (
            "freezing_emitter_1_26_50.json",
            "minecraft:freezing_emitter",
        ),
    ] {
        let text = bake(name).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let prefix = format!(
            r#"{{"format_version":"1.26.50","particle_effect":{{"description":{{"identifier":"{identifier}","basic_render_parameters":{{"material":"particles_alpha","texture":"textures/particle/freezing"}}}},"components":{{"#
        );
        assert!(text.starts_with(&prefix), "{name}: {text}");
        // The whole text, pinned: the decoded JSON as read, each member in the layout's form.
        let decoded = name.replace(".json", ".decoded.json");
        let pinned = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/storage/mcb")
                .join(&decoded),
        )
        .unwrap();
        assert_eq!(text, pinned, "{name}");
    }
}

#[test]
fn a_particle_body_at_a_version_vanilla_does_not_bake_is_unsupported() {
    for version in [(1, 26, 40), (1, 26, 60)] {
        let body = Body::default()
            .string("demo:spark")
            .string("particles_alpha")
            .string("textures/particle/demo")
            .u32(0)
            .raw(&[0, 0]);
        assert_unsupported(to_json_text(&document("particle_effect", version, &body.0)));
    }
}

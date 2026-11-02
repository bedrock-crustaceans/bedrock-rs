//! `minecraft:camera_entity` bodies.

use super::{Body, assert_unsupported, document};
use crate::storage::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 26, 10);

/// The body of the one vanilla camera after its identifier, component by component.
fn death_camera(identifier: &str) -> Body {
    Body::default()
        .string(identifier)
        .u32(13)
        .component("minecraft:allow_inside_block")
        .component("minecraft:camera")
        .f32s(&[66.0, 0.025, 2500.0])
        .component("minecraft:camera_attach_to_player")
        .raw(&[0])
        .component("minecraft:camera_avoidance")
        .f32s(&[0.0, 0.25])
        .raw(&[0, 0])
        .component("minecraft:camera_look_at_player")
        .component("minecraft:camera_offset")
        .f32s(&[0.0, 0.0, -0.0, 2.0, 5.0])
        .component("minecraft:camera_orbit")
        .f32s(&[0.0, 0.0, 0.0, 8.0, 60.0, 60.0, -180.0, 180.0])
        .raw(&[0, 0])
        .component("minecraft:camera_render_player_model")
        .component("minecraft:camera_third_person")
        .component("minecraft:death_camera_component")
        .component("minecraft:gameplay_affects_fov")
        .component("minecraft:player_state_affects_rendering")
        .component("minecraft:update_player_from_camera")
        .string("along_camera")
}

#[test]
fn the_one_known_camera_body_reads_as_its_twin_has_it() {
    let text = to_json_text(&document(
        "minecraft:camera_entity",
        VERSION,
        &death_camera("demo:camera").0,
    ))
    .unwrap();
    assert_eq!(
        text,
        concat!(
            r#"{"format_version":"1.26.10","minecraft:camera_entity":{"#,
            r#""description":{"identifier":"demo:camera"},"components":{"#,
            r#""minecraft:allow_inside_block":{},"#,
            r#""minecraft:camera":{"field_of_view":66.0},"#,
            r#""minecraft:camera_attach_to_player":{},"#,
            r#""minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0.0,"#,
            r#""distance_constraint_min":0.25},"#,
            r#""minecraft:camera_look_at_player":{},"#,
            // the body's third float is `-0.0` (the twin writes `-0`): the sign is the body's
            r#""minecraft:camera_offset":{"entity":[-0.0,2.0,5.0]},"#,
            r#""minecraft:camera_orbit":{"azimuth_smoothing_spring":0.0,"#,
            r#""polar_angle_smoothing_spring":0.0,"distance_smoothing_spring":0.0,"#,
            r#""radius":8.0,"polar_angle_min":60.0,"polar_angle_max":60.0},"#,
            r#""minecraft:camera_render_player_model":{},"#,
            r#""minecraft:camera_third_person":{},"#,
            r#""minecraft:death_camera_component":{},"#,
            r#""minecraft:gameplay_affects_fov":{},"#,
            r#""minecraft:player_state_affects_rendering":{},"#,
            r#""minecraft:update_player_from_camera":{}}}}"#
        )
    );
}

#[test]
fn any_other_camera_body_is_unsupported() {
    // one document shows no field apart from its neighbours: a body that differs from it in
    // a single float is not read
    let mut body = death_camera("demo:camera").0;
    let field_of_view = body
        .windows(4)
        .position(|window| window == 66.0_f32.to_le_bytes())
        .unwrap();
    body[field_of_view..field_of_view + 4].copy_from_slice(&70.0_f32.to_le_bytes());
    assert_unsupported(to_json_text(&document(
        "minecraft:camera_entity",
        VERSION,
        &body,
    )));
}

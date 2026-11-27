//! `minecraft:camera_entity` bodies.
//!
//! Vanilla bakes one camera, `minecraft:death_camera` (the same bytes in every install). One
//! document cannot separate a field from its neighbours, so no field layout is known: the body
//! is the identifier (string) followed by the bytes below, and is read only when it is those
//! bytes, as the members its JSON twin has. Any other camera body is unsupported.
//!
//! What the bytes show: a `u32` component count (13), then per component the FNV-1a 32 hash
//! of its name (in name order) and its fields; seven components have none.

use super::McbValue;
use super::reader::{Decoded, Reader, object, string};

/// Everything after the identifier of the one vanilla camera document.
const DEATH_CAMERA: &[u8] = &[
    0x0d, 0x00, 0x00, 0x00, // 13 components
    0xd2, 0x20, 0xde, 0x77, // minecraft:allow_inside_block
    0x25, 0x49, 0x9f, 0xf7, // minecraft:camera: 66.0, 0.025, 2500.0
    0x00, 0x00, 0x84, 0x42, 0xcd, 0xcc, 0xcc, 0x3c, 0x00, 0x40, 0x1c, 0x45, //
    0xc3, 0x1d, 0xa3, 0x63, 0x00, // minecraft:camera_attach_to_player
    0xc0, 0xc1, 0xbc, 0xc4, // minecraft:camera_avoidance: 0.0, 0.25, 00 00
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3e, 0x00, 0x00, //
    0x91, 0x88, 0x59, 0xa5, // minecraft:camera_look_at_player
    0xb9, 0x90, 0xf4, 0x51, // minecraft:camera_offset: 0.0, 0.0, -0.0, 2.0, 5.0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, //
    0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0xa0, 0x40, //
    0x3e, 0x04, 0xb9,
    0x4d, // minecraft:camera_orbit: 0.0 × 3, 8.0, 60.0, 60.0, -180.0, 180.0, 00 00
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
    0x00, 0x00, 0x00, 0x41, 0x00, 0x00, 0x70, 0x42, 0x00, 0x00, 0x70, 0x42, //
    0x00, 0x00, 0x34, 0xc3, 0x00, 0x00, 0x34, 0x43, 0x00, 0x00, //
    0x78, 0x71, 0xcf, 0xd7, // minecraft:camera_render_player_model
    0x9d, 0x5d, 0x69, 0xcb, // minecraft:camera_third_person
    0x1a, 0x12, 0x27, 0xda, // minecraft:death_camera_component
    0x0f, 0x52, 0xa5, 0x65, // minecraft:gameplay_affects_fov
    0xef, 0xf7, 0xbf, 0x97, // minecraft:player_state_affects_rendering
    0x40, 0x0c, 0x1c, 0x68, // minecraft:update_player_from_camera: "along_camera"
    0x0c, b'a', b'l', b'o', b'n', b'g', b'_', b'c', b'a', b'm', b'e', b'r', b'a',
];

/// The payload of a camera document.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let reader = &mut Reader::new(bytes);
    let identifier = reader.string()?;
    reader.pinned(DEATH_CAMERA)?;
    reader.end()?;
    let empty = || object(Vec::new());
    let float = |name: &str, value: f32| (Box::<str>::from(name), McbValue::F32(value));
    let components = vec![
        ("minecraft:allow_inside_block".into(), empty()),
        (
            "minecraft:camera".into(),
            object(vec![float("field_of_view", 66.0)]),
        ),
        ("minecraft:camera_attach_to_player".into(), empty()),
        (
            "minecraft:camera_avoidance".into(),
            object(vec![
                float("relax_distance_smoothing_spring", 0.0),
                float("distance_constraint_min", 0.25),
            ]),
        ),
        ("minecraft:camera_look_at_player".into(), empty()),
        (
            "minecraft:camera_offset".into(),
            object(vec![(
                "entity".into(),
                // The body has `-0.0` here (the twin `-0`), and the writer keeps the sign.
                McbValue::Array(vec![
                    McbValue::F32(-0.0),
                    McbValue::F32(2.0),
                    McbValue::F32(5.0),
                ]),
            )]),
        ),
        (
            "minecraft:camera_orbit".into(),
            object(vec![
                float("azimuth_smoothing_spring", 0.0),
                float("polar_angle_smoothing_spring", 0.0),
                float("distance_smoothing_spring", 0.0),
                float("radius", 8.0),
                float("polar_angle_min", 60.0),
                float("polar_angle_max", 60.0),
            ]),
        ),
        ("minecraft:camera_render_player_model".into(), empty()),
        ("minecraft:camera_third_person".into(), empty()),
        ("minecraft:death_camera_component".into(), empty()),
        ("minecraft:gameplay_affects_fov".into(), empty()),
        ("minecraft:player_state_affects_rendering".into(), empty()),
        ("minecraft:update_player_from_camera".into(), empty()),
    ];
    Ok(object(vec![
        (
            "description".into(),
            object(vec![("identifier".into(), string(identifier))]),
        ),
        ("components".into(), object(components)),
    ]))
}

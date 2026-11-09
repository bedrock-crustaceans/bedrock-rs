//! The camera entity rows: cam23 and cam24, with validation.md rule 7.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::document::Content;
use bedrock_addon::families::camera::{LookMode, PlayerViewMode, Present, RotationSpace};

use super::common::*;

/// The seven documents of vanilla's resource pack, copied unchanged from
/// `resource_packs/vanilla/cameras/` of the 1.26.20 development build (identical in retail
/// 1.26.45), `format_version` "1.26.10".
const VANILLA: [&str; 7] = [
    "death",
    "first_person",
    "fixed_boom",
    "follow_orbit",
    "free",
    "third_person",
    "third_person_front",
];

fn vanilla(name: &str) -> String {
    fixture(&format!("cam23_camera_entity_vanilla/cameras/{name}.json"))
}

#[test]
fn cam23_the_seven_vanilla_camera_documents_parse_without_a_finding() {
    let mut keys = std::collections::BTreeSet::new();
    for name in VANILLA {
        let text = vanilla(name);
        let document = loads(entity(&text));
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert!(
            document.description.identifier.starts_with("minecraft:"),
            "{name}"
        );
        // Written back and re-read, the document is the same, and so is the second writing.
        let written = document.to_json().unwrap();
        let again = loads(entity(&written));
        assert_eq!(again.value(), document.value(), "{name}");
        assert_eq!(again.to_json().unwrap(), written, "{name}");
        // Every component key of the file is written back: none is lost on the way.
        let json: bedrock_addon::json::Value = facet_json::from_str(&text).unwrap();
        let rewritten: bedrock_addon::json::Value = facet_json::from_str(&written).unwrap();
        let components = |value: &bedrock_addon::json::Value| -> Vec<String> {
            value
                .as_object()
                .unwrap()
                .get("minecraft:camera_entity")
                .unwrap()
                .as_object()
                .unwrap()
                .get("components")
                .unwrap()
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, _)| key.as_str().to_owned())
                .collect()
        };
        let mut before = components(&json);
        let mut after = components(&rewritten);
        before.sort();
        after.sort();
        assert_eq!(before, after, "{name}");
        keys.extend(before);
    }
    // The vanilla documents use 30 of the 31 components (all but `minecraft:camera_fly_move`).
    assert_eq!(keys.len(), 30);
    assert!(!keys.contains("minecraft:camera_fly_move"));
}

#[test]
fn cam23_absent_members_take_the_constructor_defaults() {
    let free = loads(entity(&vanilla("free")));
    let target = free.components.target.as_ref().unwrap();
    assert_eq!(target.tracking_radius, None);
    assert_eq!(target.tracking_radius(), 50.0);
    assert_eq!(target.rotation_speed(), 0.0);
    assert!(!target.snap_to_target() && !target.continue_targeting());
    assert_eq!(target.horizontal_rotation_limit, Some([0.0, 360.0]));
    assert_eq!(target.vertical_rotation_limit, Some([0.0, 180.0]));
    assert_eq!(free.components.stationary_camera.unwrap().pos, None);
    assert_eq!(free.components.extend_player_rendering, Some(Present {}));
    assert_eq!(free.components.camera_first_person, None);

    let first = loads(entity(&vanilla("first_person")));
    let camera = first.components.camera.as_ref().unwrap();
    assert_eq!(camera.field_of_view, Some(66.0));
    assert_eq!(camera.near_clipping_plane, None);
    assert_eq!(camera.near_clipping_plane(), 0.025);
    assert_eq!(camera.far_clipping_plane(), 2500.0);
    let look = first.components.camera_direct_look.as_ref().unwrap();
    assert_eq!((look.pitch_min(), look.pitch_max()), (-89.9, 89.9));
    assert_eq!(
        first
            .components
            .camera_perspective_option
            .unwrap()
            .view_mode,
        PlayerViewMode::FirstPerson
    );
    assert_eq!(
        first
            .components
            .update_player_from_camera
            .unwrap()
            .look_mode,
        None
    );
}

#[test]
fn cam23_every_default_is_the_schemas() {
    let document = loads(entity(&entity_json(
        "ex:all",
        r#""minecraft:camera_avoidance": {}, "minecraft:camera": {"field_of_view": 80},
        "minecraft:camera_direct_look": {}, "minecraft:camera_fly_move": {},
        "minecraft:camera_orbit": {}, "minecraft:camera_shake": {}, "minecraft:target": {},
        "minecraft:camera_third_person_boom": {"rotation_space": "world"},
        "minecraft:update_player_from_camera": {"look_mode": "at_camera"},
        "minecraft:camera_attach_to_player": {"locator": "head"},
        "minecraft:camera_perspective_option": {"view_mode": "third_person_front"}"#,
    )));
    let c = &document.components;
    let avoidance = c.camera_avoidance.as_ref().unwrap();
    assert_eq!(
        (
            avoidance.relax_distance_smoothing_spring(),
            avoidance.distance_constraint_min()
        ),
        (4.0, 0.0)
    );
    let look = c.camera_direct_look.as_ref().unwrap();
    assert_eq!((look.pitch_min(), look.pitch_max()), (-90.0, 90.0));
    let fly = c.camera_fly_move.as_ref().unwrap();
    assert_eq!((fly.speed(), fly.sprint_speed), (1.0, None));
    let orbit = c.camera_orbit.as_ref().unwrap();
    assert_eq!(
        (
            orbit.azimuth_smoothing_spring(),
            orbit.polar_angle_smoothing_spring(),
            orbit.distance_smoothing_spring(),
            orbit.radius(),
            orbit.polar_angle_min(),
            orbit.polar_angle_max(),
            orbit.yaw_angle_min(),
            orbit.yaw_angle_max(),
            orbit.invert_x_input(),
            orbit.invert_y_input()
        ),
        (4.0, 4.0, 4.0, 1.0, 5.0, 65.0, -180.0, 180.0, false, false)
    );
    let shake = c.camera_shake.as_ref().unwrap();
    assert_eq!(
        (
            shake.frequency(),
            shake.amplitude(),
            shake.noise_multiplier()
        ),
        (0.0, 0.0, 0.0)
    );
    assert_eq!(
        c.camera_third_person_boom.unwrap().rotation_space,
        Some(RotationSpace::World)
    );
    assert_eq!(
        c.update_player_from_camera.unwrap().look_mode,
        Some(LookMode::AtCamera)
    );
    assert_eq!(
        c.camera_attach_to_player
            .as_ref()
            .unwrap()
            .locator
            .as_ref()
            .unwrap(),
        "head"
    );
    assert_eq!(
        c.camera_perspective_option.unwrap().view_mode,
        PlayerViewMode::ThirdPersonFront
    );
    assert_eq!(RotationSpace::default(), RotationSpace::Local);
    assert_eq!(LookMode::default(), LookMode::AlongCamera);
}

#[test]
fn cam24_camera_entity_errors() {
    let at = |path: &str| format!("minecraft:camera_entity/{path}");
    let missing = |path: &str| {
        finding(
            "core/missing_required_field_worded",
            &at(path),
            "missing required field",
        )
    };
    let d = rejects(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera": {}"#,
    )));
    assert_eq!(
        errors(&d),
        [missing("components/minecraft:camera/field_of_view")]
    );
    let d = rejects(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera_perspective_option": {}"#,
    )));
    assert_eq!(
        errors(&d),
        [missing(
            "components/minecraft:camera_perspective_option/view_mode"
        )]
    );

    let limit = at("components/minecraft:target/horizontal_rotation_limit");
    let target = |limit: &str| {
        entity_json(
            "minecraft:free",
            &format!(r#""minecraft:target": {{"horizontal_rotation_limit": {limit}}}"#),
        )
    };
    for outside in ["[0, 400]", "[-1, 0]", "[360.5, 0]"] {
        let d = rejects(entity(&target(outside)));
        assert_eq!(
            errors(&d),
            [finding(
                "camera/target_horizontal_range",
                &limit,
                "x and y components of the horizontal rotation must be in the [0, 360] range."
            )],
            "{outside}"
        );
    }
    let d = rejects(entity(&target("[200, 200]")));
    assert_eq!(
        errors(&d),
        [finding(
            "camera/target_horizontal_sum",
            &limit,
            "The sum of the [x, y] horizontal rotation limits must not exceed 360.0 degrees."
        )]
    );
    loads(entity(&target("[0, 360]")));
    loads(entity(&target("[180, 180]")));

    for identifier in ["free", "a:b:c", ""] {
        let d = rejects(entity(&entity_json(identifier, "")));
        assert_eq!(
            errors(&d),
            [finding(
                "camera/entity_identifier",
                &at("description/identifier"),
                "\"identifier\" must contain a namespace and a name, separated by a colon"
            )],
            "{identifier}"
        );
    }
}

#[test]
fn rule_7_the_camera_entity_envelope_and_required_members() {
    let at = |path: &str| format!("minecraft:camera_entity/{path}");
    let missing = |path: &str| {
        finding(
            "core/missing_required_field_worded",
            &at(path),
            "missing required field",
        )
    };
    let d = rejects(entity(
        r#"{"format_version": "1.26.10", "minecraft:camera_entity": {}}"#,
    ));
    assert_eq!(errors(&d), [missing("description"), missing("components")]);
    let d = rejects(entity(
        r#"{"format_version": "1.26.10", "minecraft:camera_entity": {"description": {}, "components": {}}}"#,
    ));
    assert_eq!(errors(&d), [missing("description/identifier")]);
    // One struct at 0.0.0 reads every version up to the engine's.
    for version in ["0.0.0", "1.0.0", "1.26.10", "1.26.50", "1.26.60"] {
        let text = entity_json("minecraft:free", "").replace("1.26.10", version);
        loads(entity(&text));
    }
    let text = entity_json("minecraft:free", "").replace("1.26.10", "1.26.61");
    assert_eq!(
        errors(&rejects(entity(&text)))[0].0,
        "core/no_parser_for_version"
    );
    // The envelope is checked: two members, the payload under its key, a version.
    let text = entity_json("minecraft:free", "").replacen('{', r#"{"extra": 1, "#, 1);
    assert_eq!(
        errors(&rejects(entity(&text)))[0].0,
        "core/document_member_count"
    );
    let d = rejects(entity(
        r#"{"minecraft:camera_entity": {"description": {"identifier": "a:b"}, "components": {}}}"#,
    ));
    assert_eq!(errors(&d)[0].0, "core/no_valid_version");
    // The written document carries the version vanilla ships.
    let document = loads(entity(&entity_json("minecraft:free", "")));
    assert!(
        document
            .to_json()
            .unwrap()
            .contains(r#""format_version": "1.26.10""#)
    );
    assert_eq!(
        <bedrock_addon::families::camera::CameraEntity as Content>::NAME,
        "camera entity"
    );
}

#[test]
fn rule_7_camera_entity_ranges_enums_and_unknown_components() {
    let at = |path: &str| format!("minecraft:camera_entity/components/{path}");
    let range =
        |min: &str, max: &str| format!("the input is outside of the accepted range [{min}, {max}]");
    for (component, path, text) in [
        (
            r#""minecraft:camera_orbit": {"radius": 0.05}"#,
            "minecraft:camera_orbit/radius",
            range("0.100000", "64"),
        ),
        (
            r#""minecraft:camera_orbit": {"radius": 64.5}"#,
            "minecraft:camera_orbit/radius",
            range("0.100000", "64"),
        ),
        (
            r#""minecraft:camera_orbit": {"polar_angle_min": -1}"#,
            "minecraft:camera_orbit/polar_angle_min",
            range("0", "180"),
        ),
        (
            r#""minecraft:camera_orbit": {"polar_angle_max": 181}"#,
            "minecraft:camera_orbit/polar_angle_max",
            range("0", "180"),
        ),
        (
            r#""minecraft:camera_orbit": {"yaw_angle_min": -181}"#,
            "minecraft:camera_orbit/yaw_angle_min",
            range("-180", "180"),
        ),
        (
            r#""minecraft:camera_orbit": {"yaw_angle_max": 181}"#,
            "minecraft:camera_orbit/yaw_angle_max",
            range("-180", "180"),
        ),
        (
            r#""minecraft:camera_offset": {"view": [100.5, 0]}"#,
            "minecraft:camera_offset/view/0",
            range("-100", "100"),
        ),
        (
            r#""minecraft:camera_offset": {"entity": [0, 0, -100.5]}"#,
            "minecraft:camera_offset/entity/2",
            range("-100", "100"),
        ),
        (
            r#""minecraft:target": {"rotation_speed": 361}"#,
            "minecraft:target/rotation_speed",
            range("0", "360"),
        ),
        (
            r#""minecraft:target": {"rotation_speed": -1}"#,
            "minecraft:target/rotation_speed",
            range("0", "360"),
        ),
        (
            r#""minecraft:target": {"vertical_rotation_limit": [0, 181]}"#,
            "minecraft:target/vertical_rotation_limit/1",
            range("0", "180"),
        ),
        (
            r#""minecraft:target": {"vertical_rotation_limit": [-1, 0]}"#,
            "minecraft:target/vertical_rotation_limit/0",
            range("0", "180"),
        ),
    ] {
        let d = rejects(entity(&entity_json("minecraft:free", component)));
        assert_eq!(
            errors(&d),
            [finding("core/outside_accepted_range", &at(path), &text)],
            "{component}"
        );
    }
}

#[test]
fn rule_7_camera_entity_lower_bound_only_members() {
    let at = |path: &str| format!("minecraft:camera_entity/components/{path}");
    // A lower bound only: 0 or more.
    for (component, path) in [
        (
            r#""minecraft:camera_avoidance": {"relax_distance_smoothing_spring": -1}"#,
            "minecraft:camera_avoidance/relax_distance_smoothing_spring",
        ),
        (
            r#""minecraft:camera_avoidance": {"distance_constraint_min": -1}"#,
            "minecraft:camera_avoidance/distance_constraint_min",
        ),
        (
            r#""minecraft:camera_orbit": {"azimuth_smoothing_spring": -1}"#,
            "minecraft:camera_orbit/azimuth_smoothing_spring",
        ),
        (
            r#""minecraft:camera_orbit": {"polar_angle_smoothing_spring": -1}"#,
            "minecraft:camera_orbit/polar_angle_smoothing_spring",
        ),
        (
            r#""minecraft:camera_orbit": {"distance_smoothing_spring": -1}"#,
            "minecraft:camera_orbit/distance_smoothing_spring",
        ),
    ] {
        let d = rejects(entity(&entity_json("minecraft:free", component)));
        let found = errors(&d);
        assert_eq!(found.len(), 1, "{component}: {found:?}");
        assert_eq!(
            (found[0].0, found[0].1.clone()),
            ("core/invalid_value", at(path)),
            "{component}"
        );
    }
}

#[test]
fn rule_7_camera_entity_range_ends_load() {
    // The ends load; the vertical limit of `minecraft:target` has no order check.
    loads(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera_orbit": {"radius": 0.1, "polar_angle_min": 0, "polar_angle_max": 180, "yaw_angle_min": -180, "yaw_angle_max": 180, "azimuth_smoothing_spring": 0},
        "minecraft:camera_offset": {"view": [-100, 100], "entity": [100, -100, 0]},
        "minecraft:target": {"rotation_speed": 360, "vertical_rotation_limit": [180, 0]},
        "minecraft:camera_avoidance": {"distance_constraint_min": 0, "avoidance_rays_horizontal": [{"angle": 30, "weight": 0.5}, {}]}"#,
    )));
    loads(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera_orbit": {"radius": 64}, "minecraft:target": {"rotation_speed": 0}"#,
    )));
}

#[test]
fn rule_7_camera_entity_enums_and_unknown_components() {
    let at = |path: &str| format!("minecraft:camera_entity/components/{path}");
    for (component, path) in [
        (
            r#""minecraft:camera_perspective_option": {"view_mode": "second_person"}"#,
            "minecraft:camera_perspective_option/view_mode",
        ),
        (
            r#""minecraft:camera_third_person_boom": {"rotation_space": "screen"}"#,
            "minecraft:camera_third_person_boom/rotation_space",
        ),
        (
            r#""minecraft:update_player_from_camera": {"look_mode": "away"}"#,
            "minecraft:update_player_from_camera/look_mode",
        ),
    ] {
        let d = rejects(entity(&entity_json("minecraft:free", component)));
        assert_eq!(errors(&d).len(), 1, "{component}");
        assert_eq!(
            (errors(&d)[0].0, errors(&d)[0].1.clone()),
            ("core/invalid_enum_value", at(path))
        );
    }
    let d = rejects(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera_zoom": {}"#,
    )));
    assert_eq!(errors(&d).len(), 1);
    assert_eq!(
        (errors(&d)[0].0, errors(&d)[0].1.clone()),
        ("core/unrecognized_component", at("minecraft:camera_zoom"))
    );
    let d = rejects(entity(&entity_json(
        "minecraft:free",
        r#""minecraft:camera_first_person": {"fov": 1}"#,
    )));
    assert_eq!(errors(&d)[0].0, "core/unrecognized_field_worded");
}

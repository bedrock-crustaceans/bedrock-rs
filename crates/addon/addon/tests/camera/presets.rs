//! The rows about one preset file: cam01–cam06, cam08,
//! cam12–cam16, cam20 and cam21.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::camera::{
    AudioListener, CameraPreset, ControlScheme, PresetAimAssist, TargetMode, VANILLA_CAMERAS,
};
use bedrock_addon::id::{AimAssistPresetId, CameraPresetId};
use bedrock_addon::version::FormatVersion;

use super::common::*;

const UNRECOGNIZED: &str = "Unrecognized field on camera preset!";

fn f(value: f32) -> f32 {
    value
}

fn pair(x: f32, y: f32) -> [f32; 2] {
    [x, y]
}

/// The range text of `[min, max]`.
fn range(min: &str, max: &str) -> String {
    format!("the input is outside of the accepted range [{min}, {max}]")
}

/// The errors of a preset `ex:a` with `members`, at 1.19.50 and at 1.21.60 (vanilla's two
/// slices), which must agree; the closing line is checked and left out.
#[track_caller]
fn errors_of(members: &str) -> Vec<(&'static str, String, String)> {
    let mut seen = None;
    for version in ["1.19.50", "1.21.60"] {
        let d = rejects(preset(&preset_with(version, members)));
        let mut found = errors(&d);
        assert_eq!(found.pop(), Some(failed_presets()), "{version}: {d:#?}");
        if let Some(earlier) = &seen {
            assert_eq!(earlier, &found, "{version}");
        }
        seen = Some(found);
    }
    seen.unwrap()
}

/// A preset `ex:a` with `members` loads at both slices without a finding.
#[track_caller]
fn accepted(members: &str) {
    for version in ["1.19.50", "1.21.60"] {
        let document = loads(preset(&preset_with(version, members)));
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|f| f.severity < Severity::Warning),
            "{version}: {:#?}",
            document.diagnostics()
        );
    }
}

#[test]
fn cam01_the_docs_1_19_50_example_reaches_latest_unchanged() {
    // The creator docs' 1.19.50 example preset.
    let document = loads(preset(
        r#"{
  "format_version": "1.19.50",
  "minecraft:camera_preset": {
    "identifier": "example:example_free",
    "inherit_from": "minecraft:free",
    "pos_x": 30,
    "pos_y": 90,
    "pos_z": -20,
    "rot_x": 10,
    "rot_y": 40
  }
}"#,
    ));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let mut expected = CameraPreset::new(CameraPresetId::parse("example:example_free"));
    expected.inherit_from = Some(CameraPresetId::parse("minecraft:free"));
    expected.pos_x = Some(f(30.0));
    expected.pos_y = Some(f(90.0));
    expected.pos_z = Some(f(-20.0));
    expected.rot_x = Some(f(10.0));
    expected.rot_y = Some(f(40.0));
    // The step sets the flag.
    expected.apply_inherited_starting_rotation = true;
    assert_eq!(document.value(), &expected);
    let version = document.version();
    assert_eq!(version.written, Some(FormatVersion::new(1, 19, 50)));
    assert_eq!(version.parsed_as, FormatVersion::new(1, 19, 50));
    assert!(version.upgraded);
}

#[test]
fn cam02_a_preset_below_1_19_50_is_unsupported() {
    for version in ["1.0.0", "1.19.40"] {
        let d = rejects(preset(&preset_json(version, r#""identifier": "ex:a""#)));
        assert_eq!(
            errors(&d),
            [
                finding(
                    "UnsupportedVersion",
                    "format_version",
                    &format!("unsupported version: no parser available for version {version}")
                ),
                failed_presets()
            ]
        );
    }
    // The floor itself loads.
    loads(preset(&preset_with("1.19.50", r#""pos_x": 1"#)));
}

#[test]
fn cam02_a_preset_above_the_engine_is_unsupported() {
    let d = rejects(preset(&preset_json("1.26.61", r#""identifier": "ex:a""#)));
    assert_eq!(
        errors(&d),
        [
            finding(
                "UnsupportedVersion",
                "format_version",
                "unsupported version: no parser available for version 1.26.61"
            ),
            failed_presets()
        ]
    );
}

#[test]
fn cam04_an_unknown_member_drops_the_preset_at_both_slices() {
    for version in ["1.19.50", "1.21.60"] {
        let d = rejects(preset(&preset_with(version, r#""unexpected_field": 1.5"#)));
        assert_eq!(
            errors(&d),
            [finding(
                "UnrecognizedField",
                "minecraft:camera_preset/unexpected_field",
                UNRECOGNIZED
            )],
            "{version}"
        );
    }
}

#[test]
fn cam04_an_unknown_member_of_aim_assist_drops_the_preset() {
    let d = rejects(preset(&preset_with(
        "1.21.60",
        r#""aim_assist": {"preset": "ex:aa", "strength": 1}"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "UnrecognizedField",
            "minecraft:camera_preset/aim_assist/strength",
            UNRECOGNIZED
        )]
    );
}

#[test]
fn cam05_align_target_and_camera_forward_is_unknown_at_1_21_70() {
    let d = rejects(preset(&preset_with(
        "1.21.70",
        r#""align_target_and_camera_forward": true"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "UnrecognizedField",
            "minecraft:camera_preset/align_target_and_camera_forward",
            UNRECOGNIZED
        )]
    );
}

#[test]
fn cam06_every_field_is_accepted_at_1_19_50() {
    let document = loads(preset(&preset_json(
        "1.19.50",
        r#""identifier": "ex:all", "inherit_from": "minecraft:follow_orbit",
        "pos_x": 1, "pos_y": 2.5, "pos_z": -3, "rot_x": -45, "rot_y": 720,
        "rotation_speed": 180, "snap_to_target": true,
        "horizontal_rotation_limit": [10, 20], "vertical_rotation_limit": [30, 60],
        "continue_targeting": false, "tracking_radius": 25,
        "view_offset": [1, -1], "entity_offset": [0.5, 1.5, -2], "radius": 12,
        "yaw_limits_min": -90, "yaw_limits_max": 90, "listener": "player",
        "player_effects": true,
        "aim_assist": {"preset": "ex:aa", "target_mode": "distance", "angle": [40, 30], "distance": 8},
        "control_scheme": "camera_relative_strafe""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let mut expected = CameraPreset::new(CameraPresetId::parse("ex:all"));
    expected.inherit_from = Some(CameraPresetId::parse("minecraft:follow_orbit"));
    expected.pos_x = Some(f(1.0));
    expected.pos_y = Some(f(2.5));
    expected.pos_z = Some(f(-3.0));
    expected.rot_x = Some(f(-45.0));
    expected.rot_y = Some(f(720.0));
    expected.rotation_speed = Some(f(180.0));
    expected.snap_to_target = Some(true);
    expected.horizontal_rotation_limit = Some(pair(10.0, 20.0));
    expected.vertical_rotation_limit = Some(pair(30.0, 60.0));
    expected.continue_targeting = Some(false);
    expected.tracking_radius = Some(f(25.0));
    expected.view_offset = Some(pair(1.0, -1.0));
    expected.entity_offset = Some([0.5, 1.5, -2.0]);
    expected.radius = Some(f(12.0));
    expected.yaw_limits_min = Some(f(-90.0));
    expected.yaw_limits_max = Some(f(90.0));
    expected.listener = Some(AudioListener::Player);
    expected.player_effects = Some(true);
    expected.aim_assist = Some(PresetAimAssist {
        preset: Some(AimAssistPresetId::parse("ex:aa")),
        target_mode: Some(TargetMode::Distance),
        angle: Some(pair(40.0, 30.0)),
        distance: Some(f(8.0)),
    });
    expected.control_scheme = Some(ControlScheme::CameraRelativeStrafe);
    expected.apply_inherited_starting_rotation = true;
    assert_eq!(document.value(), &expected);
}

#[test]
fn cam06_every_enum_name_is_read() {
    for (name, scheme) in [
        (
            "locked_player_relative_strafe",
            ControlScheme::LockedPlayerRelativeStrafe,
        ),
        ("camera_relative", ControlScheme::CameraRelative),
        (
            "camera_relative_strafe",
            ControlScheme::CameraRelativeStrafe,
        ),
        ("player_relative", ControlScheme::PlayerRelative),
        (
            "player_relative_strafe",
            ControlScheme::PlayerRelativeStrafe,
        ),
    ] {
        let document = loads(preset(&preset_with(
            "1.19.50",
            &format!(r#""control_scheme": "{name}""#),
        )));
        assert_eq!(document.control_scheme, Some(scheme));
    }
    let document = loads(preset(&preset_with(
        "1.19.50",
        r#""listener": "camera", "aim_assist": {"target_mode": "angle"}"#,
    )));
    assert_eq!(document.listener, Some(AudioListener::Camera));
    assert_eq!(
        document.aim_assist.as_ref().unwrap().target_mode,
        Some(TargetMode::Angle)
    );
}

#[test]
fn rule_4a_a_name_outside_an_enum_is_an_invalid_enum_value() {
    for (members, path) in [
        (r#""listener": "speaker""#, "listener"),
        (r#""control_scheme": "tank""#, "control_scheme"),
        (
            r#""aim_assist": {"target_mode": "nearest"}"#,
            "aim_assist/target_mode",
        ),
    ] {
        let found = errors_of(members);
        assert_eq!(found.len(), 1, "{members}: {found:?}");
        assert_eq!(found[0].0, "InvalidEnumValue", "{members}");
        assert_eq!(found[0].1, format!("minecraft:camera_preset/{path}"));
    }
}

#[test]
fn rule_4a_a_preset_needs_a_namespaced_identifier() {
    const IDENTIFIER: &str = "Identifier requires a namespace and name separated by a ':' and each must have at least one alphanumeric or underscore character.";
    let d = rejects(preset(&preset_json("1.19.50", r#""pos_x": 1"#)));
    assert_eq!(
        errors(&d),
        [
            finding(
                "MissingRequiredField",
                "minecraft:camera_preset/identifier",
                "missing required field"
            ),
            failed_presets()
        ]
    );
    for (members, path) in [
        (r#""identifier": "nonamespace""#, "identifier"),
        (r#""identifier": "a:b:c""#, "identifier"),
        (r#""identifier": "ex:with-dash""#, "identifier"),
        (r#""identifier": ":name""#, "identifier"),
        (r#""identifier": "ex:""#, "identifier"),
        (
            r#""identifier": "ex:a", "inherit_from": "free""#,
            "inherit_from",
        ),
    ] {
        let d = rejects(preset(&preset_json("1.19.50", members)));
        assert_eq!(
            errors(&d),
            [
                finding(
                    "InvalidValue",
                    &format!("minecraft:camera_preset/{path}"),
                    IDENTIFIER
                ),
                failed_presets()
            ],
            "{members}"
        );
    }
    // `^(\w+:\w+)?$` takes the empty string, and so does vanilla.
    loads(preset(&preset_json(
        "1.19.50",
        r#""identifier": "Ex_1:Name_2", "inherit_from": """#,
    )));
}

#[test]
fn rule_4a_a_value_of_the_wrong_json_type_drops_the_preset() {
    for (members, path) in [
        (r#""radius": "5""#, "radius"),
        (r#""radius": null"#, "radius"),
        (r#""snap_to_target": "true""#, "snap_to_target"),
        (r#""snap_to_target": null"#, "snap_to_target"),
        (r#""pos_x": "1""#, "pos_x"),
        (r#""listener": 0"#, "listener"),
        (r#""listener": true"#, "listener"),
        (r#""control_scheme": 1"#, "control_scheme"),
        (r#""inherit_from": 1"#, "inherit_from"),
        (r#""view_offset": [1]"#, "view_offset"),
        (r#""view_offset": [1, 2, 3]"#, "view_offset"),
        (r#""view_offset": [1, "2"]"#, "view_offset/1"),
        (r#""entity_offset": [1, 2]"#, "entity_offset"),
        (r#""aim_assist": []"#, "aim_assist"),
        (r#""aim_assist": {"distance": "8"}"#, "aim_assist/distance"),
        (r#""aim_assist": {"preset": 1}"#, "aim_assist/preset"),
        (
            r#""aim_assist": {"target_mode": true}"#,
            "aim_assist/target_mode",
        ),
    ] {
        let found = errors_of(members);
        assert_eq!(found.len(), 1, "{members}: {found:?}");
        assert_eq!(found[0].0, "InvalidType", "{members}");
        assert_eq!(found[0].1, format!("minecraft:camera_preset/{path}"));
    }
}

/// Below 1.26.50 the game's legacy preset parser reads
/// with the non-strict schema reader: a bool member takes a
/// number (`0` is `false`, any other `true`) and a float member, the
/// elements of the float pairs and triples included, takes `true` / `false` as 1 / 0
/// (the game's float conversion). Each such place carries an information, since the write-back
/// writes the member's own type there.
#[test]
fn c2_below_1_26_50_a_bool_takes_a_number_and_a_float_takes_a_bool() {
    let members = r#""snap_to_target": 1, "continue_targeting": 0, "player_effects": 2.5,
        "pos_x": true, "pos_y": false, "pos_z": true, "rot_x": true, "rot_y": false,
        "rotation_speed": true, "tracking_radius": false, "radius": true,
        "yaw_limits_min": false, "yaw_limits_max": true,
        "horizontal_rotation_limit": [true, false], "vertical_rotation_limit": [false, true],
        "view_offset": [true, 2], "entity_offset": [false, true, 3],
        "aim_assist": {"angle": [true, false], "distance": true}"#;
    let coerced = [
        "snap_to_target",
        "continue_targeting",
        "player_effects",
        "pos_x",
        "pos_y",
        "pos_z",
        "rot_x",
        "rot_y",
        "rotation_speed",
        "tracking_radius",
        "radius",
        "yaw_limits_min",
        "yaw_limits_max",
        "horizontal_rotation_limit/0",
        "horizontal_rotation_limit/1",
        "vertical_rotation_limit/0",
        "vertical_rotation_limit/1",
        "view_offset/0",
        "entity_offset/0",
        "entity_offset/1",
        "aim_assist/angle/0",
        "aim_assist/angle/1",
        "aim_assist/distance",
    ];
    for version in ["1.19.50", "1.21.60", "1.26.49"] {
        let document = loads(preset(&preset_with(version, members)));
        assert_eq!(document.snap_to_target, Some(true), "{version}");
        assert_eq!(document.continue_targeting, Some(false));
        assert_eq!(document.player_effects, Some(true));
        assert_eq!(
            (document.pos_x, document.pos_y),
            (Some(f(1.0)), Some(f(0.0)))
        );
        assert_eq!(document.radius, Some(f(1.0)));
        assert_eq!(document.horizontal_rotation_limit, Some(pair(1.0, 0.0)));
        assert_eq!(document.vertical_rotation_limit, Some(pair(0.0, 1.0)));
        assert_eq!(document.view_offset, Some(pair(1.0, 2.0)));
        assert_eq!(document.entity_offset, Some([0.0, 1.0, 3.0]));
        let aim_assist = document.aim_assist.as_ref().unwrap();
        assert_eq!(
            (aim_assist.angle, aim_assist.distance),
            (Some(pair(1.0, 0.0)), Some(f(1.0)))
        );
        // No warning or error; one information at each coerced place, in document order.
        let d = document.diagnostics();
        assert!(
            d.iter().all(|f| f.severity == Severity::Info),
            "{version}: {d:#?}"
        );
        let places: Vec<String> = d
            .iter()
            .filter(|f| f.code.name() == "ValueRewritten")
            .map(|f| f.location.json_path.to_string())
            .collect();
        let expected: Vec<String> = coerced
            .iter()
            .map(|path| format!("minecraft:camera_preset/{path}"))
            .collect();
        assert_eq!(places, expected, "{version}");
        // Written in the members' own types; read back to the same preset, written the same.
        let written = document.to_json().unwrap();
        assert!(written.contains(r#""snap_to_target": true"#), "{written}");
        let again = loads(preset(&written));
        assert_eq!(again.value(), document.value());
        assert!(again.diagnostics().is_empty(), "{:#?}", again.diagnostics());
        assert_eq!(again.to_json().unwrap(), written);
    }
}

/// From 1.26.50 the strict slice reads presets: the same
/// values are type errors.
#[test]
fn c2_from_1_26_50_no_scalar_is_coerced() {
    for (members, path) in [
        (r#""snap_to_target": 1"#, "snap_to_target"),
        (r#""player_effects": 0"#, "player_effects"),
        (r#""pos_x": true"#, "pos_x"),
        (r#""view_offset": [true, 2]"#, "view_offset/0"),
        (r#""aim_assist": {"distance": true}"#, "aim_assist/distance"),
        (
            r#""aim_assist": {"angle": [1, false]}"#,
            "aim_assist/angle/1",
        ),
    ] {
        for version in ["1.26.50", "1.26.60"] {
            let d = rejects(preset(&preset_with(version, members)));
            let found = errors(&d);
            assert_eq!(found.len(), 2, "{version} {members}: {found:?}");
            assert_eq!(
                (found[0].0, found[0].1.as_str()),
                (
                    "InvalidType",
                    format!("minecraft:camera_preset/{path}").as_str()
                ),
                "{version} {members}"
            );
            assert_eq!(found[1], failed_presets());
        }
    }
}

/// A coerced value still meets the bind constraints: `radius: false` is 0, below 0.1.
#[test]
fn c2_a_coerced_value_is_checked_against_its_range() {
    let found = errors_of(r#""radius": false"#);
    assert_eq!(
        found,
        [finding(
            "InvalidValue",
            "minecraft:camera_preset/radius",
            &range("0.100000", "64")
        )]
    );
}

// A number no `f32` holds in an element of a fixed
// array is reported at that element, and its span is the element's text.
#[test]
fn a_bad_element_of_a_fixed_array_is_reported_at_its_index() {
    for (members, path, token) in [
        (r#""view_offset": [1, 1e40]"#, "view_offset/1", "1e40"),
        (r#""view_offset": [1e40, 1]"#, "view_offset/0", "1e40"),
        (
            r#""entity_offset": [0, 1, -1e40]"#,
            "entity_offset/2",
            "-1e40",
        ),
    ] {
        let text = preset_with("1.21.50", members);
        let d = rejects(preset(&text));
        let found: Vec<_> = d
            .at_least(Severity::Error)
            .filter(|f| f.code.name() != "DocumentRejected")
            .collect();
        assert_eq!(found.len(), 1, "{members}: {found:#?}");
        assert_eq!(
            found[0].location.json_path.to_string(),
            format!("minecraft:camera_preset/{path}"),
            "{members}"
        );
        let span = found[0].location.span.expect("a span");
        let start = span.offset as usize;
        assert_eq!(&text[start..start + span.len as usize], token, "{members}");
    }
}

#[test]
fn cam08_values_outside_their_ranges_drop_the_preset() {
    let at = |key: &str| format!("minecraft:camera_preset/{key}");
    for (members, path, text) in [
        (r#""rot_x": 90.5"#, at("rot_x"), range("-90", "90")),
        (r#""rot_x": -90.5"#, at("rot_x"), range("-90", "90")),
        (
            r#""rotation_speed": -0.5"#,
            at("rotation_speed"),
            range("0", "360"),
        ),
        (
            r#""rotation_speed": 360.5"#,
            at("rotation_speed"),
            range("0", "360"),
        ),
        (
            r#""view_offset": [64.5, 0]"#,
            at("view_offset/0"),
            range("-64", "64"),
        ),
        (
            r#""view_offset": [0, -64.5]"#,
            at("view_offset/1"),
            range("-64", "64"),
        ),
        (r#""radius": 0.09"#, at("radius"), range("0.100000", "64")),
        (r#""radius": 64.01"#, at("radius"), range("0.100000", "64")),
        (
            r#""yaw_limits_min": -180.5"#,
            at("yaw_limits_min"),
            range("-180", "179.900000"),
        ),
        (
            r#""yaw_limits_min": 180"#,
            at("yaw_limits_min"),
            range("-180", "179.900000"),
        ),
        (
            r#""yaw_limits_max": -180"#,
            at("yaw_limits_max"),
            range("-179.900000", "180"),
        ),
        (
            r#""yaw_limits_max": 180.5"#,
            at("yaw_limits_max"),
            range("-179.900000", "180"),
        ),
    ] {
        assert_eq!(
            errors_of(members),
            [finding("InvalidValue", &path, &text)],
            "{members}"
        );
    }
}

#[test]
fn cam08_the_ends_of_every_range_load() {
    for members in [
        r#""rot_x": 90"#,
        r#""rot_x": -90"#,
        r#""rotation_speed": 0"#,
        r#""rotation_speed": 360"#,
        r#""view_offset": [64, -64]"#,
        r#""radius": 0.1"#,
        r#""radius": 64"#,
        r#""yaw_limits_min": -180"#,
        r#""yaw_limits_min": 179.9"#,
        r#""yaw_limits_max": -179.9"#,
        r#""yaw_limits_max": 180"#,
        // `rot_y` and `tracking_radius` have no constraint.
        r#""rot_y": 100000, "tracking_radius": -5"#,
    ] {
        accepted(members);
    }
}

#[test]
fn cam12_starting_rot_loads_at_1_26_50_only() {
    let document = loads(preset(&preset_with(
        "1.26.50",
        r#""starting_rot": [10, 20]"#,
    )));
    assert_eq!(document.starting_rot, Some(pair(10.0, 20.0)));
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 50));
    assert!(!document.version().upgraded);
    // The copy step leaves it unset.
    let older = loads(preset(&preset_with("1.26.49", r#""pos_x": 1"#)));
    assert_eq!(older.starting_rot, None);
    assert_eq!(older.version().parsed_as, FormatVersion::new(1, 19, 50));
    // Below 1.26.50 it is an unknown member, as in vanilla's own struct.
    let d = rejects(preset(&preset_with(
        "1.26.49",
        r#""starting_rot": [10, 20]"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "UnrecognizedField",
            "minecraft:camera_preset/starting_rot",
            UNRECOGNIZED
        )]
    );
    let d = rejects(preset(&preset_with("1.26.50", r#""starting_rot": [10]"#)));
    assert_eq!(errors(&d)[0].0, "InvalidType");
}

/// The 1.26.50 preset schema binds `starting_rot`
/// only; "apply inherited starting rotation" is a member of the network packet, not a JSON key
/// (the string is not in the game). A preset that writes it is dropped at every version with
/// "Unrecognized field on camera preset!" and nothing else.
#[test]
fn cam12_apply_inherited_starting_rotation_is_an_unknown_member_at_every_version() {
    for version in ["1.19.50", "1.26.49", "1.26.50", "1.26.60"] {
        let d = rejects(preset(&preset_with(
            version,
            r#""apply_inherited_starting_rotation": true"#,
        )));
        assert_eq!(
            errors(&d),
            [finding(
                "UnrecognizedField",
                "minecraft:camera_preset/apply_inherited_starting_rotation",
                UNRECOGNIZED
            )],
            "{version}"
        );
    }
    // An upgraded preset applies the inherited starting rotation (the step
    // sets the flag), up to the last version below the 1.26.50 slice.
    for version in ["1.19.50", "1.26.49"] {
        let document = loads(preset(&preset_with(version, r#""pos_x": 1"#)));
        assert!(document.version().upgraded, "{version}");
        assert!(document.applies_inherited_starting_rotation(), "{version}");
    }
    // A preset read at 1.26.50 does not (the constructor's zero fill).
    let document = loads(preset(&preset_with("1.26.50", r#""pos_x": 1"#)));
    assert!(!document.applies_inherited_starting_rotation());
    assert!(!document.apply_inherited_starting_rotation);
}

#[test]
fn cam13_the_docs_starting_rot_x_and_y_are_unknown() {
    let d = rejects(preset(&preset_json(
        "1.19.50",
        r#""identifier": "minecraft:follow_orbit", "starting_rot_x": 10, "starting_rot_y": 10"#,
    )));
    assert_eq!(
        errors(&d),
        [
            finding(
                "UnrecognizedField",
                "minecraft:camera_preset/starting_rot_x",
                UNRECOGNIZED
            ),
            finding(
                "UnrecognizedField",
                "minecraft:camera_preset/starting_rot_y",
                UNRECOGNIZED
            )
        ]
    );
}

#[test]
fn cam14_the_vertical_limit_is_ordered_not_summed() {
    let at = "minecraft:camera_preset/vertical_rotation_limit";
    assert_eq!(
        errors_of(r#""vertical_rotation_limit": [90, 90]"#),
        [finding(
            "InvalidValue",
            at,
            "Vector's elements cannot equal each other!"
        )]
    );
    assert_eq!(
        errors_of(r#""vertical_rotation_limit": [120, 45]"#),
        [finding(
            "InvalidValue",
            at,
            "Vector's x must be less than y!"
        )]
    );
    assert_eq!(
        errors_of(r#""vertical_rotation_limit": [0, 190]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/1"),
            &range("0", "180")
        )]
    );
    assert_eq!(
        errors_of(r#""vertical_rotation_limit": [-1, 90]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/0"),
            &range("0", "180")
        )]
    );
    // 45 + 120 is 165, and 100 + 180 is 280: there is no sum rule.
    accepted(r#""vertical_rotation_limit": [45, 120]"#);
    accepted(r#""vertical_rotation_limit": [100, 180]"#);
    accepted(r#""vertical_rotation_limit": [0, 0.5]"#);
}

#[test]
fn cam15_the_horizontal_limit_sums_to_at_most_360() {
    let at = "minecraft:camera_preset/horizontal_rotation_limit";
    let total = "Vector elements added together cannot be greater than the defined constraint!";
    assert_eq!(
        errors_of(r#""horizontal_rotation_limit": [200, 200]"#),
        [finding("InvalidValue", at, total)]
    );
    assert_eq!(
        errors_of(r#""horizontal_rotation_limit": [0.5, 360]"#),
        [finding("InvalidValue", at, total)]
    );
    assert_eq!(
        errors_of(r#""horizontal_rotation_limit": [180.125, 180]"#),
        [finding("InvalidValue", at, total)]
    );
    assert_eq!(
        errors_of(r#""horizontal_rotation_limit": [361, 0]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/0"),
            &range("0", "360")
        )]
    );
    assert_eq!(
        errors_of(r#""horizontal_rotation_limit": [0, -1]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/1"),
            &range("0", "360")
        )]
    );
    accepted(r#""horizontal_rotation_limit": [45, 30]"#);
    accepted(r#""horizontal_rotation_limit": [0, 360]"#);
    // Equal components and a descending pair are the vertical limit's rules only.
    accepted(r#""horizontal_rotation_limit": [90, 90]"#);
    accepted(r#""horizontal_rotation_limit": [120, 45]"#);
}

#[test]
fn cam16_each_entity_offset_element_is_in_range() {
    let at = "minecraft:camera_preset/entity_offset";
    assert_eq!(
        errors_of(r#""entity_offset": [0, 65, 0]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/1"),
            &range("-64", "64")
        )]
    );
    assert_eq!(
        errors_of(r#""entity_offset": [-64.5, 0, 0]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/0"),
            &range("-64", "64")
        )]
    );
    assert_eq!(
        errors_of(r#""entity_offset": [0, 0, 64.5]"#),
        [finding(
            "InvalidValue",
            &format!("{at}/2"),
            &range("-64", "64")
        )]
    );
    accepted(r#""entity_offset": [0, 5, 0]"#);
    accepted(r#""entity_offset": [-64, 64, 64]"#);
}

/// The game logs the line at level 3 while setting up presets and goes on with
/// the next preset, the preset not set up and the document kept in
/// `CameraPresets`: a kept error.
#[test]
fn cam20_a_custom_preset_without_inherit_from_is_kept_with_a_kept_error_deviation() {
    const TEXT: &str =
        "Could not set up Camera Preset with name 'ex:a'. Are you missing 'inherit_from'?";
    for members in [
        r#""identifier": "ex:a""#,
        r#""identifier": "ex:a", "inherit_from": """#,
    ] {
        let document = loads(preset(&preset_json("1.19.50", members)));
        assert!(document.diagnostics().iter().all(|f| f.kept), "{members}");
        assert_eq!(
            errors(document.diagnostics()),
            [finding(
                "MissingRequiredField",
                "minecraft:camera_preset/inherit_from",
                TEXT
            )],
            "{members}"
        );
    }
    // With a parent, or named like a vanilla camera, nothing is missing.
    let document = loads(preset(&preset_with("1.19.50", r#""pos_x": 1"#)));
    assert!(document.diagnostics().is_empty());
    for name in [
        "minecraft:free",
        "minecraft:follow_orbit",
        "minecraft:fixed_boom",
        "minecraft:third_person",
        "minecraft:third_person_front",
        "minecraft:first_person",
        "minecraft:death_camera",
    ] {
        let document = loads(preset(&preset_json(
            "1.19.50",
            &format!(r#""identifier": "{name}""#),
        )));
        assert!(document.diagnostics().is_empty(), "{name}");
    }
}

/// New in 1.26.60: the game logs
/// "Preset %s cannot change the starting rotation of a vanilla camera" (level 3) for a preset
/// named like a vanilla camera that sets `starting_rot`, ignores the value and keeps the
/// preset. The log needs a camera entity with a third-person boom, which none of the four
/// vanilla cameras has, so the game is silent for real data: ours at load, an
/// information.
#[test]
fn p10_a_vanilla_camera_s_starting_rotation_is_kept_with_an_information_deviation() {
    for name in VANILLA_CAMERAS {
        let document = loads(preset(&preset_json(
            "1.26.50",
            &format!(r#""identifier": "{name}", "starting_rot": [10, 20]"#),
        )));
        assert_eq!(
            infos(document.diagnostics()),
            [finding(
                "InvalidValue",
                "minecraft:camera_preset/starting_rot",
                &format!("Preset {name} cannot change the starting rotation of a vanilla camera")
            )],
            "{name}"
        );
        assert_eq!(document.starting_rot, Some(pair(10.0, 20.0)));
    }
    // Not a vanilla camera, or no `starting_rot`: no finding.
    for members in [
        r#""identifier": "ex:a", "inherit_from": "minecraft:follow_orbit", "starting_rot": [10, 20]"#,
        r#""identifier": "minecraft:free", "starting_rot": [10, 20]"#,
        r#""identifier": "minecraft:third_person", "pos_x": 1"#,
    ] {
        let document = loads(preset(&preset_json("1.26.50", members)));
        assert!(
            document.diagnostics().is_empty(),
            "{members}: {:#?}",
            document.diagnostics()
        );
    }
}

/// The game logs the line at level 3 when applying preset values and goes on
/// with the preset: a kept error.
#[test]
fn cam21_yaw_limits_out_of_order_are_kept_with_a_kept_error_deviation() {
    const TEXT: &str = "Yaw limit min must be smaller than the yaw limit's max for preset ex:a!";
    let document = loads(preset(&preset_with(
        "1.19.50",
        r#""yaw_limits_min": 10, "yaw_limits_max": -10"#,
    )));
    assert_eq!(
        errors(document.diagnostics()),
        [finding(
            "InvalidValue",
            "minecraft:camera_preset/yaw_limits_min",
            TEXT
        )]
    );
    assert!(document.diagnostics().iter().all(|f| f.kept));
    assert_eq!(document.identifier.as_str(), "ex:a");
    for limits in [
        r#""yaw_limits_min": -10, "yaw_limits_max": 10"#,
        r#""yaw_limits_min": 10"#,
        r#""yaw_limits_max": -10"#,
    ] {
        let document = loads(preset(&preset_with("1.19.50", limits)));
        assert!(document.diagnostics().is_empty(), "{limits}");
    }
}

/// The game logs only when the limit is exceeded: equal
/// limits say nothing.
#[test]
fn equal_yaw_limits_say_nothing() {
    let document = loads(preset(&preset_with(
        "1.19.50",
        r#""yaw_limits_min": 5, "yaw_limits_max": 5"#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn a_duplicate_member_keeps_its_last_value_in_the_check_and_in_the_struct() {
    // The last of two members of one key is the member (the envelope's rule), for the shape check, the constraints and the typed value alike.
    let document = loads(preset(&preset_with(
        "1.21.60",
        r#""radius": 100, "radius": 5, "listener": 7, "listener": "player""#,
    )));
    assert_eq!(document.radius, Some(f(5.0)));
    assert_eq!(document.listener, Some(AudioListener::Player));
    // The last value is the one checked: a good first value does not save a bad last one.
    assert_eq!(
        errors_of(r#""radius": 5, "radius": 100"#),
        [finding(
            "InvalidValue",
            "minecraft:camera_preset/radius",
            &range("0.100000", "64")
        )]
    );
    let found = errors_of(r#""radius": 5, "radius": "far""#);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "InvalidType");
    // An unknown member written twice is one unknown member.
    let d = rejects(preset(&preset_with("1.21.60", r#""zoom": 1, "zoom": 2"#)));
    assert_eq!(errors(&d).len(), 1, "{d:#?}");
    // Two payloads at the root: the last one is the preset.
    let text = r#"{"format_version": "1.21.60",
        "minecraft:camera_preset": {"identifier": "ex:first", "bogus": 1},
        "minecraft:camera_preset": {"identifier": "ex:last", "inherit_from": "minecraft:free"}}"#;
    let document = loads(preset(text));
    assert_eq!(document.identifier.as_str(), "ex:last");
    let text = r#"{"format_version": "1.21.60",
        "minecraft:camera_preset": {"identifier": "ex:first", "inherit_from": "minecraft:free"},
        "minecraft:camera_preset": {"identifier": "ex:last", "bogus": 1}}"#;
    let d = rejects(preset(text));
    assert_eq!(
        errors(&d),
        [finding(
            "UnrecognizedField",
            "minecraft:camera_preset/bogus",
            UNRECOGNIZED
        )]
    );
}

#[test]
fn an_error_only_the_typed_parse_finds_is_reported_the_same_beside_a_duplicate_member() {
    // The second read's own failure is the finding, with the code, path and message
    // the document gives without the duplicate, and a span on the failing value in the file.
    for (duplicate, bad, code, path, written) in [
        (
            r#""radius": 1, "radius": 5"#,
            r#""listener": "nobody""#,
            "InvalidEnumValue",
            "listener",
            r#""nobody""#,
        ),
        (
            r#""radius": 1, "radius": 5"#,
            r#""aim_assist": {"target_mode": "nearest"}"#,
            "InvalidEnumValue",
            "aim_assist/target_mode",
            r#""nearest""#,
        ),
        (
            r#""rot_y": 1, "rot_y": 2"#,
            r#""pos_x": 1e40"#,
            "InvalidType",
            "pos_x",
            "1e40",
        ),
        (
            r#""pos_y": 0, "pos_y": 0"#,
            r#""control_scheme": "tank""#,
            "InvalidEnumValue",
            "control_scheme",
            r#""tank""#,
        ),
    ] {
        for version in ["1.21.60", "1.26.50"] {
            let plain = rejects(preset(&preset_with(version, bad)));
            for members in [format!("{duplicate}, {bad}"), format!("{bad}, {duplicate}")] {
                let text = preset_with(version, &members);
                let doubled = rejects(preset(&text));
                assert_eq!(errors(&doubled), errors(&plain), "{members} at {version}");
                let first = doubled.iter().next().unwrap();
                assert_eq!(first.code.name(), code, "{members}");
                assert_eq!(
                    first.location.json_path.to_string(),
                    format!("minecraft:camera_preset/{path}")
                );
                let span = first.location.span.expect("a span in the file");
                let (start, end) = (span.offset as usize, (span.offset + span.len) as usize);
                assert_eq!(&text[start..end], written, "{members}");
            }
        }
    }
    // A duplicate inside the object that holds the bad member, and the bad member itself
    // written twice: the last value is the one judged.
    let text = preset_with(
        "1.21.60",
        r#""aim_assist": {"distance": 1, "distance": 2, "target_mode": "angle", "target_mode": "nearest"}"#,
    );
    let d = rejects(preset(&text));
    let first = d.iter().next().unwrap();
    assert_eq!(first.code.name(), "InvalidEnumValue");
    assert_eq!(
        first.location.json_path.to_string(),
        "minecraft:camera_preset/aim_assist/target_mode"
    );
    let span = first.location.span.unwrap();
    assert_eq!(
        &text[span.offset as usize..(span.offset + span.len) as usize],
        r#""nearest""#
    );
    loads(preset(&preset_with(
        "1.21.60",
        r#""aim_assist": {"target_mode": "nearest", "target_mode": "angle"}"#,
    )));
}

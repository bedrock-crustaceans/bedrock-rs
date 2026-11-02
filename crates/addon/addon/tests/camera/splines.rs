//! The spline rows: cam11, cam22 (the texts) and cam25, with validation.md rule 6.

use bedrock_addon::document::Content;
use bedrock_addon::families::camera::{Easing, SplineType};

use super::common::*;

const FAILED: &str = "Custom camera splines not loaded correctly.";

/// The errors of a spline file holding one spline `ex:s` with `members`; the closing line is
/// checked and left out.
#[track_caller]
fn errors_of(name: &str, members: &str) -> Vec<(&'static str, String, String)> {
    let d = rejects(spline(&splines_json(&spline_json(name, members))));
    let mut found = errors(&d);
    assert_eq!(
        found.pop(),
        Some(finding("DocumentRejected", "", FAILED)),
        "{d:#?}"
    );
    found
}

fn at(path: &str) -> String {
    format!("minecraft:camera_custom_splines/splines/0/{path}")
}

#[test]
fn cam11_spline_names_types_and_times_are_constrained() {
    for name in ["nonamespace", "a:b:c", ":s", "ex:", "ex:with-dash", ""] {
        assert_eq!(
            errors_of(name, ""),
            [finding(
                "InvalidValue",
                &at("name"),
                "\"name\" must be in \"namespace:name\" format."
            )],
            "{name}"
        );
    }
    for spline_type in ["bezier", "Linear", "", "linear "] {
        assert_eq!(
            errors_of("ex:s", &format!(r#""spline_type": "{spline_type}""#)),
            [finding(
                "InvalidValue",
                &at("spline_type"),
                "\"spline_type\" must be either \"catmullrom\" or \"linear\"."
            )],
            "{spline_type}"
        );
    }
    assert_eq!(
        errors_of(
            "ex:s",
            r#""progress_key_frames": [{"progress": 0.5, "time": 0}, {"progress": 1, "time": -0.5}]"#
        ),
        [finding(
            "InvalidValue",
            &at("progress_key_frames/1/time"),
            "Time cannot be a negative value."
        )]
    );
    let document = loads(spline(&splines_json(&spline_json(
        "Ex_1:S_2",
        r#""spline_type": "linear", "total_time": 0"#,
    ))));
    assert!(document.diagnostics().is_empty());
    let first = &document.splines[0];
    assert_eq!(first.name.0, "Ex_1:S_2");
    assert_eq!(first.spline_type, Some(SplineType::Linear));
    assert_eq!(first.total_time.0, 0.0);
    assert_eq!(first.control_points.len(), 2);
    assert_eq!(
        first.control_points[1].position.map(|v| v.0),
        [1.0, 2.0, 3.0]
    );
}

#[test]
fn cam22_each_spline_constraint_has_its_own_text() {
    assert_eq!(
        errors_of("ex:s", r#""total_time": -1"#),
        [finding(
            "InvalidValue",
            &at("total_time"),
            "Total time cannot be a negative value."
        )]
    );
    for progress in ["1.5", "-0.5"] {
        assert_eq!(
            errors_of(
                "ex:s",
                &format!(r#""progress_key_frames": [{{"progress": {progress}, "time": 0}}]"#)
            ),
            [finding(
                "InvalidValue",
                &at("progress_key_frames/0/progress"),
                "The progress values must be between 0.0 and 1.0."
            )],
            "{progress}"
        );
    }
    // The rotation key frame's text has no final period.
    assert_eq!(
        errors_of(
            "ex:s",
            r#""rotation_key_frames": [{"rotation": [0, 0, 0], "time": -1}]"#
        ),
        [finding(
            "InvalidValue",
            &at("rotation_key_frames/0/time"),
            "Time cannot be a negative value"
        )]
    );
    // The ends of each range load.
    loads(spline(&splines_json(&spline_json(
        "ex:s",
        r#""total_time": 0, "progress_key_frames": [{"progress": 0, "time": 0}, {"progress": 1, "time": 0}], "rotation_key_frames": [{"rotation": [0, 0, 0], "time": 0}]"#,
    ))));
}

#[test]
fn rule_6_every_required_spline_member_is_required() {
    let missing = |path: &str| finding("MissingRequiredField", &at(path), "missing required field");
    let d = rejects(spline(
        r#"{"format_version": "1.26.0", "minecraft:camera_custom_splines": {}}"#,
    ));
    assert_eq!(
        errors(&d)[0],
        finding(
            "MissingRequiredField",
            "minecraft:camera_custom_splines/splines",
            "missing required field"
        )
    );
    let d = rejects(spline(&splines_json(r#"{"spline_type": "linear"}"#)));
    let mut found = errors(&d);
    found.pop();
    assert_eq!(
        found,
        [
            missing("name"),
            missing("total_time"),
            missing("control_points"),
            missing("progress_key_frames"),
            missing("rotation_key_frames"),
        ]
    );
    assert_eq!(
        errors_of("ex:s", r#""control_points": [{}]"#),
        [missing("control_points/0/position")]
    );
    assert_eq!(
        errors_of("ex:s", r#""progress_key_frames": [{}]"#),
        [
            missing("progress_key_frames/0/progress"),
            missing("progress_key_frames/0/time")
        ]
    );
    assert_eq!(
        errors_of("ex:s", r#""rotation_key_frames": [{}]"#),
        [
            missing("rotation_key_frames/0/rotation"),
            missing("rotation_key_frames/0/time")
        ]
    );
    // An unknown member fails the file under the strict mask, with cereal's text.
    assert_eq!(
        errors_of("ex:s", r#""loop": true"#),
        [finding(
            "UnrecognizedField",
            &at("loop"),
            "this member was found in the input, but is not present in the Schema"
        )]
    );
    let found = errors_of("ex:s", r#""control_points": [{"position": [0, 0]}]"#);
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("InvalidType", at("control_points/0/position").as_str())
    );
}

#[test]
fn cam25_easing_is_one_of_the_32_names() {
    let document = loads(spline(&splines_json(&spline_json(
        "ex:s",
        r#""progress_key_frames": [{"progress": 0.5, "time": 1, "easing": "in_out_back"}], "rotation_key_frames": [{"rotation": [0, 90, 0], "time": 1, "easing": "spring"}]"#,
    ))));
    let first = &document.splines[0];
    assert_eq!(first.progress_key_frames[0].easing, Some(Easing::InOutBack));
    assert_eq!(first.rotation_key_frames[0].easing, Some(Easing::Spring));
    assert_eq!(
        first.rotation_key_frames[0].rotation.map(|v| v.0),
        [0.0, 90.0, 0.0]
    );
    // Without `spline_type` the spline is a Catmull-Rom curve; the member stays unwritten.
    assert_eq!(first.spline_type, None);
    assert_eq!(first.spline_type(), SplineType::Catmullrom);
    assert_eq!(first.progress_key_frames[0].progress.0, 0.5);

    for (frames, path) in [
        (
            "progress_key_frames",
            r#"{"progress": 0.5, "time": 1, "easing": "ease_in"}"#,
        ),
        (
            "rotation_key_frames",
            r#"{"rotation": [0, 0, 0], "time": 1, "easing": "ease_in"}"#,
        ),
    ] {
        let found = errors_of("ex:s", &format!(r#""{frames}": [{path}]"#));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            (found[0].0, found[0].1.clone()),
            ("InvalidEnumValue", at(&format!("{frames}/0/easing")))
        );
    }

    // Every name of vanilla's table reads, in both key frame kinds.
    assert_eq!(Easing::NAMES.len(), 32);
    for name in Easing::NAMES {
        let document = loads(spline(&splines_json(&spline_json(
            "ex:s",
            &format!(
                r#""progress_key_frames": [{{"progress": 0, "time": 0, "easing": "{name}"}}]"#
            ),
        ))));
        let easing = document.splines[0].progress_key_frames[0].easing.unwrap();
        let written = document.to_json().unwrap();
        assert!(
            written.contains(&format!(r#""easing": "{name}""#)),
            "{name}: {easing:?}"
        );
    }
}

#[test]
fn every_member_of_a_spline_file_and_a_list_reads_into_its_field() {
    use bedrock_addon::families::camera::{
        CameraList, CameraSpline, ControlPoint, ProgressKeyFrame, RotationKeyFrame, Spline,
    };
    use bedrock_addon::id::{CameraListId, CameraPresetId};
    use bedrock_addon::json::{StrictFloat, StrictString};

    // The member table and the struct agree key for key: a document with every member equals
    // the value built by hand.
    let f = StrictFloat::<f32>;
    let document = loads(spline(&splines_json(
        r#"{"name": "ex:all", "total_time": 6.5, "spline_type": "linear",
        "control_points": [{"position": [1, 2, 3]}, {"position": [-4, 5.5, 6]}],
        "progress_key_frames": [{"progress": 0.25, "time": 1.5, "easing": "out_expo"}, {"progress": 1, "time": 6.5}],
        "rotation_key_frames": [{"rotation": [10, 20, 30], "time": 2, "easing": "in_back"}]}"#,
    )));
    assert_eq!(
        document.value(),
        &CameraSpline {
            splines: vec![Spline {
                name: StrictString("ex:all".to_owned()),
                total_time: f(6.5),
                spline_type: Some(SplineType::Linear),
                control_points: vec![
                    ControlPoint {
                        position: [f(1.0), f(2.0), f(3.0)]
                    },
                    ControlPoint {
                        position: [f(-4.0), f(5.5), f(6.0)]
                    },
                ],
                progress_key_frames: vec![
                    ProgressKeyFrame {
                        progress: f(0.25),
                        time: f(1.5),
                        easing: Some(Easing::OutExpo),
                    },
                    ProgressKeyFrame {
                        progress: f(1.0),
                        time: f(6.5),
                        easing: None,
                    },
                ],
                rotation_key_frames: vec![RotationKeyFrame {
                    rotation: [f(10.0), f(20.0), f(30.0)],
                    time: f(2.0),
                    easing: Some(Easing::InBack),
                }],
            }],
        }
    );
    let document = loads(list(&list_json(
        "1.21.60",
        "ex:all",
        r#"["minecraft:free", "ex:b"]"#,
    )));
    assert_eq!(
        document.value(),
        &CameraList::new(
            CameraListId::parse("ex:all"),
            vec![
                CameraPresetId::parse("minecraft:free"),
                CameraPresetId::parse("ex:b")
            ]
        )
    );
}

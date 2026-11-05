//! The controller chain `v1_8_0 → AnimationControllerFile`: its versions, range selection and
//! what the step carries.

use bedrock_addon::families::animation_controller::AnimationControllerFile;
use bedrock_addon::families::animation_controller::history::{Any, v1_8_0};
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::Versioned;

#[test]
fn the_controller_chain_is_contiguous() {
    assert_eq!(
        <v1_8_0::AnimationControllerFile as Versioned>::VERSION,
        FormatVersion::new(1, 8, 0)
    );
    assert_eq!(
        <AnimationControllerFile as Versioned>::VERSION,
        FormatVersion::new(1, 10, 0)
    );
}

#[test]
fn range_selection_floor_and_ceiling() {
    use super::common::{bp, controller, rejected};
    let cx = bp(1, 16, 0);
    let file = |v: &str| {
        format!(
            r#"{{"format_version": "{v}", "animation_controllers": {{"controller.animation.x": {{"states": {{"a": {{}}}}}}}}}}"#
        )
    };
    let as_written = |v: &str| {
        AnimationControllerFile::parse_as_written(
            &bedrock_addon::storage::PackPath::from_static("animation_controllers/x.json"),
            file(v).as_bytes(),
            &cx,
        )
        .unwrap()
        .into_value()
    };
    assert!(matches!(as_written("1.8.0"), Any::V1_8_0(_)));
    assert!(matches!(as_written("1.9.9"), Any::V1_8_0(_)));
    assert!(matches!(as_written("1.10.0-beta.1"), Any::V1_8_0(_)));
    assert!(matches!(as_written("1.10.0"), Any::V1_10_0(_)));
    assert!(matches!(as_written("1.26.50"), Any::V1_10_0(_)));
    rejected(controller(&file("1.7.9"), &cx));
    rejected(controller(&file("1.26.61"), &cx));
    let upgraded = bedrock_addon::families::animation_controller::history::upgrade(
        AnimationControllerFile::parse_as_written(
            &bedrock_addon::storage::PackPath::from_static("animation_controllers/x.json"),
            file("1.8.0").as_bytes(),
            &cx,
        )
        .unwrap(),
        &cx,
    )
    .unwrap();
    assert!(upgraded.version().upgraded);
}

/// A 1.8.0 particle event's `pre_effect_script` is carried through the upgrade as Molang text.
#[test]
fn a_1_8_particle_effects_script_is_carried_as_molang_text() {
    use super::common::{controller, loaded, rp};
    let text = r#"{"format_version": "1.8.0", "animation_controllers": {"controller.animation.pig.move": {"initial_state": "default", "states": {"default": {"parameters": ["query.modified_move_speed", "query.is_baby"], "animations": [{"walk": [{"0.0": 0.0, "1.0": 1.0}, {"0.0": 1.0, "1.0": 0.5}]}], "transitions": [{"idle": "!query.is_moving"}], "particle_effects": [{"effect": "dust", "pre_effect_script": "v.x = 1;"}]}, "idle": {"animations": [{"look": []}]}}}}}"#;
    let file = loaded(controller(text, &rp(1, 16, 0)));
    assert!(file.version().upgraded);
    let state = &file.animation_controllers["controller.animation.pig.move"].states["default"];
    let effects = state.particle_effects.as_ref().expect("particle effects");
    let script = effects[0].pre_effect_script.as_ref().expect("the script");
    assert_eq!(script.source().expect("Molang text").as_str(), "v.x = 1;");
}

/// Every finding of reading a file as written and of its upgrade names no pack (the file was
/// read on its own) and the file.
#[test]
fn every_finding_of_the_reading_and_the_step_carries_its_pack_and_path() {
    use bedrock_addon::diagnostic::Diagnostics;
    use bedrock_addon::families::animation_controller::history::upgrade;
    use bedrock_addon::pack::PackRef;
    use bedrock_addon::storage::PackPath;
    let cx = super::common::rp(1, 16, 0);
    let path = PackPath::from_static("animation_controllers/x.json");
    let check = |found: &Diagnostics| {
        assert!(!found.is_empty());
        for finding in found {
            assert_eq!(
                finding.location.pack,
                Some(PackRef::app_data()),
                "{finding:#?}"
            );
            assert_eq!(finding.location.path.as_ref(), Some(&path), "{finding:#?}");
        }
    };
    // Not JSON: rejected while read.
    let rejected = AnimationControllerFile::parse_as_written(&path, b"{", &cx).unwrap_err();
    check(&rejected.diagnostics);
    // Members in another case: an information each while read.
    let cased = r#"{"format_version": "1.10.0", "animation_controllers": {"Controller.Animation.Pig": {"States": {"a": {"Animations": ["x"]}}}}}"#;
    let read = AnimationControllerFile::parse_as_written(&path, cased.as_bytes(), &cx).unwrap();
    check(read.diagnostics());
    // An input no curve reads: an information of the step.
    let old = r#"{"format_version": "1.8.0", "animation_controllers": {"controller.animation.x": {"states": {"default": {"parameters": ["q.x", "q.y"]}}}}}"#;
    let read = AnimationControllerFile::parse_as_written(&path, old.as_bytes(), &cx).unwrap();
    assert!(read.diagnostics().is_empty(), "{:#?}", read.diagnostics());
    let upgraded = upgrade(read, &cx).unwrap();
    check(upgraded.diagnostics());
}

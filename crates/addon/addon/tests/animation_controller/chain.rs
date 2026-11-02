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

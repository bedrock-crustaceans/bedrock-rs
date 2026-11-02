//! Steps of the item chain run on hand-built inputs through the step runner: the legacy format
//! carried unchanged, the digger's side-car copy and the sound events' canonical names.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::testing::{Checks, Run, StepOutput, step_output};
use bedrock_addon_core::versioning::{Upgrade, VersionStruct};
use bedrock_addon_family_item::history;
use facet_value::VObject;

use super::common::allowlisted;

/// The JSON at `path` (`/`-separated) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/').try_fold(value, |v, key| {
        v.as_object()
            .and_then(|o| o.get(key))
            .or_else(|| key.parse::<usize>().ok().and_then(|i| v.as_array()?.get(i)))
    })
}

/// `text` as a JSON value.
fn json(text: &str) -> Value {
    facet_json::from_str(text).unwrap()
}

/// Runs `S`'s step on `input` with the allowlisted load context.
fn run<S>(input: &Value) -> StepOutput
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let checks = Checks {
        load: allowlisted(),
        identifier: "minecraft:x",
        ..Checks::default()
    };
    step_output::<S>(input, Run::default(), &checks)
        .unwrap_or_else(|failure| panic!("the step failed: {failure:#?}"))
}

#[test]
fn the_legacy_format_is_carried_to_the_next_struct_without_components() {
    let input = json(
        r#"{"description": {"identifier": "minecraft:camera", "category": "items"}, "components": {"minecraft:max_damage": 5, "minecraft:use_duration": 100000}}"#,
    );
    let out = run::<history::v1_10_0::Item>(&input);
    let components = at(&out.payload, "components").expect("no components");
    assert!(
        components.as_object().is_some_and(VObject::is_empty),
        "the components are not empty: {components:?}"
    );
    assert!(
        at(&out.payload, "description/category").is_none(),
        "description.category was not moved to legacy"
    );
    assert!(
        at(&out.payload, "description/menu_category").is_none(),
        "a menu_category was created"
    );
}

#[test]
fn a_digger_without_triggers_is_not_copied_to_the_side_car() {
    let plain = json(
        r#"{"description": {"identifier": "minecraft:x"}, "components": {"minecraft:digger": {"destroy_speeds": [{"block": "minecraft:dirt", "speed": 6}]}}}"#,
    );
    let out = run::<history::v1_26_0::Item>(&plain);
    assert!(
        at(
            &out.payload,
            "__deprecated_components/legacy_on_dig_trigger"
        )
        .is_none(),
        "a digger without triggers was copied"
    );
}

#[test]
fn sound_events_are_written_by_their_canonical_name() {
    let input = json(
        r#"{"description": {"identifier": "minecraft:x"}, "components": {"minecraft:use_modifiers": {"use_duration": 1.0, "start_sound": "HURT"}, "minecraft:swing_sounds": {"attack_hit": "attack.strong"}, "minecraft:durability_sensor": {"durability_thresholds": [{"durability": 5, "sound_event": "break"}]}}}"#,
    );
    let out = run::<history::v1_26_30::Item>(&input);
    for (path, name) in [
        ("components/minecraft:use_modifiers/start_sound", "hurt"),
        (
            "components/minecraft:swing_sounds/attack_hit",
            "attack.strong",
        ),
        (
            "components/minecraft:durability_sensor/durability_thresholds/0/sound_event",
            "break",
        ),
    ] {
        assert_eq!(at(&out.payload, path), Some(&Value::from(name)), "{path}");
    }
}

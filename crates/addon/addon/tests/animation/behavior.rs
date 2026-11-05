//! The behaviour-pack rows 1–19 (`min_engine_version` 1.16.0 unless the row says otherwise).
//! Findings are matched by code and severity, and by vanilla's text where the row gives it.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::animation::{EventString, LerpMode, LoopMode, OneOrMany};
use bedrock_addon::families::animation_controller::{AnimationControllerFile, StateAnimation};
use bedrock_addon::id::AnimationRef;
use bedrock_addon::molang::{MolangVersion, RawVersion};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{FormatVersion, MinEngineVersion};

use super::common::{
    animation, bp, controller, expect, expect_prefix, find, fixture, lines, loaded, rejected,
    round_trip, stacks,
};

const FAMILY: &str = "animation/bp";

/// A controller file of `version` with one controller `controller.animation.x` whose state
/// `default` is `state`.
fn ctrl(version: &str, state: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "animation_controllers": {{"controller.animation.x": {{"initial_state": "default", "states": {{"default": {state}, "b": {{}}}}}}}}}}"#
    )
}

/// An animation file of `version` with one animation `animation.x` of `body`.
fn anim(version: &str, body: &str) -> String {
    format!(r#"{{"format_version": "{version}", "animations": {{"animation.x": {body}}}}}"#)
}

/// A behaviour pack at 1.16.0 without the Molang compile check: the upgrade rows use the
/// folder's placeholder queries (`query.x`), which name no real query.
fn unchecked() -> bedrock_addon::load::LoadContext {
    #[cfg_attr(not(feature = "molang"), allow(unused_mut))]
    let mut cx = bp(1, 16, 0);
    #[cfg(feature = "molang")]
    {
        cx.molang = None;
    }
    cx
}

/// The written form of an upgraded `before` fixture equals the written form of `after`.
#[track_caller]
fn upgrade_pair(name: &str) {
    let cx = unchecked();
    let before = loaded(controller(
        &fixture(FAMILY, &format!("{name}.before.json")),
        &cx,
    ));
    let after = loaded(controller(
        &fixture(FAMILY, &format!("{name}.after.json")),
        &cx,
    ));
    assert!(before.version().upgraded);
    assert_eq!(before.to_json().unwrap(), after.to_json().unwrap());
}

#[test]
fn r01_up_1_8_curves_become_state_variables() {
    upgrade_pair("up_1_8_curves");
    let cx = unchecked();
    let file = loaded(controller(
        &fixture(FAMILY, "up_1_8_curves.before.json"),
        &cx,
    ));
    let state = &file.animation_controllers["controller.animation.x"].states["default"];
    let variable = &state.variables.as_ref().unwrap()["a_0"];
    assert_eq!(variable.input.source().unwrap().as_str(), "query.x");
    assert_eq!(variable.remap_curve.as_ref().unwrap().len(), 2);
    assert_eq!(file.format_version, FormatVersion::new(1, 10, 0));
}

#[test]
fn r01a_up_1_8_two_curves_join_their_variables() {
    upgrade_pair("up_1_8_two_curves");
}

#[test]
fn r01b_up_1_8_empty_curves_weigh_one() {
    upgrade_pair("up_1_8_empty_curves");
}

#[test]
fn r01c_up_1_8_string_entry_is_rejected() {
    let d = rejected(controller(
        &ctrl("1.8.0", r#"{"animations": ["a"]}"#),
        &bp(1, 16, 0),
    ));
    expect_prefix(
        &d,
        "InvalidType",
        Severity::Error,
        "unknown child schema option type.  Allowed types: ",
    );
}

/// The context version of the first transition of state `default`.
fn transition_version(file: &AnimationControllerFile) -> Option<RawVersion> {
    file.animation_controllers["controller.animation.x"].states["default"]
        .transitions
        .as_ref()?[0]
        .condition
        .raw_version()
}

/// The context version of variable `v`'s `input` in state `default`.
fn input_version(file: &AnimationControllerFile) -> Option<RawVersion> {
    file.animation_controllers["controller.animation.x"].states["default"]
        .variables
        .as_ref()?["v"]
        .input
        .source()?
        .raw_version()
}

fn molang_of(a: u16, b: u16, c: u16) -> RawVersion {
    RawVersion::from(MinEngineVersion::new(a, b, c).molang_version())
}

#[test]
fn r02_eff_below_fix_takes_the_file_version() {
    let text = ctrl(
        "1.10.0",
        r#"{"transitions": [{"b": "query.is_sneaking"}], "variables": {"v": {"input": "q.life_time"}}}"#,
    );
    let file = loaded(controller(&text, &bp(1, 16, 0)));
    assert_eq!(transition_version(&file), Some(molang_of(1, 10, 0)));
    assert_eq!(input_version(&file), Some(molang_of(1, 10, 0)));
    assert!(file.animation_controllers["controller.animation.x"].legacy_event_skipping);
}

#[test]
fn r02_typed_controller_members_take_the_effective_version_too() {
    // Just below the 1.20.50 fix the pack's own version differs from the file's.
    assert_ne!(molang_of(1, 20, 40), molang_of(1, 10, 0));
    let text = ctrl(
        "1.10.0",
        r#"{"variables": {"v": {"input": "q.life_time"}}}"#,
    );
    let file = loaded(controller(&text, &bp(1, 20, 40)));
    assert_eq!(input_version(&file), Some(molang_of(1, 10, 0)));
}

#[test]
fn r03_eff_at_fix_takes_the_pack_version_but_keeps_the_legacy_start() {
    let text = ctrl(
        "1.10.0",
        r#"{"transitions": [{"b": "query.is_sneaking"}], "variables": {"v": {"input": "q.life_time"}}}"#,
    );
    let file = loaded(controller(&text, &bp(1, 20, 50)));
    assert_eq!(transition_version(&file), Some(molang_of(1, 20, 50)));
    assert_eq!(input_version(&file), Some(molang_of(1, 20, 50)));
    assert_ne!(molang_of(1, 20, 50), molang_of(1, 10, 0));
    assert!(file.animation_controllers["controller.animation.x"].legacy_event_skipping);
}

#[test]
fn r04_lowercase_transitions_below_1_17_30() {
    let state = r#"{"transitions": [{"b": "q.get_name == 'Dinnerbone'"}], "blend_via_shortest_path": true}"#;
    let old = loaded(controller(&ctrl("1.17.20", state), &bp(1, 16, 0)));
    let new = loaded(controller(&ctrl("1.17.30", state), &bp(1, 16, 0)));
    let view = |file: &AnimationControllerFile| {
        let c = &file.animation_controllers["controller.animation.x"];
        let t = &c.states["default"].transitions.as_ref().unwrap()[0];
        (c.transition_expression(t), c.lowercase_transitions)
    };
    assert_eq!(view(&old), ("q.get_name == 'dinnerbone'".to_owned(), true));
    assert_eq!(view(&new), ("q.get_name == 'Dinnerbone'".to_owned(), false));
    for file in [&old, &new] {
        let state = &file.animation_controllers["controller.animation.x"].states["default"];
        assert_eq!(
            state.blend_via_shortest_path,
            Some(bedrock_addon::shared::TrueOnly)
        );
    }
}

#[test]
fn r04a_event_fix_boundary_is_per_file() {
    let skipping = |version: &str| {
        let file = loaded(controller(&ctrl(version, "{}"), &bp(1, 16, 0)));
        file.animation_controllers["controller.animation.x"].legacy_event_skipping
    };
    assert!(skipping("1.18.0"));
    assert!(!skipping("1.18.10"));
}

#[test]
fn r05_controller_versions_outside_the_range_are_rejected() {
    let cx = bp(1, 16, 0);
    let d = rejected(controller(&ctrl("1.7.0", "{}"), &cx));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in \"format_version\" field: 1.7.0",
    );
    let missing = r#"{"animation_controllers": {"controller.animation.x": {"states": {"a": {}}}}}"#;
    let d = rejected(controller(missing, &cx));
    expect(
        &d,
        "MissingFormatVersion",
        Severity::Error,
        "Error: ill-formatted/missing \"format_version\" field. Skipping.",
    );
    let d = rejected(controller(&ctrl("99.0.0", "{}"), &cx));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Error: animation controller file format version: 99.0.0 exceeds game version: 1.26.60.  Skipping.",
    );
    let number = ctrl("1.10.0", "{}").replace("\"1.10.0\"", "1.10");
    let d = rejected(controller(&number, &cx));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Error: ill-formatted/missing \"format_version\" field. Skipping.",
    );
    let array = ctrl("1.10.0", "{}").replace("\"1.10.0\"", "[1, 10, 0]");
    let d = rejected(controller(&array, &cx));
    expect_prefix(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "unknown child schema option type.  Allowed types: ",
    );
}

#[test]
fn r06_format_version_key_matches_ignoring_case() {
    let text = ctrl("1.10.0", "{}").replace("format_version", "Format_Version");
    let file = loaded(controller(&text, &bp(1, 16, 0)));
    assert_eq!(file.version().written, Some(FormatVersion::new(1, 10, 0)));
}

#[test]
fn r07_animation_versions() {
    let cx = bp(1, 16, 0);
    let body = r#"{"loop": true, "timeline": {"0.0": "/say hi"}}"#;
    let old = loaded(animation(&anim("1.8.0", body), &cx));
    let new = loaded(animation(&anim("1.10.0", body), &cx));
    assert_eq!(old.animations, new.animations);
    let d = rejected(animation(r#"{"animations": {"animation.x": {}}}"#, &cx));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child format_version not found",
    );
    let d = rejected(animation(&anim("1.5.0", "{}"), &cx));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.5.0 below min required version (1.8.0)",
    );
    let above = loaded(animation(&anim("99.0.0", "{}"), &cx));
    assert!(
        find(above.diagnostics(), "UnsupportedVersion", Severity::Info).is_some(),
        "{:?}",
        lines(above.diagnostics())
    );
}

/// The command version of the first `timeline` event of `animation.x`.
fn timeline_command_version(cx: &bedrock_addon::load::LoadContext) -> u8 {
    let file = loaded(animation(
        &anim("1.8.0", r#"{"timeline": {"0.0": "/say hi"}}"#),
        cx,
    ));
    match &file.animations["animation.x"].timeline.as_ref().unwrap()["0.0"] {
        OneOrMany::One(EventString::Command {
            command,
            command_version,
        }) => {
            assert_eq!(command, "say hi");
            *command_version
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn r08_timeline_command_version_follows_the_pack() {
    assert_eq!(timeline_command_version(&bp(1, 16, 0)), 12);
    assert_eq!(timeline_command_version(&bp(1, 21, 0)), 39);
}

#[test]
fn r09_timeline_forms() {
    let body = r#"{"timeline": {"0.0": ["/say hi", "@s wiki:grow", "v.x = 1;", "@a foo", "@sfoo", ""], "0.5": "@s "}}"#;
    let file = loaded(animation(&anim("1.8.0", body), &bp(1, 16, 0)));
    let timeline = file.animations["animation.x"].timeline.as_ref().unwrap();
    let OneOrMany::Many(events) = &timeline["0.0"] else {
        panic!("{timeline:?}")
    };
    assert!(matches!(&events[0], EventString::Command { command, .. } if command == "say hi"));
    assert!(matches!(&events[1], EventString::EntityEvent(e) if e == "wiki:grow"));
    assert!(matches!(&events[2], EventString::Molang(s) if s.as_str() == "v.x = 1;"));
    assert!(matches!(&events[3], EventString::Ignored(t) if t == "@a foo"));
    assert!(matches!(&events[4], EventString::Ignored(t) if t == "@sfoo"));
    assert!(matches!(&events[5], EventString::Ignored(t) if t.is_empty()));
    assert!(
        matches!(&timeline["0.5"], OneOrMany::One(EventString::EntityEvent(e)) if e.is_empty())
    );
    let unrecognized = file
        .diagnostics()
        .iter()
        .filter(|d| d.code.name() == "UnrecognizedEventString" && d.severity == Severity::Info)
        .count();
    assert_eq!(unrecognized, 2, "{:?}", lines(file.diagnostics()));
}

#[test]
fn r10_controller_events() {
    let state = r#"{"on_entry": ["/tag @s add a", "@s wiki:e"], "on_exit": ["v.y = 0;"]}"#;
    let cx = bp(1, 16, 0);
    let file = loaded(controller(&ctrl("1.10.0", state), &cx));
    let state = &file.animation_controllers["controller.animation.x"].states["default"];
    let entry = state.on_entry.as_ref().unwrap();
    assert!(
        matches!(&entry[0], EventString::Command { command, command_version: 12 } if command == "tag @s add a")
    );
    assert!(matches!(&entry[1], EventString::EntityEvent(e) if e == "wiki:e"));
    let exit = state.on_exit.as_ref().unwrap();
    assert!(
        matches!(&exit[0], EventString::Molang(s) if s.raw_version() == Some(molang_of(1, 10, 0)))
    );
}

#[test]
fn r11_names_lowercased() {
    let file = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.Wiki.Cmd": {"initial_state": "Default", "states": {"Default": {"animations": ["Walk"]}}}}}"#;
    let set = stacks(&[&[("animation_controllers/c.json", file)]], &[]);
    let registry = set.behavior.animation_controllers();
    let id = AnimationRef::parse("controller.animation.wiki.cmd");
    let controller = registry.get(&id).expect("registered lower-cased");
    let state = &controller.states["Default"];
    assert!(
        matches!(&state.animations.as_ref().unwrap()[0], StateAnimation::Name(n) if n == "Walk")
    );
    assert_eq!(
        state.animations.as_ref().unwrap()[0].name(),
        Some("walk".to_owned())
    );
    expect(
        controller.diagnostics(),
        "UnresolvedReference",
        Severity::Info,
        "Error: state Default not found - defaulting to using state 0",
    );
}

#[test]
fn r12_stack_replace_keeps_the_lower_definition_when_the_higher_fails() {
    let low = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"low": {}}}}}"#;
    let high = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"high": {}}}}}"#;
    let broken = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"high": {"foo": 1}}}}}"#;
    let id = AnimationRef::parse("controller.animation.x");
    let set = stacks(
        &[
            &[("animation_controllers/c.json", high)],
            &[("animation_controllers/c.json", low)],
        ],
        &[],
    );
    let winner = set.behavior.animation_controllers().get(&id).unwrap();
    assert!(winner.states.contains_key("high"));
    let set = stacks(
        &[
            &[("animation_controllers/c.json", broken)],
            &[("animation_controllers/c.json", low)],
        ],
        &[],
    );
    let registry = set.behavior.animation_controllers();
    assert!(registry.get(&id).unwrap().states.contains_key("low"));
    assert_eq!(registry.rejected().len(), 1);
}

#[test]
fn r13_unknown_member_rejects_the_file() {
    let d = rejected(controller(&ctrl("1.10.0", r#"{"foo": 1}"#), &bp(1, 16, 0)));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'foo' not valid here.",
    );
}

#[test]
fn r14_anim_member_case() {
    let text = r#"{"format_version": "1.8.0", "animations": {"Animation.Wiki.Walk": {
        "Loop": true, "Timeline": {"0.0": "/say hi"},
        "bones": {"head": {"rotation": {"0.0": {"lerp_mode": "CatmullRom", "post": [0, 0, 0]}}}},
        "particle_effects": {"0.0": {"effect": "Smoke", "locator": "Head"}}}}}"#;
    let set = stacks(&[&[("animations/a.json", text)]], &[]);
    let registry = set.behavior.animations();
    let walk = registry
        .get(&AnimationRef::parse("animation.wiki.walk"))
        .unwrap_or_else(|| panic!("{registry:?}"));
    assert_eq!(walk.loop_mode(), LoopMode::Loop);
    assert_eq!(walk.timeline.as_ref().unwrap().len(), 1);
    let channel = walk.bone("head").unwrap().rotation.as_ref().unwrap();
    let frames = channel.keyframes_by_time();
    assert!(matches!(
        frames[0].1,
        bedrock_addon::families::animation::Keyframe::Split(parts) if parts.lerp_mode == Some(LerpMode::CatmullRom)
    ));
    let OneOrMany::One(effect) = &walk.particle_effects.as_ref().unwrap()["0.0"] else {
        panic!()
    };
    assert_eq!(effect.to_effect_name(), "smoke");
    assert_eq!(effect.to_locator_name(), "head");
}

#[test]
fn r15_anim_bind_to_actor_true_is_rejected() {
    let cx = bp(1, 16, 0);
    let event = |bind: &str| {
        anim(
            "1.8.0",
            &format!(r#"{{"particle_effects": {{"0.0": {{"effect": "x"{bind}}}}}}}"#),
        )
    };
    let d = rejected(animation(&event(r#", "bind_to_actor": true"#), &cx));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'true' not allowed",
    );
    loaded(animation(&event(r#", "bind_to_actor": false"#), &cx));
    loaded(animation(&event(""), &cx));
}

#[test]
fn r16_timeline_bad_key_rejects_the_file() {
    let d = rejected(animation(
        &anim("1.8.0", r#"{"timeline": {".5": "/say hi"}}"#),
        &bp(1, 16, 0),
    ));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child '.5' not valid here.",
    );
}

#[test]
fn r17_controller_value_sets() {
    let cx = bp(1, 16, 0);
    let d = rejected(controller(
        &ctrl("1.10.0", r#"{"blend_via_shortest_path": false}"#),
        &cx,
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'false' not allowed",
    );
    loaded(controller(
        &ctrl("1.10.0", r#"{"blend_via_shortest_path": true}"#),
        &cx,
    ));
    let d = rejected(controller(
        &ctrl(
            "1.10.0",
            r#"{"particle_effects": [{"effect": "x", "bind_to_actor": true}]}"#,
        ),
        &cx,
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'true' not allowed",
    );
}

#[test]
fn r18_controller_counts_and_keys() {
    let cx = bp(1, 16, 0);
    let d = rejected(controller(
        &ctrl("1.10.0", r#"{"transitions": [{"a": "1", "b": "0"}]}"#),
        &cx,
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Node has too many children (2 > 1)",
    );
    let file = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"animations": [{"walk": "1", "idle": "0.5"}]}"#,
        ),
        &cx,
    ));
    let state = &file.animation_controllers["controller.animation.x"].states["default"];
    assert!(
        matches!(&state.animations.as_ref().unwrap()[0], StateAnimation::Weighted(w) if w.len() == 2)
    );
    let file = loaded(controller(
        &ctrl("1.10.0", r#"{"blend_transition": {"0.25": 0.5, "x": 1.0}}"#),
        &cx,
    ));
    let state = &file.animation_controllers["controller.animation.x"].states["default"];
    let frames = state.blend_transition.as_ref().unwrap().frames();
    assert!(
        frames.contains(&(0.25, 0.5)) && frames.contains(&(0.0, 1.0)),
        "{frames:?}"
    );
}

#[test]
fn r19_event_at_s_spacing_keeps_the_leading_space() {
    let file = loaded(animation(
        &anim("1.8.0", r#"{"timeline": {"0": "@s  wiki:grow"}}"#),
        &bp(1, 16, 0),
    ));
    let timeline = file.animations["animation.x"].timeline.as_ref().unwrap();
    assert!(
        matches!(&timeline["0"], OneOrMany::One(EventString::EntityEvent(e)) if e == " wiki:grow")
    );
}

#[test]
fn round_trip_of_both_kinds_is_value_equal() {
    let cx = bp(1, 16, 0);
    let text = anim(
        "1.8.0",
        r#"{"loop": "hold_on_last_frame", "blend_weight": 0.1, "bones": {"b": {"relative_to": {"rotation": "entity"}, "rotation": [{"y": 1}, 0, "q.life_time"], "scale": 2}},
           "particle_effects": {"0.0": [{"effect": "x", "pre_effect_script": "v.x = 1;"}]}, "sound_effects": {}, "animation_length": 0.3}"#,
    );
    let document = loaded(animation(&text, &cx));
    round_trip(&document, |t| animation(t, &cx));
    let plain = unchecked();
    let document = loaded(controller(
        &fixture(FAMILY, "up_1_8_two_curves.before.json"),
        &plain,
    ));
    round_trip(&document, |t| controller(t, &plain));
    let full = ctrl(
        "1.17.30",
        r#"{"variables": {"v": {"input": "q.life_time", "remap_curve": {"0": 1}}}, "animations": ["a", {"b": 0.5}],
            "transitions": [{"b": "q.is_sneaking"}], "blend_transition": 0.2, "blend_via_shortest_path": true,
            "sound_effects": [{"effect": "s"}], "on_entry": ["/say x"], "on_exit": []}"#,
    );
    let document = loaded(controller(&full, &cx));
    round_trip(&document, |t| controller(t, &cx));
}

#[test]
fn default_paths_follow_the_first_definition() {
    use bedrock_addon::document::Placed;
    let cx = bp(1, 16, 0);
    let file = loaded(animation(&anim("1.8.0", "{}"), &cx));
    assert_eq!(
        file.default_path(),
        PackPath::from_static("animations/x.json")
    );
    let file = loaded(controller(&ctrl("1.10.0", "{}"), &cx));
    assert_eq!(
        file.default_path(),
        PackPath::from_static("animation_controllers/x.json")
    );
}

#[test]
fn molang_version_of_the_pack_maps_through_the_engine_version() {
    // The mapping the rows above rely on: 1.10.0 and 1.20.50 are different Molang versions.
    assert_eq!(
        molang_of(1, 20, 50),
        RawVersion::from(MolangVersion::from(
            &MinEngineVersion::new(1, 20, 50).engine_version()
        ))
    );
}

// ---- ----

#[test]
fn invalid_json_skips_either_file_kind() {
    let cx = bp(1, 16, 0);
    expect_prefix(
        &rejected(animation("{\"format_version\" 1}", &cx)),
        "MalformedInput",
        Severity::Error,
        "parse error:\n",
    );
    expect_prefix(
        &rejected(controller("[1, ", &cx)),
        "MalformedInput",
        Severity::Error,
        "parse error:\n",
    );
}

#[test]
fn events_in_a_pack_without_min_engine_version_have_no_command_version() {
    let cx = super::common::context(bedrock_addon::pack::PackType::Behavior, None);
    let file = loaded(animation(
        &anim("1.8.0", r#"{"timeline": {"0": "/say hi"}}"#),
        &cx,
    ));
    assert!(
        find(
            file.diagnostics(),
            "MissingMinEngineVersion",
            Severity::Info
        )
        .is_some(),
        "{:?}",
        lines(file.diagnostics())
    );
    let file = loaded(controller(
        &ctrl("1.10.0", r#"{"on_entry": ["/say hi"]}"#),
        &cx,
    ));
    assert!(
        find(
            file.diagnostics(),
            "MissingMinEngineVersion",
            Severity::Info
        )
        .is_some()
    );
}

#[test]
fn required_controller_members_are_reported() {
    let cx = bp(1, 16, 0);
    let d = rejected(controller(
        r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {}}}"#,
        &cx,
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child states not found",
    );
    let d = rejected(controller(
        &ctrl("1.10.0", r#"{"variables": {"v": {}}}"#),
        &cx,
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child input not found",
    );
    let d = rejected(controller(
        &ctrl("1.10.0", r#"{"sound_effects": [{"locator": "a"}]}"#),
        &cx,
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child effect not found",
    );
}

#[test]
fn a_loop_string_matches_case_sensitively() {
    let d = rejected(animation(
        &anim("1.8.0", r#"{"loop": "Hold_On_Last_Frame"}"#),
        &bp(1, 16, 0),
    ));
    expect_prefix(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'Hold_On_Last_Frame' not allowed.",
    );
}

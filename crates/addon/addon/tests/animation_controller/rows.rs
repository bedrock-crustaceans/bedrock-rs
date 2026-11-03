//! The resource-pack rows: version gate `v00`–`v07`, upgrade `u01`–`u08`, parse and defaults
//! `p01`–`p21`, gates `g01`–`g05`, Molang `m00`–`m09`, stack `s01`–`s06`. Resource pack at
//! `min_engine_version` 1.16.0 unless the row says otherwise.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::animation_controller::{
    AnimationControllerFile, BlendTransition, EventString, StateAnimation,
};
use bedrock_addon::id::AnimationRef;
use bedrock_addon::molang::{Molang, RawVersion};
use bedrock_addon::version::MinEngineVersion;

use super::common::{
    clean, controller, expect, expect_prefix, find, fixture, lines, loaded, rejected, rp, stacks,
};

const FAMILY: &str = "client_animation_controller";
const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";

/// A file of `version` with controller `controller.animation.x`, state `default` = `state` and
/// a second state `idle`.
fn ctrl(version: &str, state: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "animation_controllers": {{"controller.animation.x": {{"initial_state": "default", "states": {{"default": {state}, "idle": {{}}}}}}}}}}"#
    )
}

fn state(file: &AnimationControllerFile) -> &bedrock_addon::families::animation_controller::State {
    &file.animation_controllers["controller.animation.x"].states["default"]
}

fn pack_molang(a: u16, b: u16, c: u16) -> RawVersion {
    RawVersion::from(MinEngineVersion::new(a, b, c).molang_version())
}

/// A resource pack at 1.16.0 without the Molang compile check (rows whose queries are
/// placeholders).
fn unchecked() -> bedrock_addon::load::LoadContext {
    #[cfg_attr(not(feature = "molang"), allow(unused_mut))]
    let mut cx = rp(1, 16, 0);
    #[cfg(feature = "molang")]
    {
        cx.molang = None;
    }
    cx
}

// ---- version gate ----

#[test]
fn v00_missing_version() {
    let d = rejected(controller(
        r#"{"animation_controllers": {}}"#,
        &rp(1, 16, 0),
    ));
    expect(
        &d,
        "MissingFormatVersion",
        Severity::Error,
        "Error: ill-formatted/missing \"format_version\" field. Skipping.",
    );
}

#[test]
fn v01_case_insensitive_key() {
    let text = ctrl("1.10.0", "{}").replace("format_version", "Format_Version");
    let document = loaded(controller(&text, &rp(1, 16, 0)));
    assert_eq!(
        document.version().written,
        Some(bedrock_addon::version::FormatVersion::new(1, 10, 0))
    );
    let infos: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| d.severity >= Severity::Warning || d.code.name() != "NonCanonicalMemberCase")
        .collect();
    assert!(
        infos.iter().all(|d| d.severity < Severity::Warning),
        "{:?}",
        lines(document.diagnostics())
    );
    assert!(
        find(
            document.diagnostics(),
            "NonCanonicalMemberCase",
            Severity::Info
        )
        .is_some()
    );
}

#[test]
fn v02_below_floor() {
    let d = rejected(controller(&ctrl("1.7.0", "{}"), &rp(1, 16, 0)));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in \"format_version\" field: 1.7.0",
    );
}

#[test]
fn v03_above_engine() {
    let d = rejected(controller(&ctrl("99.0.0", "{}"), &rp(1, 16, 0)));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Error: animation controller file format version: 99.0.0 exceeds game version: 1.26.60.  Skipping.",
    );
}

#[test]
fn v04_partial() {
    let document = loaded(controller(&ctrl("1.10", "{}"), &rp(1, 16, 0)));
    assert!(!document.version().upgraded);
}

#[test]
fn v05_array_version() {
    let text = ctrl("1.10.0", "{}").replace("\"1.10.0\"", "[1, 10, 0]");
    expect_prefix(
        &rejected(controller(&text, &rp(1, 16, 0))),
        "UnsupportedVersion",
        Severity::Error,
        TYPE_ERROR,
    );
}

#[test]
fn v06_prerelease_is_the_1_8_range() {
    let document = loaded(controller(&ctrl("1.10.0-beta.1", "{}"), &rp(1, 16, 0)));
    assert!(document.version().upgraded);
    assert!(
        find(
            document.diagnostics(),
            "PrereleaseFormatVersion",
            Severity::Info
        )
        .is_some()
    );
}

#[test]
fn v07_vanilla_unit_tests() {
    let cx = rp(1, 16, 0);
    expect(
        &rejected(controller(&ctrl("0.9.0", "{}"), &cx)),
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in \"format_version\" field: 0.9.0",
    );
    expect(
        &rejected(controller(r#"{"animation_controllers": {}}"#, &cx)),
        "MissingFormatVersion",
        Severity::Error,
        "Error: ill-formatted/missing \"format_version\" field. Skipping.",
    );
    expect_prefix(
        &rejected(controller(&ctrl("1.26.61", "{}"), &cx)),
        "UnsupportedVersion",
        Severity::Error,
        "Error: animation controller file format version: 1.26.61 exceeds game version: ",
    );
}

// ---- upgrade ----

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
fn u01_parameters_to_variables() {
    upgrade_pair("u01_parameters_to_variables");
}

#[test]
fn u02_empty_curve_list() {
    upgrade_pair("u02_empty_curve_list");
}

#[test]
fn u03_more_curves_than_parameters() {
    let text = ctrl(
        "1.8.0",
        r#"{"parameters": ["q.x"], "animations": [{"walk": [{"0": 0}, {"0": 1}]}]}"#,
    );
    let d = rejected(controller(&text, &unchecked()));
    let found = d
        .iter()
        .find(|f| f.code.name() == "InvalidType")
        .unwrap_or_else(|| panic!("{:?}", lines(&d)));
    assert!(
        found
            .location
            .json_path
            .to_string()
            .ends_with("variables/walk_1/input")
    );
    assert!(found.code.to_string().starts_with(TYPE_ERROR));
}

/// an input of `parameters` no blend curve of the state reads goes with the member
/// the step removes, and is noted where it is (the legacy judge: a state's `animations`
/// taken out lost both inputs with no word).
#[test]
fn u03b_a_parameter_no_curve_reads_is_noted_as_dropped() {
    const STATE: &str = "animation_controllers/controller.animation.x/states/default";

    let noted = |state: &str| -> Vec<String> {
        let file = loaded(controller(&ctrl("1.8.0", state), &unchecked()));
        file.diagnostics()
            .iter()
            .filter(|f| f.code.name() == "DroppedField")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect()
    };
    // Two inputs, one curve: the second input is read by nothing.
    assert_eq!(
        noted(r#"{"parameters": ["q.x", "q.y"], "animations": [{"walk": [{"0": 0}]}]}"#),
        [format!("Info {STATE}/parameters/1")]
    );
    // No animation at all: both go.
    assert_eq!(
        noted(r#"{"parameters": ["q.x", "q.y"]}"#),
        [
            format!("Info {STATE}/parameters/0"),
            format!("Info {STATE}/parameters/1")
        ]
    );
    // As many curves as inputs: nothing is lost.
    assert!(
        noted(r#"{"parameters": ["q.x", "q.y"], "animations": [{"walk": [{"0": 0}, {"0": 1}]}]}"#)
            .is_empty()
    );
}

/// Of two `animations` entries of one name the later's
/// curves take the variables `<name>_<i>` of the earlier's; a curve of the earlier entry that is
/// written over by another is noted where it is.
#[test]
fn u03c_a_curve_a_later_entry_of_its_name_writes_over_is_noted_as_dropped() {
    const STATE: &str = "animation_controllers/controller.animation.x/states/default";

    let noted = |state: &str| -> Vec<String> {
        let file = loaded(controller(&ctrl("1.8.0", state), &unchecked()));
        file.diagnostics()
            .iter()
            .filter(|f| f.code.name() == "DroppedField")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect()
    };
    // The second `walk` has one curve: it takes `walk_0`; `walk_1` stays the first's.
    assert_eq!(
        noted(
            r#"{"parameters": ["q.x", "q.y"], "animations": [{"walk": [{"0": 0}, {"0": 1}]}, {"walk": [{"0": 0.5}]}]}"#
        ),
        [format!("Info {STATE}/animations/0/walk/0")]
    );
    // The same curve again writes nothing over.
    assert!(
        noted(
            r#"{"parameters": ["q.x", "q.y"], "animations": [{"walk": [{"0": 0}, {"0": 1}]}, {"walk": [{"0": 0}]}]}"#
        )
        .is_empty()
    );
}

#[test]
fn u04_string_animation_rejected() {
    expect_prefix(
        &rejected(controller(
            &ctrl("1.8.0", r#"{"animations": ["walk"]}"#),
            &unchecked(),
        )),
        "InvalidType",
        Severity::Error,
        TYPE_ERROR,
    );
}

#[test]
fn u05_v1_10_member_in_1_8() {
    let d = rejected(controller(
        &ctrl("1.8.0", r#"{"sound_effects": [{"effect": "a"}]}"#),
        &unchecked(),
    ));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'sound_effects' not valid here.",
    );
}

#[test]
fn u07_two_member_animation_element() {
    let d = rejected(controller(
        &ctrl("1.8.0", r#"{"animations": [{"walk": [{}], "run": [{}]}]}"#),
        &unchecked(),
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Node has too many children (2 > 1)",
    );
}

#[test]
fn u08_bind_to_actor_true_1_8() {
    let d = rejected(controller(
        &ctrl(
            "1.8.0",
            r#"{"particle_effects": [{"effect": "a", "bind_to_actor": true}]}"#,
        ),
        &unchecked(),
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value 'true' not allowed",
    );
}

#[test]
fn u06_uppercase_parameters_key_survives_and_fails() {
    let text = ctrl(
        "1.8.0",
        r#"{"Parameters": ["q.x"], "animations": [{"walk": [{"0": 0}]}]}"#,
    );
    let d = rejected(controller(&text, &unchecked()));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'Parameters' not valid here.",
    );
}

// ---- parse and defaults ----

#[test]
fn p01_full_state() {
    let body = r#"{"variables": {"v": {"input": "query.modified_move_speed", "remap_curve": {"0": 0, "1": 1}}},
        "animations": ["walk", {"run": "query.modified_move_speed"}], "transitions": [{"idle": "query.is_sneaking"}],
        "blend_transition": 0.2, "blend_via_shortest_path": true, "particle_effects": [{"effect": "dust", "locator": "head"}],
        "sound_effects": [{"effect": "step"}], "on_entry": ["/say hi", "@s minecraft:ev", "v.x = 1;"], "on_exit": ["/say bye"]}"#;
    let document = loaded(controller(&ctrl("1.10.0", body), &rp(1, 16, 0)));
    clean(document.diagnostics());
    let s = state(&document);
    assert_eq!(s.variable_names(), ["variable.v"]);
    assert_eq!(s.animations.as_ref().unwrap().len(), 2);
    assert_eq!(s.transitions.as_ref().unwrap()[0].target, "idle");
    assert!(matches!(
        s.blend_transition,
        Some(BlendTransition::Seconds(_))
    ));
    let entry = s.on_entry.as_ref().unwrap();
    assert!(matches!(&entry[0], EventString::Command { .. }));
    assert!(matches!(&entry[1], EventString::EntityEvent(e) if e == "minecraft:ev"));
    assert!(matches!(&entry[2], EventString::Molang(_)));
    assert_eq!(s.on_exit.as_ref().unwrap().len(), 1);
}

#[test]
fn p02_blend_transition_curve() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"blend_transition": {"0.0": 1.0, "0.5": 0.0}}"#,
        ),
        &rp(1, 16, 0),
    ));
    assert_eq!(
        state(&document).blend_transition.as_ref().unwrap().frames(),
        [(0.0, 1.0), (0.5, 0.0)]
    );
}

#[test]
fn p03_blend_transition_zero() {
    let document = loaded(controller(
        &ctrl("1.10.0", r#"{"blend_transition": 0}"#),
        &rp(1, 16, 0),
    ));
    assert!(
        state(&document)
            .blend_transition
            .as_ref()
            .unwrap()
            .frames()
            .is_empty()
    );
}

fn no_initial(states: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "animation_controllers": {{"controller.animation.x": {{"states": {states}}}}}}}"#
    )
}

#[test]
fn p04_initial_state_absent_default() {
    let document = loaded(controller(
        &no_initial(r#"{"idle": {}, "default": {}}"#),
        &rp(1, 16, 0),
    ));
    assert_eq!(
        document.animation_controllers["controller.animation.x"].initial_state_name(),
        Some("default")
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        lines(document.diagnostics())
    );
}

#[test]
fn p05_initial_state_absent_no_default() {
    let document = loaded(controller(
        &no_initial(r#"{"idle": {}, "walk": {}}"#),
        &rp(1, 16, 0),
    ));
    assert_eq!(
        document.animation_controllers["controller.animation.x"].initial_state_name(),
        Some("idle")
    );
    assert!(
        find(
            document.diagnostics(),
            "ImplicitInitialState",
            Severity::Info
        )
        .is_some()
    );
}

#[test]
fn p06_initial_state_case() {
    let text = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"initial_state": "Walk", "states": {"Walk": {}}}}}"#;
    let document = loaded(controller(text, &rp(1, 16, 0)));
    expect(
        document.diagnostics(),
        "UnresolvedReference",
        Severity::Info,
        "Error: state Walk not found - defaulting to using state 0",
    );
}

#[test]
fn p07_transition_target_missing() {
    let document = loaded(controller(
        &ctrl("1.10.0", r#"{"transitions": [{"nowhere": "1"}]}"#),
        &rp(1, 16, 0),
    ));
    expect(
        document.diagnostics(),
        "UnresolvedReference",
        Severity::Info,
        "Error: state nowhere not found - defaulting to using state 0",
    );
}

#[test]
fn p08_transition_case_1_10() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"idle": "query.get_name == 'Bob'"}]}"#,
        ),
        &rp(1, 16, 0),
    ));
    let c = &document.animation_controllers["controller.animation.x"];
    let t = &state(&document).transitions.as_ref().unwrap()[0];
    assert_eq!(c.transition_expression(t), "query.get_name == 'bob'");
    assert!(
        find(
            document.diagnostics(),
            "CaseFoldedTransition",
            Severity::Info
        )
        .is_some()
    );
}

#[test]
fn p09_transition_case_1_17_30() {
    let document = loaded(controller(
        &ctrl(
            "1.17.30",
            r#"{"transitions": [{"idle": "query.get_name == 'Bob'"}]}"#,
        ),
        &rp(1, 16, 0),
    ));
    let c = &document.animation_controllers["controller.animation.x"];
    let t = &state(&document).transitions.as_ref().unwrap()[0];
    assert_eq!(c.transition_expression(t), "query.get_name == 'Bob'");
    assert!(
        find(
            document.diagnostics(),
            "CaseFoldedTransition",
            Severity::Info
        )
        .is_none()
    );
}

#[test]
fn p10_unknown_member() {
    expect(
        &rejected(controller(
            &ctrl("1.10.0", r#"{"comment": "x"}"#),
            &rp(1, 16, 0),
        )),
        "UnrecognizedField",
        Severity::Error,
        "child 'comment' not valid here.",
    );
}

#[test]
fn p11_names_lowercased() {
    let text = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.Pig.Move": {"initial_state": "walk", "states": {"Walk": {
        "variables": {"Speed": {"input": "query.modified_move_speed"}}, "animations": ["Walk_Anim"],
        "particle_effects": [{"effect": "Dust", "locator": "Head"}]}}}}}"#;
    let set = stacks(&[], &[&[("animation_controllers/pig.json", text)]]);
    let registry = set.resource.animation_controllers();
    let pig = registry
        .get(&AnimationRef::parse("controller.animation.pig.move"))
        .unwrap_or_else(|| panic!("{registry:?}"));
    let walk = pig.state("walk").unwrap();
    assert_eq!(pig.initial_state_name(), Some("Walk"));
    assert_eq!(walk.variable_names(), ["variable.speed"]);
    assert_eq!(
        walk.animations.as_ref().unwrap()[0].name().as_deref(),
        Some("walk_anim")
    );
    let effect = &walk.particle_effects.as_ref().unwrap()[0];
    assert_eq!(
        (
            effect.to_effect_name().as_str(),
            effect.to_locator_name().as_str()
        ),
        ("dust", "head")
    );
}

#[test]
fn p12_bad_prefix() {
    let text = r#"{"format_version": "1.10.0", "animation_controllers": {"animation.pig.move": {"states": {"a": {}}}}}"#;
    expect(
        &rejected(controller(text, &rp(1, 16, 0))),
        "UnrecognizedField",
        Severity::Error,
        "child 'animation.pig.move' not valid here.",
    );
}

#[test]
fn p13_particle_defaults() {
    let document = loaded(controller(
        &ctrl("1.10.0", r#"{"particle_effects": [{"effect": "a"}]}"#),
        &rp(1, 16, 0),
    ));
    let effect = &state(&document).particle_effects.as_ref().unwrap()[0];
    assert!(effect.binds_to_actor());
    assert!(effect.locator.is_none());
}

#[test]
fn p14_member_case() {
    let text = r#"{"format_version": "1.10.0", "animation_controllers": {"Controller.Animation.Pig": {"States": {"a": {"Animations": ["x"], "Transitions": [{"a": "1"}]}}}}}"#;
    let document = loaded(controller(text, &rp(1, 16, 0)));
    let infos = document
        .diagnostics()
        .iter()
        .filter(|d| d.code.name() == "NonCanonicalMemberCase" && d.severity == Severity::Info)
        .count();
    assert_eq!(infos, 4, "{:?}", lines(document.diagnostics()));
    let set = stacks(&[], &[&[("animation_controllers/pig.json", text)]]);
    assert!(
        set.resource
            .animation_controllers()
            .get(&AnimationRef::parse("controller.animation.pig"))
            .is_some()
    );
}

/// A member's name is matched in any case, so two
/// members that differ in case alone are one member, and the later one's value replaces the
/// earlier's. The earlier one is told where the file has it; one member alone, in whatever
/// case, has nothing replaced.
#[test]
fn p14b_a_member_another_spelling_of_its_name_replaces_is_told_where_it_is() {
    const STATE: &str = "animation_controllers/controller.animation.x/states/default";

    let replaced = |text: &str| -> Vec<String> {
        loaded(controller(text, &rp(1, 16, 0)))
            .diagnostics()
            .iter()
            .filter(|d| d.code.name() == "DroppedField")
            .map(|d| format!("{:?} {}", d.severity, d.location.json_path))
            .collect()
    };
    for version in ["1.10.0", "1.8.0"] {
        assert_eq!(
            replaced(&ctrl(
                version,
                r#"{"on_entry": ["/say first"], "On_Entry": ["/say second"], "Transitions": [{"idle": "1"}], "transitions": [{"default": "0"}]}"#
            )),
            [
                format!("Info {STATE}/on_entry"),
                format!("Info {STATE}/Transitions"),
            ],
            "the member that is replaced is not told: {version}"
        );
        assert!(
            replaced(&ctrl(version, r#"{"On_Entry": ["/say second"]}"#)).is_empty(),
            "{version}"
        );
    }
    // A controller's own members.
    assert_eq!(
        replaced(
            r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"initial_state": "default", "Initial_State": "other", "states": {"default": {"on_entry": ["/say gone"]}}, "States": {"default": {}, "other": {}}}}}"#
        ),
        [
            "Info animation_controllers/controller.animation.x/initial_state",
            "Info animation_controllers/controller.animation.x/states",
        ]
    );
}

/// Two spellings of one member with the SAME value
/// replace nothing that matters, so nothing is told, like every other "replaced" note; another
/// value keeps the warning.
#[test]
fn p14c_two_spellings_of_one_member_with_the_same_value_say_nothing() {
    let replaced = |text: &str| -> Vec<String> {
        loaded(controller(text, &rp(1, 16, 0)))
            .diagnostics()
            .iter()
            .filter(|d| d.code.name() == "DroppedField")
            .map(|d| format!("{:?} {}", d.severity, d.location.json_path))
            .collect()
    };
    let controller_with = |first: &str, second: &str| {
        format!(
            r#"{{"format_version": "1.10.0", "animation_controllers": {{"controller.animation.x": {{"initial_state": {first}, "Initial_State": {second}, "states": {{"default": {{}}, "other": {{}}}}}}}}}}"#
        )
    };
    assert!(
        replaced(&controller_with(r#""default""#, r#""default""#)).is_empty(),
        "a replaced member of the same value is told"
    );
    assert_eq!(
        replaced(&controller_with(r#""default""#, r#""other""#)),
        ["Info animation_controllers/controller.animation.x/initial_state"],
        "a replaced member of another value is not told"
    );
}

#[test]
fn p15_key_full_match() {
    expect(
        &rejected(controller(
            &no_initial(r#"{"my state": {}}"#),
            &rp(1, 16, 0),
        )),
        "UnrecognizedField",
        Severity::Error,
        "child 'my state' not valid here.",
    );
}

#[test]
fn p17_shortest_path_false() {
    expect(
        &rejected(controller(
            &ctrl("1.10.0", r#"{"blend_via_shortest_path": false}"#),
            &rp(1, 16, 0),
        )),
        "InvalidValue",
        Severity::Error,
        "Value 'false' not allowed",
    );
    loaded(controller(
        &ctrl("1.10.0", r#"{"blend_via_shortest_path": true}"#),
        &rp(1, 16, 0),
    ));
}

#[test]
fn p18_bind_to_actor_true() {
    expect(
        &rejected(controller(
            &ctrl(
                "1.10.0",
                r#"{"particle_effects": [{"effect": "a", "bind_to_actor": true}]}"#,
            ),
            &rp(1, 16, 0),
        )),
        "InvalidValue",
        Severity::Error,
        "Value 'true' not allowed",
    );
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"particle_effects": [{"effect": "a", "bind_to_actor": false}]}"#,
        ),
        &rp(1, 16, 0),
    ));
    assert!(!state(&document).particle_effects.as_ref().unwrap()[0].binds_to_actor());
}

#[test]
fn p19_transition_member_count() {
    let cx = rp(1, 16, 0);
    expect(
        &rejected(controller(
            &ctrl("1.10.0", r#"{"transitions": [{"a": "q.x", "b": "q.y"}]}"#),
            &cx,
        )),
        "InvalidValue",
        Severity::Error,
        "Node has too many children (2 > 1)",
    );
    expect(
        &rejected(controller(&ctrl("1.10.0", r#"{"transitions": [{}]}"#), &cx)),
        "InvalidValue",
        Severity::Error,
        "Node has too few children (0 < 1)",
    );
}

#[test]
fn p20_multi_member_animation_object() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"animations": [{"a": "1", "b": "q.life_time"}]}"#,
        ),
        &rp(1, 16, 0),
    ));
    assert!(
        matches!(&state(&document).animations.as_ref().unwrap()[0], StateAnimation::Weighted(w) if w.keys().collect::<Vec<_>>() == ["a", "b"])
    );
}

#[test]
fn p21_empty_arrays() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"animations": [], "transitions": [], "on_entry": []}"#,
        ),
        &rp(1, 16, 0),
    ));
    assert_eq!(state(&document).animations.as_deref(), Some(&[][..]));
}

#[test]
fn p16_remap_curve() {
    let cx = rp(1, 16, 0);
    let read = |curve: &str| {
        loaded(controller(
            &ctrl(
                "1.10.0",
                &format!(r#"{{"variables": {{"v": {{"input": "q.life_time"{curve}}}}}}}"#),
            ),
            &cx,
        ))
    };
    let document = read(r#", "remap_curve": {"0.0": 0.0, "1.0": 1.0}"#);
    assert_eq!(
        state(&document).variables.as_ref().unwrap()["v"].frames(),
        [(0.0, 0.0), (1.0, 1.0)]
    );
    let document = read(r#", "remap_curve": {"2": 1.0, "10": 0.0}"#);
    assert_eq!(
        state(&document).variables.as_ref().unwrap()["v"].frames(),
        [(10.0, 0.0), (2.0, 1.0)]
    );
    assert!(
        find(
            document.diagnostics(),
            "UnorderedRemapCurve",
            Severity::Info
        )
        .is_some()
    );
    let document = read("");
    assert!(
        state(&document).variables.as_ref().unwrap()["v"]
            .frames()
            .is_empty()
    );
}

// ---- effective version and gates ----

fn first_transition_version(file: &AnimationControllerFile) -> Option<RawVersion> {
    state(file).transitions.as_ref()?[0].condition.raw_version()
}

#[test]
fn g01_effective_fv() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"idle": "query.is_sneaking"}]}"#,
        ),
        &rp(1, 16, 0),
    ));
    assert_eq!(
        first_transition_version(&document),
        Some(pack_molang(1, 10, 0))
    );
    assert!(document.animation_controllers["controller.animation.x"].legacy_event_skipping);
}

#[test]
fn g02_effective_mev() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"idle": "query.is_sneaking"}]}"#,
        ),
        &rp(1, 21, 0),
    ));
    assert_eq!(
        first_transition_version(&document),
        Some(pack_molang(1, 21, 0))
    );
    assert!(document.animation_controllers["controller.animation.x"].legacy_event_skipping);
}

#[test]
fn g03_event_fix() {
    let document = loaded(controller(&ctrl("1.18.10", "{}"), &rp(1, 16, 0)));
    assert!(!document.animation_controllers["controller.animation.x"].legacy_event_skipping);
}

#[test]
fn g04_command_version_from_pack() {
    let version = |cx| match &state(&loaded(controller(
        &ctrl("1.10.0", r#"{"on_entry": ["/say hi"]}"#),
        &cx,
    )))
    .on_entry
    .as_ref()
    .unwrap()[0]
    {
        EventString::Command {
            command_version, ..
        } => *command_version,
        other => panic!("{other:?}"),
    };
    assert_eq!(version(rp(1, 16, 0)), 12);
    assert_eq!(version(rp(1, 21, 0)), 39);
}

#[test]
fn g05_slice_pack_molang() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"idle": "query.is_sneaking"}]}"#,
        ),
        &rp(1, 26, 10),
    ));
    let c = &document.animation_controllers["controller.animation.x"];
    assert_eq!(
        first_transition_version(&document),
        Some(pack_molang(1, 26, 10))
    );
    assert!(c.legacy_event_skipping && c.lowercase_transitions);
}

// ---- Molang ----

#[test]
fn m00_input_number_bool() {
    let document = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"variables": {"a": {"input": 2}}, "animations": [{"walk": true}]}"#,
        ),
        &rp(1, 21, 0),
    ));
    let s = state(&document);
    assert!(matches!(
        s.variables.as_ref().unwrap()["a"].input,
        Molang::Const(_)
    ));
    assert!(
        matches!(&s.animations.as_ref().unwrap()[0], StateAnimation::Weighted(w) if matches!(w["walk"], Molang::Bool(true)))
    );
    clean(document.diagnostics());
}

#[test]
fn m01_input_string() {
    clean(
        loaded(controller(
            &ctrl(
                "1.10.0",
                r#"{"variables": {"speed": {"input": "query.modified_move_speed"}}}"#,
            ),
            &rp(1, 21, 0),
        ))
        .diagnostics(),
    );
}

#[test]
fn m02_client_only_query() {
    clean(
        loaded(controller(
            &ctrl(
                "1.10.0",
                r#"{"animations": [{"walk": "query.distance_from_camera < 32"}]}"#,
            ),
            &rp(1, 21, 0),
        ))
        .diagnostics(),
    );
}

#[test]
fn m03_is_on_screen() {
    clean(
        loaded(controller(
            &ctrl(
                "1.10.0",
                r#"{"transitions": [{"idle": "!query.is_on_screen"}]}"#,
            ),
            &rp(1, 21, 0),
        ))
        .diagnostics(),
    );
}

#[cfg(feature = "molang")]
mod compiled {
    use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon::molang::DiagCode;

    use super::super::common::{controller, fixture, lines, loaded, rejected, rp};
    use super::{FAMILY, ctrl};

    fn invalid(d: &Diagnostics) -> Vec<(DiagCode, Severity)> {
        d.iter()
            .filter(|f| f.code.name() == "InvalidMolang")
            .map(|f| match f.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => (m.code(), f.severity),
                _ => (DiagCode::Syntax, f.severity),
            })
            .collect()
    }

    #[test]
    fn m04_input_rejected() {
        let d = rejected(controller(
            &ctrl(
                "1.10.0",
                r#"{"variables": {"a": {"input": "query.does_not_exist"}}}"#,
            ),
            &rp(1, 21, 0),
        ));
        assert_eq!(
            invalid(&d),
            [
                (DiagCode::UnknownQuery, Severity::Error),
                (DiagCode::Syntax, Severity::Error)
            ],
            "{:?}",
            lines(&d)
        );
    }

    #[test]
    fn m05_transition_rejected_is_kept() {
        let document = loaded(controller(
            &ctrl("1.10.0", r#"{"transitions": [{"idle": "v.x = 1"}]}"#),
            &rp(1, 21, 0),
        ));
        assert_eq!(
            invalid(document.diagnostics()),
            [(DiagCode::StatementForm, Severity::Error)]
        );
    }

    #[test]
    fn m06_on_entry_rejected_is_kept() {
        let document = loaded(controller(
            &ctrl(
                "1.10.0",
                r#"{"on_entry": ["v.x = (", "/say hi", "@s minecraft:ev"]}"#,
            ),
            &rp(1, 21, 0),
        ));
        let found = invalid(document.diagnostics());
        assert!(
            !found.is_empty() && found.iter().all(|(_, s)| *s == Severity::Error),
            "{:?}",
            lines(document.diagnostics())
        );
        // Only the Molang arm reached the compiler: every finding is at `on_entry/0`.
        assert!(
            document
                .diagnostics()
                .iter()
                .filter(|d| d.code.name() == "InvalidMolang")
                .all(|d| d.location.json_path.to_string().ends_with("on_entry/0")),
            "{:?}",
            lines(document.diagnostics())
        );
        let entry = super::state(&document).on_entry.as_ref().unwrap();
        let molang = entry
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    bedrock_addon::families::animation_controller::EventString::Molang(_)
                )
            })
            .count();
        assert_eq!(molang, 1);
    }

    #[test]
    fn m07_v1_8_upgraded_compiles_clean() {
        let document = loaded(controller(
            &fixture(FAMILY, "u01_parameters_to_variables.before.json"),
            &rp(1, 16, 0),
        ));
        assert!(
            invalid(document.diagnostics()).is_empty(),
            "{:?}",
            lines(document.diagnostics())
        );
    }

    #[test]
    fn m08_v1_8_transition_rejected() {
        let d = rejected(controller(
            &ctrl("1.8.0", r#"{"transitions": [{"idle": "query.("}]}"#),
            &rp(1, 21, 0),
        ));
        assert!(
            invalid(&d).iter().any(|(_, s)| *s == Severity::Error),
            "{:?}",
            lines(&d)
        );
    }

    #[test]
    fn m09_lowercase_view_compiles_clean() {
        let document = loaded(controller(
            &ctrl(
                "1.10.0",
                r#"{"transitions": [{"idle": "query.get_name == 'Dinnerbone'"}]}"#,
            ),
            &rp(1, 21, 0),
        ));
        assert!(invalid(document.diagnostics()).is_empty());
        let t = &super::state(&document).transitions.as_ref().unwrap()[0];
        assert_eq!(t.condition.as_str(), "query.get_name == 'Dinnerbone'");
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|d| d.code.name() == "CaseFoldedTransition" && d.severity == Severity::Info)
        );
    }

    /// A lower-cased transition compiles in lower case, but its findings quote the text as
    /// written.
    #[test]
    fn a_lowercase_transitions_findings_quote_the_written_text() {
        use bedrock_addon::diagnostic::DiagnosticCode;
        let written = "Query.Is_Baby +";
        let document = loaded(controller(
            &ctrl(
                "1.10.0",
                &format!(r#"{{"transitions": [{{"idle": "{written}"}}]}}"#),
            ),
            &rp(1, 21, 0),
        ));
        let quoted: Vec<&str> = document
            .diagnostics()
            .iter()
            .filter_map(|d| match &d.code {
                DiagnosticCode::InvalidMolang { expression } => Some(expression.as_str()),
                _ => None,
            })
            .collect();
        assert!(!quoted.is_empty(), "{:?}", lines(document.diagnostics()));
        assert!(quoted.iter().all(|e| *e == written), "{quoted:?}");
    }
}

// ---- stack ----

fn one(name: &str, state: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "animation_controllers": {{"{name}": {{"states": {{"{state}": {{}}}}}}}}}}"#
    )
}

#[test]
fn s01_override() {
    let set = stacks(
        &[],
        &[
            &[(
                "animation_controllers/x.json",
                &one("controller.animation.x", "top"),
            )],
            &[(
                "animation_controllers/x.json",
                &one("controller.animation.x", "low"),
            )],
        ],
    );
    let registry = set.resource.animation_controllers();
    assert!(
        registry
            .get(&AnimationRef::parse("controller.animation.x"))
            .unwrap()
            .states
            .contains_key("top")
    );
    assert!(registry.diagnostics().any(|d| d.code.name() == "AnimationControllerShadowed" && d.severity == Severity::Info));
}

#[test]
fn s02_case_override() {
    let set = stacks(
        &[],
        &[
            &[(
                "animation_controllers/x.json",
                &one("controller.animation.x", "top"),
            )],
            &[(
                "animation_controllers/x.json",
                &one("controller.animation.X", "low"),
            )],
        ],
    );
    let registry = set.resource.animation_controllers();
    assert_eq!(registry.len(), 1);
    assert!(
        registry
            .get(&AnimationRef::parse("controller.animation.x"))
            .unwrap()
            .states
            .contains_key("top")
    );
}

#[test]
fn s03_broken_override_keeps_lower() {
    let broken = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"top": {"x": 1}}}}}"#;
    let set = stacks(
        &[],
        &[
            &[("animation_controllers/x.json", broken)],
            &[(
                "animation_controllers/x.json",
                &one("controller.animation.x", "low"),
            )],
        ],
    );
    assert!(
        set.resource
            .animation_controllers()
            .get(&AnimationRef::parse("controller.animation.x"))
            .unwrap()
            .states
            .contains_key("low")
    );
}

#[test]
fn s04_sibling_loss() {
    let text = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.a": {"states": {"s": {}}}, "controller.animation.b": {"states": {"s": {"x": 1}}}}}"#;
    let set = stacks(&[], &[&[("animation_controllers/x.json", text)]]);
    assert!(set.resource.animation_controllers().is_empty());
}

#[test]
fn s05_subfolder_ignored() {
    let set = stacks(
        &[],
        &[&[(
            "animation_controllers/sub/x.json",
            &one("controller.animation.x", "s"),
        )]],
    );
    assert!(set.resource.animation_controllers().is_empty());
}

#[test]
fn s06_non_json_extension() {
    let text = one("controller.animation.x", "s");
    let set = stacks(
        &[],
        &[&[
            ("animation_controllers/x.JSON", &text),
            ("animation_controllers/y.json.txt", &text),
        ]],
    );
    assert!(set.resource.animation_controllers().is_empty());
}

// ---- link rows (V14, V15, When = link) ----

/// The two files that give the pack the material the particle effect is drawn with.
const MATERIAL_LIST: (&str, &str) = (
    "materials/common.json",
    r#"[{"path": "materials/particles.material"}]"#,
);
const SMOKE_MATERIAL: (&str, &str) = (
    "materials/particles.material",
    r#"{"materials": {"version": "1.0.0", "particles_alpha": {}}}"#,
);

/// A particle effect that loads.
const SMOKE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:smoke", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_once": {"active_time": 1}}}}"#;

/// V14 / V15: a state's particle or sound `effect` that is no key of the using
/// client entity's `particle_effects` / `sound_effects` is an `UnresolvedReference` info of the
/// link pass.
// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn v14_v15_effect_keys_resolve_against_the_using_client_entity() {
    let entity = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "minecraft:pig",
        "particle_effects": {"smoke": "demo:smoke"}, "sound_effects": {"step": "random.click"},
        "animations": {"a": "controller.animation.x"}}}}"#;
    let text = ctrl(
        "1.10.0",
        r#"{"particle_effects": [{"effect": "smoke"}, {"effect": "dust"}], "sound_effects": [{"effect": "Step"}, {"effect": "nowhere"}]}"#,
    );
    let set = stacks(
        &[],
        &[&[
            ("entity/x.entity.json", entity),
            ("animation_controllers/x.json", &text),
            // The entity is one of the game's and its particle is defined: the references
            // of the client entity itself are other rows'.
            ("particles/smoke.particle.json", SMOKE),
            // The particle's material is defined too: its reference is another row's.
            MATERIAL_LIST,
            SMOKE_MATERIAL,
        ]],
    );
    let mut found: Vec<_> = set
        .link()
        .iter()
        .filter(|d| d.code.name() == "UnresolvedReference")
        .map(|d| (d.severity, d.location.json_path.to_string()))
        .collect();
    found.sort();
    let at = "animation_controllers/controller.animation.x/states/default";
    assert_eq!(
        found,
        [
            (Severity::Info, format!("{at}/particle_effects/1/effect")),
            (Severity::Info, format!("{at}/sound_effects/1/effect")),
        ]
    );
}

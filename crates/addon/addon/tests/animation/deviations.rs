//! The listed deviations from vanilla, one test each (named `..._deviation`).

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::AnimationRef;
use bedrock_addon::pack::PackType;

use super::common::{animation, bp, context, controller, find, lines, loaded, rp, stacks};

fn ctrl(version: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "animation_controllers": {{"controller.animation.x": {{"states": {{"default": {{}}}}}}}}}}"#
    )
}

/// `legacy_event_skipping` is per file: vanilla takes the version of the first file per cache
/// bucket; the stored version is the file's own.
#[test]
fn legacy_event_skipping_is_per_file_deviation() {
    let skipping = |version: &str| {
        loaded(controller(&ctrl(version), &bp(1, 16, 0))).animation_controllers
            ["controller.animation.x"]
            .legacy_event_skipping
    };
    // Vanilla: a 1.17.30 file loaded after a 1.18.10 file in one process shares that bucket's
    // stored version and starts without skipping; here each file decides for itself.
    assert!(skipping("1.17.30"));
    assert!(!skipping("1.18.10"));
}

/// Vanilla makes no check: an animation file above the engine loads with a finding of ours.
#[test]
fn animation_above_the_engine_is_reported_deviation() {
    let text = r#"{"format_version": "1.26.61", "animations": {"animation.x": {}}}"#;
    let behavior = loaded(animation(text, &bp(1, 16, 0)));
    assert!(find(behavior.diagnostics(), "UnsupportedVersion", Severity::Info).is_some());
    let resource = loaded(animation(text, &rp(1, 21, 0)));
    assert!(
        find(
            resource.diagnostics(),
            "FormatVersionAboveEngine",
            Severity::Info
        )
        .is_some()
    );
}

/// Vanilla recurses except for app-shipped packs: sub-folders of `animations/` are read in every
/// pack.
#[test]
fn animations_below_sub_folders_are_read_in_every_pack_deviation() {
    let text = r#"{"format_version": "1.8.0", "animations": {"animation.x": {}}}"#;
    let set = stacks(&[], &[&[("animations/deep/er/a.json", text)]]);
    assert!(
        set.resource
            .animations()
            .get(&AnimationRef::parse("animation.x"))
            .is_some()
    );
}

/// Vanilla combines in `forEachIn` enumeration order: files of one pack are combined in sorted
/// path order, so the later path wins.
#[test]
fn files_of_one_pack_combine_in_sorted_order_deviation() {
    let named = |state: &str| {
        format!(
            r#"{{"format_version": "1.10.0", "animation_controllers": {{"controller.animation.x": {{"states": {{"{state}": {{}}}}}}}}}}"#
        )
    };
    let (a, b) = (named("from_a"), named("from_b"));
    let set = stacks(
        &[],
        &[&[
            ("animation_controllers/b.json", b.as_str()),
            ("animation_controllers/a.json", a.as_str()),
        ]],
    );
    let registry = set.resource.animation_controllers();
    let x = registry
        .get(&AnimationRef::parse("controller.animation.x"))
        .unwrap();
    assert!(x.states.contains_key("from_b"));
    assert!(
        registry
            .diagnostics()
            .any(|d| d.code.name() == "DuplicateIdentifier")
    );
}

/// Vanilla only asserts in debug builds: events in a pack without `min_engine_version` are a
/// warning.
#[test]
fn events_without_a_command_version_are_reported_deviation() {
    let cx = context(PackType::Resource, None);
    let file = loaded(animation(
        r#"{"format_version": "1.8.0", "animations": {"animation.x": {"timeline": {"0": "/say"}}}}"#,
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
}

/// `client_animation.toml` "our added rules", `vanilla = "no diagnostic"`: `DuplicateBone`,
/// `ShadowedDefinition`; the validation rules: `UnrecognizedEventString`.
#[test]
fn added_rules_report_what_vanilla_is_silent_on_deviation() {
    let file = loaded(animation(
        r#"{"format_version": "1.8.0", "animations": {"animation.x": {"bones": {"A": {"scale": 1}, "a": {"scale": 1}}, "timeline": {"0": "@a x"}}}}"#,
        &rp(1, 21, 0),
    ));
    assert!(find(file.diagnostics(), "DuplicateBone", Severity::Info).is_some());
    assert!(
        find(
            file.diagnostics(),
            "UnrecognizedEventString",
            Severity::Info
        )
        .is_some()
    );
    let text = r#"{"format_version": "1.8.0", "animations": {"animation.x": {}}}"#;
    let set = stacks(
        &[],
        &[
            &[("animations/a.json", text)],
            &[("animations/a.json", text)],
        ],
    );
    assert!(
        set.resource
            .animations()
            .diagnostics()
            .any(|d| d.code.name() == "ShadowedDefinition" && d.severity == Severity::Info)
    );
}

/// A Molang member that is a number
/// beyond `f32` is the largest finite float of its sign, with a warning at the number, where
/// vanilla's `asFloat` gives an infinity the writer has no spelling for.
#[test]
fn a_molang_number_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    use bedrock_addon::molang::Molang;
    let text = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"anim_time_update": 1e39, "blend_weight": -1e999, "bones": {"b": {"scale": [1, 1e39, 2]}}}}}"#;
    let document = loaded(animation(text, &rp(1, 21, 0)));
    let a = &document.animations["animation.x"];
    assert_eq!(a.anim_time_update, Some(Molang::Const(f32::MAX)));
    assert_eq!(a.blend_weight, Some(Molang::Const(f32::MIN)));
    let warnings: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "InvalidValue" && f.severity == Severity::Info)
        .map(|f| {
            let span = f.location.span.expect("the number's token");
            let (from, to) = (span.offset as usize, (span.offset + span.len) as usize);
            format!("{} {}", f.location.json_path, &text[from..to])
        })
        .collect();
    assert_eq!(
        warnings,
        [
            "animations/animation.x/anim_time_update 1e39",
            "animations/animation.x/blend_weight -1e999",
            "animations/animation.x/bones/b/scale/1 1e39",
        ],
        "{}",
        lines(document.diagnostics()).join("\n")
    );
    // What was read is written, and reads back equal without the finding.
    let written = document.to_json().unwrap();
    let again = loaded(animation(&written, &rp(1, 21, 0)));
    assert_eq!(again.value(), document.value(), "{written}");
    assert!(find(again.diagnostics(), "InvalidValue", Severity::Info).is_none());
    // The largest float itself is no deviation.
    let text = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"blend_weight": 3.4028235e38}}}"#;
    let document = loaded(animation(text, &rp(1, 21, 0)));
    assert!(find(document.diagnostics(), "InvalidValue", Severity::Info).is_none());
}

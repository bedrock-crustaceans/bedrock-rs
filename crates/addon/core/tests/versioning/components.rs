//! Components one at a time and the generic checks on the newest struct: `released` /
//! `requires` gates, deprecation, area, `supersedes`, synthesized entries bypassing them, and
//! `FieldMeta.since_engine`.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{FormatVersion, MinEngineVersion};
use bedrock_addon_core::versioning::{
    ComponentSection, FieldGate, FieldMeta, LoadOutcome, Loaded, check_fields_released,
    check_fields_since_engine, validate_components,
};

use crate::envelope::{codes, load_toy, load_with};
use crate::toy::{Fresh, Gated, Glow, OldGlow, Speed, Toy, Toy120, ToyComponent};

fn with_components(version: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {components}}}}}"#
    )
}

fn latest(outcome: &LoadOutcome<Toy>) -> &Toy120 {
    match &outcome.value {
        Some(Loaded::Stable(toy)) => toy,
        other => panic!("a stable value, not {other:?}: {:?}", outcome.diagnostics),
    }
}

fn names(map: &ComponentMap<ToyComponent>) -> Vec<String> {
    map.iter()
        .map(|entry| match entry {
            ComponentRef::Typed { name, .. }
            | ComponentRef::Dynamic { name, .. }
            | ComponentRef::Unknown { name, .. } => name.to_owned(),
            ComponentRef::Custom(custom) => custom.name.clone(),
            _ => String::new(),
        })
        .collect()
}

fn paths(outcome: &LoadOutcome<Toy>) -> Vec<String> {
    outcome
        .diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect()
}

#[test]
fn every_entry_kind_is_kept_in_file_order() {
    let outcome = load_toy(&with_components(
        "1.2.0",
        r#"{"demo:glow": {"a": 1}, "toy:speed": {"value": 2.5}, "toy:nope": true}"#,
    ));
    let toy = latest(&outcome);
    assert_eq!(
        names(&toy.components),
        ["demo:glow", "toy:speed", "toy:nope"]
    );
    assert_eq!(toy.components.get::<Speed>().map(|s| s.value), Some(2.5));
    assert!(matches!(
        toy.components.iter().next(),
        Some(ComponentRef::Custom(_))
    ));
    // The warning carries the game's own text for a member the schema does not list.
    assert_eq!(
        codes(&outcome),
        [(
            bedrock_addon_core::diagnostic::cereal::component_not_in_schema(),
            Severity::Warning
        )]
    );
    assert_eq!(paths(&outcome), ["minecraft:toy/components/toy:nope"]);
}

/// A failing component is one error at the component key plus facet's path, at a file offset,
/// and the document is refused: cereal's readers fail the value (level 3 in both masks)
/// and the load fails (the legacy block loader reports "Failed to parse block" the same way).
#[test]
fn a_failing_component_refuses_the_document() {
    let text = with_components(
        "1.2.0",
        r#"{"toy:speed": {"value": "fast"}, "toy:glow": {"level": 3}}"#,
    );
    let outcome = load_toy(&text);
    assert!(outcome.value.is_none(), "{:?}", outcome.diagnostics);
    let finding = outcome.diagnostics.iter().next().expect("a finding");
    // The member's finding alone: the document is refused, nothing is left to drop.
    assert_eq!(outcome.diagnostics.len(), 1, "{:?}", outcome.diagnostics);
    assert!(
        finding.code.id() == "core/invalid_type",
        "{:?}",
        finding.code
    );
    assert_eq!(finding.severity, Severity::Error);
    assert!(!finding.kept);
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:toy/components/toy:speed/value"
    );
    let span = finding.location.span.expect("a span");
    let at = text.find("\"value\": \"fast\"").expect("in the text");
    assert!(
        (at..at + 16).contains(&(span.offset as usize)),
        "the span {span:?} is on the member at {at}"
    );
}

#[test]
fn an_unknown_member_of_a_component_is_kept_and_reported_at_its_path() {
    let outcome = load_toy(&with_components(
        "1.2.0",
        r#"{"toy:speed": {"value": 1.0, "unit": "m/s"}}"#,
    ));
    let speed = latest(&outcome).components.get::<Speed>().expect("speed");
    let _ = speed;
    let at = bedrock_addon_core::diagnostic::JsonPath::of_keys([
        "minecraft:toy",
        "components",
        "toy:speed",
    ]);
    assert!(outcome.diagnostics.unknown().get(&at, "unit").is_some());
    assert_eq!(paths(&outcome), ["minecraft:toy/components/toy:speed/unit"]);
    assert_eq!(
        codes(&outcome),
        [(
            Code::from(&core_codes::UNRECOGNIZED_FIELD),
            Severity::Warning
        )]
    );
}

#[test]
fn a_released_component_needs_no_toggle_at_its_release() {
    let outcome = load_toy(&with_components("1.2.0", r#"{"toy:gated": {"on": true}}"#));
    assert!(latest(&outcome).components.contains::<Gated>());
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[test]
fn before_its_release_a_component_needs_its_requirement() {
    let text = with_components("1.1.0", r#"{"toy:gated": {"on": true}}"#);
    let outcome = load_toy(&text);
    assert!(outcome.value.is_none(), "the base components fail");
    let error = outcome.diagnostics.first_error().expect("an error");
    let code = &error.code;
    assert!(code.is(&core_codes::TURN_ON_TOGGLE_BEFORE), "{code:?}");
    // what, name, the releasing version, the toggle
    assert_eq!(
        &code.args()[1..],
        ["toy:gated", "1.2.0", "upcoming_creator_features"]
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:toy/components/toy:gated"
    );

    let mut cx = LoadContext::detached();
    cx.upcoming_creator_features = true;
    let outcome = load_with::<Toy>(&text, &cx);
    assert!(
        outcome.value.is_some(),
        "the toggle unlocks it: {:?}",
        outcome.diagnostics
    );
}

#[test]
fn an_unreleased_component_names_only_its_toggle() {
    let outcome = load_toy(&with_components(
        "1.2.0",
        r#"{"toy:preview": {"on": true}}"#,
    ));
    assert!(
        outcome
            .diagnostics
            .first_error()
            .is_some_and(|e| e.code.is(&core_codes::TURN_ON_TOGGLE))
    );
}

#[test]
fn the_legacy_creator_features_name_the_release_version() {
    let outcome = load_toy(&with_components("1.1.0", r#"{"toy:legacy": {"on": true}}"#));
    let error = outcome.diagnostics.first_error().expect("an error");
    assert!(
        matches!(
            &error.code,
            code
                if code.is(&core_codes::TOGGLE_USE_FORMAT_VERSION)
                    && code.args()[2] == FormatVersion::new(1, 2, 0).to_string()
        ),
        "{:?}",
        error.code
    );
}

#[test]
fn a_component_without_requirements_has_no_gate() {
    let outcome = load_toy(&with_components(
        "1.1.0",
        r#"{"toy:speed": {"value": 1.0}}"#,
    ));
    assert!(latest(&outcome).components.contains::<Speed>());
}

/// Components a step inserts are `synthesized` and bypass the gates; the same component
/// written by the author does not.
#[test]
fn a_synthesized_component_bypasses_the_gates() {
    let outcome = load_toy(&with_components("1.1.0", "{}"));
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    let outcome = load_toy(&with_components("1.2.0", r#"{"toy:marker": {"on": true}}"#));
    assert!(
        outcome
            .diagnostics
            .first_error()
            .is_some_and(|e| e.code.id() == "core/turn_on_toggle")
    );
}

/// At or above `deprecated_since` the member gets the deprecation text and is dropped.
#[test]
fn a_deprecated_component_is_dropped_with_the_generic_text() {
    let outcome = load_toy(&with_components(
        "1.1.0",
        r#"{"toy:speed": {"value": 1.0}, "toy:retired": {"on": true}}"#,
    ));
    assert_eq!(
        names(&latest(&outcome).components),
        ["toy:speed", "toy:marker"]
    );
    assert_eq!(
        codes(&outcome),
        [(
            core_codes::DEPRECATED_IN_FORMAT.with([
                "toy:retired".to_string(),
                FormatVersion::new(1, 1, 0).to_string()
            ]),
            Severity::Warning
        )]
    );
}

#[test]
fn a_vanilla_only_component_is_dropped_outside_base_game_packs() {
    let text = with_components("1.2.0", r#"{"toy:secret": {"on": true}}"#);
    let outcome = load_toy(&text);
    assert!(latest(&outcome).components.is_empty());
    assert_eq!(
        codes(&outcome),
        [(
            Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
            Severity::Warning
        )]
    );
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let outcome = load_with::<Toy>(&text, &cx);
    assert_eq!(names(&latest(&outcome).components), ["toy:secret"]);
}

#[test]
fn two_versions_of_a_component_keep_the_newer() {
    let outcome = load_toy(&with_components(
        "1.2.0",
        r#"{"toy:old_glow": {"level": 1}, "toy:speed": {"value": 1.0}, "toy:glow": {"level": 2}}"#,
    ));
    let toy = latest(&outcome);
    assert_eq!(names(&toy.components), ["toy:speed", "toy:glow"]);
    assert!(!toy.components.contains::<OldGlow>());
    assert_eq!(toy.components.get::<Glow>().map(|g| g.level), Some(2));
    assert_eq!(
        codes(&outcome),
        [(
            core_codes::DUPLICATE_COMPONENT_VERSION.with([
                "toy:old_glow",
                "toy:glow",
                "toy:glow",
                "toy:old_glow"
            ]),
            Severity::Info
        )]
    );
}

/// A failed gate in an optional section (a permutation) refuses the document as the main
/// section's does: a block permutation's toggle lines are level 3 and the block is refused.
#[test]
fn a_failed_gate_in_an_optional_section_is_an_error() {
    let mut map: ComponentMap<ToyComponent> = [Gated { on: true }.into()].into_iter().collect();
    let mut at = JsonPath::new();
    at.push("permutations");
    at.push_index(0);
    at.push("components");
    let mut base = JsonPath::new();
    base.push("minecraft:toy");
    let mut d = Diagnostics::default();
    let passed = validate_components::<Toy>(
        ComponentSection::new(at, true, &mut map),
        FormatVersion::new(1, 1, 0),
        &LoadContext::detached(),
        &base,
        &mut d,
    );
    assert!(!passed);
    assert!(d.has_errors());
    let finding = d.iter().next().expect("a finding");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:toy/permutations/0/components/toy:gated"
    );
}

#[test]
fn a_field_newer_than_the_packs_engine_is_reported_when_present() {
    let fields = [
        FieldMeta {
            name: "cooldown.type",
            since_engine: FormatVersion::new(1, 21, 130),
        },
        FieldMeta {
            name: "old",
            since_engine: FormatVersion::new(1, 20, 0),
        },
        FieldMeta {
            name: "absent",
            since_engine: FormatVersion::new(1, 26, 0),
        },
    ];
    let mut cx = LoadContext::detached();
    cx.min_engine_version = Some(MinEngineVersion::new(1, 21, 100));
    let mut at = JsonPath::new();
    at.push("components");
    at.push("minecraft:cooldown");
    let mut d = Diagnostics::default();
    check_fields_since_engine(&fields, |name| name != "absent", &at, &cx, &mut d);
    let found: Vec<_> = d
        .iter()
        .map(|f| (f.code.clone(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        found,
        [(
            core_codes::FIELD_SINCE_ENGINE.with(["1.21.130"]),
            "components/minecraft:cooldown/cooldown/type".to_owned()
        )]
    );
}

// ---- a released row without requirements, and the format-version field gate ----

/// For a family that does not ask for it, a row with `released` and an empty
/// `requires` is no gate at all.
#[test]
fn a_released_row_without_requirements_is_no_gate_by_default() {
    let outcome = load_toy(&with_components("1.1.0", r#"{"toy:fresh": {"on": true}}"#));
    assert!(latest(&outcome).components.contains::<Fresh>());
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[test]
fn a_member_below_its_format_version_is_refused_when_present() {
    let fields = [
        FieldGate {
            name: "underwater_music",
            released: FormatVersion::new(1, 21, 110),
        },
        FieldGate {
            name: "color.color_map",
            released: FormatVersion::new(1, 21, 70),
        },
        FieldGate {
            name: "absent",
            released: FormatVersion::new(1, 26, 0),
        },
        FieldGate {
            name: "old",
            released: FormatVersion::new(1, 21, 40),
        },
    ];
    let mut at = JsonPath::new();
    at.push("components");
    at.push("minecraft:biome_music");
    let run = |version: FormatVersion| {
        let mut d = Diagnostics::default();
        let passed = check_fields_released(
            &fields,
            |name| name != "absent",
            "biome_music",
            version,
            &at,
            &mut d,
        );
        let found: Vec<_> = d
            .iter()
            .map(|f| {
                assert_eq!(f.severity, Severity::Error);
                assert_eq!(f.code.id(), "core/use_format_version");
                (f.location.json_path.to_string(), f.code.to_string())
            })
            .collect();
        (passed, found)
    };
    // Below both: one error per present member, at the member, naming owner and member.
    let (passed, found) = run(FormatVersion::new(1, 21, 60));
    assert!(!passed);
    assert_eq!(
        found,
        [
            (
                "components/minecraft:biome_music/underwater_music".to_owned(),
                "To use field 'biome_music.underwater_music', use json format version 1.21.110 or higher".to_owned()
            ),
            (
                "components/minecraft:biome_music/color/color_map".to_owned(),
                "To use field 'biome_music.color.color_map', use json format version 1.21.70 or higher".to_owned()
            ),
        ]
    );
    // Each bound from both sides.
    let (passed, found) = run(FormatVersion::new(1, 21, 70));
    assert!(!passed);
    assert_eq!(found.len(), 1, "{found:?}");
    let (passed, found) = run(FormatVersion::new(1, 21, 109));
    assert!(!passed);
    assert_eq!(found.len(), 1, "{found:?}");
    let (passed, found) = run(FormatVersion::new(1, 21, 110));
    assert!(passed && found.is_empty(), "{found:?}");
}

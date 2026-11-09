//! The engine's own spawn inputs of 1.26.60 (copied under
//! `fixtures/spawn/spawn/engine/`) and the five drop-4 spawn rules. The library the inputs
//! come from is stripped; the candidate suites are `MobSpawnRulesServerTests`,
//! `ScriptingServerSpawnRulesTests`, …. A
//! 1.8.0 document is read by the standard (legacy) parser; a 1.21.60 one is the simplified
//! format, read with the Simplified Spawn Rules experiment on. A document that loads has no
//! warning or error (the placement test's four "must use a registered …" warnings excepted)
//! and round-trips; a refused one names its findings.
//!
//! Spawn-rule documents the game's own unit tests feed its loader; `case-2.fmt.txt` and
//! `case-4.fmt.txt` are format-string templates the tests fill before parsing.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::families::spawn_rules::{Condition, SpawnRules};
use bedrock_addon::json::Value;

use super::common::{
    context, experiment, fixture, load_group, loads, rejected, round_trip, standard,
};

/// An engine input by file name.
fn engine(name: &str) -> String {
    fixture("spawn", &format!("engine/{name}"))
}

/// A `.fmt.txt` template with its `{}` filled in order.
fn render(name: &str, values: &[&str]) -> String {
    let mut text = engine(name);
    for value in values {
        text = text.replacen("{}", value, 1);
    }
    text.replace("{{", "{").replace("}}", "}")
}

/// The findings at warning level or above: `(code name, severity, path, text)`.
fn found(d: &Diagnostics) -> Vec<(String, Severity, String, String)> {
    d.iter()
        .filter(|f| f.severity >= Severity::Warning)
        .map(|f| {
            let name = f.code.id().to_owned();
            (
                name,
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// The 1.21.60 simplified documents load with the experiment and round-trip; the standard
/// template (`{}`: identifier, population control, one condition) loads without it.
#[test]
fn the_engine_s_spawn_rules_load_and_round_trip() {
    for name in ["case-5.json", "case-6.json", "case-8.json", "case-9.json"] {
        let document = loads(&engine(name), &experiment());
        assert!(
            found(document.diagnostics()).is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        round_trip(document.value(), "spawn_rules/x.json", &experiment());
    }
    let template = render(
        "case-4.fmt.txt",
        &[
            "test:mob",
            "monster",
            r#""minecraft:weight": {"default": 1}"#,
        ],
    );
    let document = loads(&template, &context());
    assert!(
        found(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    round_trip(document.value(), "spawn_rules/x.json", &context());
}

/// case-7 (`NonExistantPlacements_Parse_ValidationLogged`, spawn validation rule 5): a
/// placement of four unregistered names loads with one warning per name.
#[test]
fn the_engine_s_unregistered_placement_loads_with_four_warnings() {
    let document = loads(&engine("case-7.json"), &experiment());
    let at = "minecraft:spawn_rules/placement";
    let warning = |code: &str, member: &str, text: &str| {
        (
            code.to_owned(),
            Severity::Warning,
            format!("{at}/{member}"),
            text.to_owned(),
        )
    };
    assert_eq!(
        found(document.diagnostics()),
        [
            warning(
                "spawn_rules/placement_type",
                "type",
                "Spawn Rules for 'type' in test:mob must use a registered placement type predicate"
            ),
            warning(
                "spawn_rules/placement_heightmap",
                "heightmap",
                "Spawn Rules for 'heightmap' in test:mob must use a registered heightmap type"
            ),
            warning(
                "spawn_rules/placement_obstruction",
                "spawn_obstruction_predicate",
                "Spawn Rules for 'spawn_obstruction_predicate' in test:mob must use a registered obstruction predicate"
            ),
            warning(
                "spawn_rules/placement_predicate",
                "spawn_predicate",
                "Spawn Rules for 'spawn_predicate' in test:mob must use a registered entity spawn predicate"
            ),
        ]
    );
}

/// A document without an identifier is refused, closed with the loader's "Failed to load spawn
/// rules": the standard parser's "Missing identifier in spawn rule file" for the
/// empty 1.8.0 payload, the simplified struct's required member for the empty 1.21.60
/// description.
#[test]
fn the_engine_s_spawn_rules_without_an_identifier_are_refused() {
    let at = "minecraft:spawn_rules/description/identifier";
    let closing = (
        "spawn_rules/failed_rules".to_owned(),
        Severity::Error,
        String::new(),
        "Failed to load spawn rules".to_owned(),
    );
    assert_eq!(
        found(&rejected(&engine("case-1.json"), &context())),
        [
            (
                "spawn/missing_identifier".to_owned(),
                Severity::Error,
                at.to_owned(),
                "Missing identifier in spawn rule file".to_owned()
            ),
            closing.clone(),
        ]
    );
    let refused = found(&rejected(&engine("case-10.json"), &experiment()));
    assert_eq!(refused.len(), 2, "{refused:#?}");
    assert_eq!(
        (refused[0].0.as_str(), refused[0].1, refused[0].2.as_str()),
        ("core/missing_required_field", Severity::Error, at)
    );
    assert_eq!(refused[1], closing);
}

/// The spawn groups: the template (`{}`: identifier, population control, one condition) loads
/// and round-trips; the empty payload is refused for its identifier, closed with "Failed to
/// load spawn groups".
#[test]
fn the_engine_s_spawn_groups() {
    let template = render(
        "case-2.fmt.txt",
        &[
            "test:group",
            "monster",
            r#""minecraft:weight": {"default": 1}"#,
        ],
    );
    let group = load_group(&template).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    assert!(
        found(group.diagnostics()).is_empty(),
        "{:#?}",
        group.diagnostics()
    );
    round_trip::<SpawnGroup>(group.value(), "definitions/spawn_groups/x.json", &context());
    let refused = load_group(&engine("case-3.json")).expect_err("refused");
    assert_eq!(
        found(&refused.diagnostics),
        [
            (
                "spawn/missing_identifier".to_owned(),
                Severity::Error,
                "minecraft:spawn_groups/description/identifier".to_owned(),
                "Missing identifier in spawn rule file".to_owned()
            ),
            (
                "spawn_groups/failed_groups".to_owned(),
                Severity::Error,
                String::new(),
                "Failed to load spawn groups".to_owned()
            ),
        ]
    );
}

/// The five drop-4 spawn rules of 1.26.60 (`experimental_y_2026_drop_4/spawn_rules/`,
/// format 1.8.0): each loads with the standard parser without a finding, writes back the JSON
/// value it read and round-trips. `frostbite` has three conditions of weights 50, 45 and 25.
#[test]
fn the_drop_4_spawn_rules_load_and_round_trip() {
    for name in ["frostbite", "skeleton", "spider", "stray", "zombie"] {
        let text = fixture("spawn", &format!("drop4/{name}.json"));
        let document = loads(&text, &context());
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        let written = round_trip::<SpawnRules>(document.value(), "spawn_rules/x.json", &context());
        let value = |t: &str| facet_json::from_str::<Value>(t).unwrap();
        assert_eq!(value(&written), value(&text), "{name}");
    }
    let frostbite = loads(&fixture("spawn", "drop4/frostbite.json"), &context());
    let weights: Vec<_> = standard(&frostbite)
        .conditions
        .as_ref()
        .expect("conditions")
        .iter()
        .map(Condition::spawn_weight)
        .collect();
    assert_eq!(weights, [50, 45, 25]);
}

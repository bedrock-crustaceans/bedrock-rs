//! The AI goals (`entity::goal`): each goal row's parse, strict and lenient, the member checks,
//! deprecated members, the lenient shape conversions, Molang, and the vanilla corpus.

mod corpus;
mod latest;
mod memory;
mod rows;
mod rules;
mod steps;
mod types;
mod versions;

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::json::{JsonSlice, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::EntityComponent;
use facet::Facet;

/// `(key, first typed format version, round trip)` of one goal.
pub type Goal = (&'static str, (u16, u16, u16), RoundTrip);

/// Parses a goal's JSON, writes it, parses the written text strictly and requires the same
/// value; returns the findings of the first parse, or `None` when the first parse dropped it.
pub type RoundTrip = fn(&str, &str, ConvertMode) -> Option<Diagnostics>;

/// The goal row `key` parses `json` in `mode`: the component (if kept) and the findings.
pub fn parse(key: &str, json: &str, mode: ConvertMode) -> (Option<EntityComponent>, Diagnostics) {
    let row = EntityComponent::TABLE
        .get(key)
        .unwrap_or_else(|| panic!("{key} has a row"));
    let mut d = Diagnostics::default();
    let value = (row.parse)(JsonSlice::new(json), mode, &LoadContext::detached(), &mut d);
    (value, d)
}

/// [`parse`], strict.
pub fn strict(key: &str, json: &str) -> (Option<EntityComponent>, Diagnostics) {
    parse(key, json, ConvertMode::Strict)
}

/// [`parse`], lenient.
pub fn lenient(key: &str, json: &str) -> (Option<EntityComponent>, Diagnostics) {
    parse(key, json, ConvertMode::Lenient)
}

/// The goal of type `T` the row `key` reads from `json` in `mode`, and the findings.
pub fn goal<T: Component<Set = EntityComponent>>(
    key: &str,
    json: &str,
    mode: ConvertMode,
) -> (Option<T>, Diagnostics) {
    let (value, d) = parse(key, json, mode);
    let value =
        value.map(|v| T::try_from_set(v).unwrap_or_else(|_| panic!("{key} is a {}", T::NAME)));
    (value, d)
}

/// The goal of type `T` that an entity document of format version `version` reads from `json`
/// under `key`, and the findings at or below `minecraft:entity/components/<key>`, with paths
/// relative to the goal (the versioned goal parse is the document's own). An error
/// rejects the document, so the goal is then `None`.
pub fn at_version<T: Component<Set = EntityComponent>>(
    key: &str,
    json: &str,
    version: (u16, u16, u16),
) -> (Option<T>, Diagnostics) {
    let (major, minor, patch) = version;
    let text = format!(
        r#"{{"format_version": "{major}.{minor}.{patch}", "minecraft:entity": {{"description": {{"identifier": "demo:goal"}}, "components": {{"{key}": {json}}}}}}}"#
    );
    let outcome = bedrock_addon_core::versioning::load::<bedrock_addon_family_entity::Entity>(
        JsonSlice::new(&text),
        &LoadContext::detached(),
    );
    let prefix = format!("minecraft:entity/components/{key}");
    let mut d = Diagnostics::default();
    for mut finding in outcome.diagnostics.iter().cloned() {
        let json_path = finding.location.json_path.to_string();
        let Some(rest) = json_path.strip_prefix(&prefix) else {
            continue;
        };
        let Some(rest) = rest.strip_prefix('/').or(rest.is_empty().then_some("")) else {
            continue;
        };
        let mut relative = bedrock_addon_core::diagnostic::JsonPath::new();
        for segment in rest.split('/').filter(|s| !s.is_empty()) {
            match segment.parse::<usize>() {
                Ok(index) => relative.push_index(index),
                Err(_) => relative.push(segment),
            }
        }
        finding.location.json_path = relative;
        d.push(finding);
    }
    // And the goal's unknown members, below it.
    let at =
        bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity", "components", key]);
    for (path, member, value) in outcome.diagnostics.unknown().iter() {
        if let Some(below) = path.strip_prefix(&at) {
            d.unknown_mut().put(&below, member, value.clone());
        }
    }
    let value = match outcome.value {
        Some(bedrock_addon_core::versioning::Loaded::Stable(mut entity)) => {
            entity.components.remove::<T>()
        }
        _ => None,
    };
    (value, d)
}

/// The round trip of goal type `T` (see [`RoundTrip`]): the written goal is a document at the
/// newest version, so it is read back strictly, without errors, and writes the same text; a
/// goal read strictly reads back equal.
pub fn round_trip<T>(key: &str, json: &str, mode: ConvertMode) -> Option<Diagnostics>
where
    T: Component<Set = EntityComponent> + for<'a> Facet<'a> + PartialEq + std::fmt::Debug,
{
    let (value, d) = goal::<T>(key, json, mode);
    let value = value?;
    let written = to_json(&value).unwrap_or_else(|e| panic!("{key} writes: {e}"));
    let (again, d2) = goal::<T>(key, &written, ConvertMode::Strict);
    assert!(
        errors(&d2).is_empty(),
        "{key}: the written goal reads back without errors: {d2:?}\n{written}"
    );
    let again = again.unwrap_or_else(|| panic!("{key}: the written goal reads back\n{written}"));
    let rewritten = to_json(&again).unwrap_or_else(|e| panic!("{key} writes: {e}"));
    assert_eq!(
        rewritten, written,
        "{key}: the written goal writes the same text"
    );
    if matches!(mode, ConvertMode::Strict) {
        assert_eq!(
            again, value,
            "{key}: the written goal reads back equal\n{written}"
        );
    }
    Some(d)
}

/// The members of fixture `name` (under `tests/fixtures/entity/entity/goal/`), each as its key
/// and its value's JSON text, in file order.
pub fn fixture(name: &str) -> Vec<(String, String)> {
    let path = format!(
        "{}/../../addon/tests/fixtures/entity/entity/goal/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let json: bedrock_addon_core::json::Value =
        facet_json::from_str_jsonc(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    json.as_object()
        .expect("a fixture is an object")
        .iter()
        .map(|(key, value)| {
            let text = facet_json::to_string(value).expect("a fixture value writes");
            (key.as_str().to_owned(), text)
        })
        .collect()
}

/// The JSON text of member `key` of fixture `name`.
pub fn fixture_member(name: &str, key: &str) -> String {
    fixture(name)
        .into_iter()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("{name} has {key}"))
        .1
}

/// The findings at error level that refuse the document (a kept error is not one).
pub fn errors(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|d| d.rejects()).collect()
}

/// The finding with `code` (by variant) at `path`, if any.
pub fn finding<'d>(d: &'d Diagnostics, code: &str, path: &str) -> Option<&'d Diagnostic> {
    d.iter()
        .find(|d| d.code.id() == code && d.location.json_path.to_string() == path)
}

/// Asserts there is a finding with `code` at `path` and `severity`.
#[track_caller]
pub fn assert_finding(d: &Diagnostics, code: &str, path: &str, severity: Severity) {
    let found = d.iter().any(|f| {
        f.code.id() == code && f.location.json_path.to_string() == path && f.severity == severity
    });
    assert!(found, "{code} ({severity:?}) at {path} in {d:?}");
}

/// `code` names a diagnostic code variant.
#[allow(dead_code)]
fn is_code(code: &Code, name: &str) -> bool {
    code.id() == name
}

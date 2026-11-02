//! The file kinds on their own: `Content::parse`, the round trip (the JSON as written, read
//! back equal and rewritten identically), the fixed paths, and the determinism fragment.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::{Content, Document, Placed, Rejected};
use bedrock_addon::families::json_ui::{DefKey, GlobalVariables, UiDefsList, UiFile};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::*;

fn parse<T: Content>(path: &str, text: &str) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::new(path).unwrap(),
        text.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Reads `text`, writes it, and requires the written text to read back equal and to rewrite
/// identically; returns the document and the written text.
fn round_trip<T: Content + PartialEq + std::fmt::Debug>(
    path: &str,
    text: &str,
) -> (Document<T>, String) {
    let document = parse::<T>(path, text).unwrap_or_else(|e| panic!("{path}: {e}"));
    let written = document.to_json().expect("it writes");
    let again = parse::<T>(path, &written).unwrap_or_else(|e| panic!("{path} rewritten: {e}"));
    assert_eq!(again.value(), document.value(), "{path}: read back");
    assert_eq!(again.to_json().unwrap(), written, "{path}: rewritten");
    assert!(again.diagnostics().is_empty() || !document.diagnostics().is_empty());
    (document, written)
}

fn vanilla(name: &str) -> String {
    std::fs::read_to_string(fixtures().join("vanilla").join(name)).unwrap()
}

#[test]
fn a_vanilla_ui_file_round_trips() {
    for name in ["hud_crosshair_overlay.json", "achievement_screen.json"] {
        let text = vanilla(name);
        let (file, written) = round_trip::<UiFile>(&format!("ui/{name}"), &text);
        assert!(file.diagnostics().is_empty());
        // every member and value as written: the same JSON
        assert_eq!(json(&written), json(&text), "{name}");
    }
    let (file, written) = round_trip::<UiFile>(
        "ui/hud_crosshair_overlay.json",
        &vanilla("hud_crosshair_overlay.json"),
    );
    assert_eq!(file.namespace.as_deref(), Some("hud_crosshair"));
    assert_eq!(file.namespace_or_root(), "hud_crosshair");
    let keys: Vec<String> = file.defs.keys().map(ToString::to_string).collect();
    assert_eq!(keys, ["cursor_renderer", "hud_crosshair_screen"]);
    // number tags and member order are kept: `16`, not `16.0`, `type` first
    assert!(
        written.contains("\"size\": [\n      16,\n      16\n    ]"),
        "{written}"
    );
    assert!(written.find("\"type\"").unwrap() < written.find("\"renderer\"").unwrap());
    assert!(written.starts_with("{\n  \"namespace\": \"hud_crosshair\""));
}

#[test]
fn a_ui_file_keeps_nulls_keys_and_writes_nothing_it_was_not_given() {
    let text = r#"{ "b@x.$y": { "layer": 1.50, "size": [ 1e2, -0 ] }, "a": null, "namespace": "n", "c@": 5 }"#;
    let (file, written) = round_trip::<UiFile>("ui/a.json", text);
    assert_eq!(json(&written), json(text));
    assert_eq!(file.defs.len(), 3);
    assert!(file.defs[&DefKey::parse("a")].is_null());
    // a null member defines nothing; definitions come in key order
    let names: Vec<&str> = file
        .definitions()
        .map(|(key, _)| key.name.as_str())
        .collect();
    assert_eq!(names, ["b", "c"]);
    // no namespace is written for a file without one
    let (bare, written) = round_trip::<UiFile>("ui/a.json", r#"{ "k": {} }"#);
    assert_eq!(bare.namespace, None);
    assert_eq!(bare.namespace_or_root(), "_root");
    assert!(!written.contains("namespace"));
    assert_eq!(round_trip::<UiFile>("ui/a.json", "{}").1, "{}\n");
}

#[test]
fn a_ui_file_built_by_hand_writes() {
    let mut file = UiFile {
        namespace: Some("mine".to_owned()),
        ..UiFile::default()
    };
    file.defs.insert(
        DefKey::parse("panel@common.base"),
        json(r#"{ "layer": 2 }"#),
    );
    assert_eq!(
        json(&file.to_json().unwrap()),
        json(r#"{ "namespace": "mine", "panel@common.base": { "layer": 2 } }"#)
    );
    let mut out = Vec::new();
    file.write_json(&mut out, bedrock_addon::json::WriteStyle::default())
        .unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), file.to_json().unwrap());
}

#[test]
fn a_file_that_is_no_json_object_is_rejected() {
    for text in ["", "[]", "5", r#"{ "k": "#, "null"] {
        let rejected = parse::<UiFile>("ui/a.json", text).unwrap_err();
        let finding = rejected.diagnostics.iter().next().unwrap();
        assert!(
            matches!(finding.code, DiagnosticCode::MalformedInput { .. }),
            "{text}"
        );
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(
            finding.location.path.as_ref().unwrap().as_str(),
            "ui/a.json"
        );
        assert!(parse::<GlobalVariables>("ui/_global_variables.json", text).is_err());
        assert!(parse::<UiDefsList>("ui/_ui_defs.json", text).is_err());
    }
    // a BOM and comments are read through
    let file = parse::<UiFile>("ui/a.json", "\u{feff}// c\n{ \"k\": {} /* d */ }").unwrap();
    assert_eq!(file.defs.len(), 1);
    // of a repeated member the last is kept
    let file = parse::<UiFile>(
        "ui/a.json",
        r#"{ "k": { "layer": 1 }, "k": { "layer": 2 } }"#,
    )
    .unwrap();
    assert_eq!(file.defs[&DefKey::parse("k")], json(r#"{ "layer": 2 }"#));
}

#[test]
fn a_namespace_that_is_no_string_is_kept_as_a_member() {
    let file = parse::<UiFile>("ui/a.json", r#"{ "namespace": 5, "k": {} }"#).unwrap();
    assert_eq!(file.namespace, None);
    assert_eq!(file.defs[&DefKey::parse("namespace")], Value::from(5_i64));
    let finding = file.diagnostics().iter().next().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::InvalidType { text: None }
    ));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(
        json(&file.to_json().unwrap()),
        json(r#"{ "namespace": 5, "k": {} }"#)
    );
    // a null namespace is none
    let file = parse::<UiFile>("ui/a.json", r#"{ "namespace": null }"#).unwrap();
    assert!(file.diagnostics().is_empty());
    assert_eq!(file.namespace_or_root(), "_root");
}

#[test]
fn a_defs_list_round_trips_and_has_its_fixed_path() {
    let text = r#"{ "ui_defs": [ "ui/b.json", "ui/a.json", 5 ], "note": "kept" }"#;
    let (list, written) = round_trip::<UiDefsList>("ui/_ui_defs.json", text);
    // an element that is not a string is read as the empty path
    assert_eq!(list.ui_defs, ["ui/b.json", "ui/a.json", ""]);
    assert!(list.extra.get("note").is_some());
    assert_eq!(
        json(&written),
        json(r#"{ "ui_defs": [ "ui/b.json", "ui/a.json", 5 ], "note": "kept" }"#)
    );
    assert_eq!(list.default_path().as_str(), "ui/_ui_defs.json");
    // a list that is no array lists nothing, with a warning, and is written back
    let odd = parse::<UiDefsList>("ui/_ui_defs.json", r#"{ "ui_defs": "ui/a.json" }"#).unwrap();
    assert!(odd.ui_defs.is_empty());
    assert_eq!(odd.diagnostics().len(), 1);
    assert!(odd.extra.get("ui_defs").is_some());
    let built = UiDefsList {
        ui_defs: vec!["ui/mine.json".to_owned()],
        ..UiDefsList::default()
    };
    assert_eq!(
        json(&built.to_json().unwrap()),
        json(r#"{ "ui_defs": ["ui/mine.json"] }"#)
    );
}

#[test]
fn global_variables_round_trip_and_have_their_fixed_path() {
    let text = r#"{ "$b": [1, 0, 0], "$a": { "x": 1 }, "$c": "text" }"#;
    let (globals, written) = round_trip::<GlobalVariables>("ui/_global_variables.json", text);
    let names: Vec<&str> = globals.variables.keys().map(String::as_str).collect();
    assert_eq!(names, ["$b", "$a", "$c"]);
    assert_eq!(json(&written), json(text));
    assert_eq!(globals.default_path().as_str(), "ui/_global_variables.json");
    assert_eq!(globals.color("$b"), Some([1.0, 0.0, 0.0, 1.0]));
    assert_eq!(globals.color("$c"), None);
    assert_eq!(globals.color("$missing"), None);
}

#[test]
fn colours_are_three_or_four_numbers_or_a_name() {
    let colour = |text: &str| {
        let globals = parse::<GlobalVariables>(
            "ui/_global_variables.json",
            &format!(r#"{{ "$c": {text} }}"#),
        )
        .unwrap();
        globals.color("$c")
    };
    assert_eq!(colour("[0.5, 0.25, 1]"), Some([0.5, 0.25, 1.0, 1.0]));
    assert_eq!(colour("[0, 0, 0, 0.5]"), Some([0.0, 0.0, 0.0, 0.5]));
    assert_eq!(colour("[1, 1]"), None);
    assert_eq!(colour("[1, 1, 1, 1, 1]"), None);
    assert_eq!(colour(r#"[1, "x", 1]"#), None);
    for name in [
        "white", "black", "grey", "gray", "red", "green", "blue", "yellow", "orange", "purple",
        "cyan", "nil",
    ] {
        assert!(colour(&format!("\"{name}\"")).is_some(), "{name}");
    }
    assert_eq!(colour("\"nil\""), Some([0.0, 0.0, 0.0, 0.0]));
    assert_eq!(colour("\"mauve\""), None);
}

#[test]
fn a_defs_list_writes_back_what_its_file_wrote() {
    let list = |text: &str| {
        let (list, written) = round_trip::<UiDefsList>("ui/_ui_defs.json", text);
        assert_eq!(written, json_text(text), "{text}");
        list.into_value()
    };
    // no member is invented for a file without one, and an empty one stays
    assert!(list("{}").ui_defs.is_empty());
    assert!(list(r#"{ "ui_defs": [] }"#).ui_defs.is_empty());
    assert!(list(r#"{ "other": 1 }"#).extra.get("ui_defs").is_none());
    // a value of the wrong type, and an element that is no string, stay as written
    assert!(list(r#"{ "ui_defs": "ui/a.json" }"#).ui_defs.is_empty());
    assert!(list(r#"{ "ui_defs": null }"#).ui_defs.is_empty());
    assert_eq!(
        list(r#"{ "ui_defs": ["ui/a.json", 5, null] }"#).ui_defs,
        ["ui/a.json", "", ""]
    );
    // a list of strings needs no copy beside it
    assert!(list(r#"{ "ui_defs": ["ui/a.json"] }"#).extra.is_empty());
    // a list changed by hand is written from the typed paths
    let mut changed = list(r#"{ "ui_defs": ["ui/a.json", 5] }"#);
    changed.ui_defs.push("ui/b.json".to_owned());
    assert_eq!(
        changed.to_json().unwrap(),
        json_text(r#"{ "ui_defs": ["ui/a.json", "", "ui/b.json"] }"#)
    );
    let mut emptied = list(r#"{ "ui_defs": ["ui/a.json"] }"#);
    emptied.ui_defs.clear();
    assert_eq!(emptied.to_json().unwrap(), "{}\n");
}

#[test]
fn the_determinism_fragment_loads_clean() {
    let dir = fixtures().join("determinism/resource_pack");
    let read = |path: &str| std::fs::read_to_string(dir.join(path)).unwrap();
    let stack = stack_of(&[&[
        ("ui/_ui_defs.json", &read("ui/_ui_defs.json")),
        (
            "ui/_global_variables.json",
            &read("ui/_global_variables.json"),
        ),
        (
            "ui/determinism_screen.json",
            &read("ui/determinism_screen.json"),
        ),
    ]]);
    assert!(findings(&stack).is_empty(), "{:?}", findings(&stack));
    assert!(
        stack
            .ui()
            .get("determinism", "determinism_screen")
            .is_some()
    );
}

/// `text` as the writer lays JSON out: what a write-back that changes nothing gives.
fn json_text(text: &str) -> String {
    bedrock_addon::json::to_json(&json(text)).unwrap()
}

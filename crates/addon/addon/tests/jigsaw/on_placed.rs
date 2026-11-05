//! `minecraft:jigsaw.on_placed`: a string member of the 1.21.20
//! `Contents`, the
//! name of a callback the game reserves a token for; absent is `""` and nothing is written; any name loads.

use bedrock_addon::families::jigsaw::JigsawStructure;

use super::common::{fixture, loads, rejected};

/// The structure payload of an engine fixture, embedded in a 1.21.20
/// document.
fn engine_document(name: &str) -> String {
    let payload = String::from_utf8(fixture(&format!("engine/{name}.json"))).unwrap();
    format!(r#"{{"format_version": "1.21.20", "minecraft:jigsaw": {payload}}}"#)
}

/// Loads, writes and loads again; the value and the second write are the same.
#[track_caller]
fn round_trip(text: &str) -> String {
    let first = loads::<JigsawStructure>("worldgen/structures/a.json", text);
    assert!(first.diagnostics().is_empty(), "{:#?}", first.diagnostics());
    let written = first.to_json().unwrap();
    let again = loads::<JigsawStructure>("worldgen/structures/a.json", &written);
    assert_eq!(again.value(), first.value());
    assert_eq!(again.to_json().unwrap(), written);
    written
}

#[test]
fn the_parser_test_s_structure_reads_its_on_placed_callback() {
    // The game's own parser test for a jigsaw structure.
    let text = engine_document("parser-7");
    let loaded = loads::<JigsawStructure>("worldgen/structures/a.json", &text);
    assert_eq!(
        loaded.on_placed,
        Some("test:on_trail_ruins_placed".to_owned())
    );
    assert_eq!(loaded.on_placed(), "test:on_trail_ruins_placed");
    assert_eq!(loaded.max_depth, 7);
    let written = round_trip(&text);
    assert!(
        written.contains(r#""on_placed": "test:on_trail_ruins_placed""#),
        "{written}"
    );
}

#[test]
fn the_unit_test_library_s_structure_reads_its_on_placed_callback() {
    let text = engine_document("case-4");
    let loaded = loads::<JigsawStructure>("worldgen/structures/a.json", &text);
    assert_eq!(loaded.on_placed(), "test:on_jigsaw_placed");
    assert_eq!(loaded.start_jigsaw_name(), "test_start_block");
    round_trip(&text);
}

#[test]
fn an_absent_on_placed_is_the_empty_name_and_is_not_written() {
    let text =
        engine_document("parser-7").replace(r#""on_placed": "test:on_trail_ruins_placed","#, "");
    let loaded = loads::<JigsawStructure>("worldgen/structures/a.json", &text);
    assert_eq!(loaded.on_placed, None);
    assert_eq!(loaded.on_placed(), "");
    let written = round_trip(&text);
    assert!(!written.contains("on_placed"), "{written}");
    assert!(!written.contains("minecraft:no_op"), "{written}");
}

#[test]
fn on_placed_null_or_not_a_string_rejects_the_structure() {
    for value in ["null", "3", "true", r#"["a:b"]"#] {
        let text = engine_document("parser-7").replace(
            r#""on_placed": "test:on_trail_ruins_placed""#,
            &format!(r#""on_placed": {value}"#),
        );
        let d = rejected::<JigsawStructure>("worldgen/structures/a.json", &text);
        let at: Vec<(String, String)> = d
            .iter()
            .filter(|f| f.severity == bedrock_addon::diagnostic::Severity::Error)
            .filter(|f| f.code.name() != "DocumentRejected")
            .map(|f| (f.code.name().to_owned(), f.location.json_path.to_string()))
            .collect();
        // An array is reported at its first element, as facet places a wrong type below a
        // string member (the error is the member's).
        assert_eq!(at.len(), 1, "{value}: {at:?}");
        assert_eq!(at[0].0, "InvalidType", "{value}");
        assert!(
            at[0].1.starts_with("minecraft:jigsaw/on_placed"),
            "{value}: {}",
            at[0].1
        );
    }
}

#[test]
fn any_name_loads_and_none_is_checked() {
    for name in ["", "x", "minecraft:no_op", "not a namespaced name"] {
        let text = engine_document("parser-7").replace("test:on_trail_ruins_placed", name);
        let loaded = loads::<JigsawStructure>("worldgen/structures/a.json", &text);
        assert_eq!(loaded.on_placed(), name);
        assert!(loaded.diagnostics().is_empty());
    }
}

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(all(feature = "block", feature = "spawn_rules", feature = "spawn_groups"))]
//! A number no double holds in a member written twice: every copy's number is
//! clamped in the text the family reads, and the information is given for the copy the family
//! keeps: the LAST everywhere a `JsonCpp` reader reads. A block refuses the file
//! instead.

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::document::Content;
use bedrock_addon::families::block::Block;
use bedrock_addon::families::spawn_rules::SpawnRules;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

/// A loaded file: what is written back, and its findings as `(code, path, text under the
/// span)`.
fn load<T: Content>(path: &'static str, text: &str) -> (String, Vec<(String, String, String)>) {
    let cx = LoadContext::detached();
    let document = T::parse(&PackPath::from_static(path), text.as_bytes(), &cx)
        .unwrap_or_else(|rejected| panic!("the file loads: {:#?}", rejected.diagnostics));
    let written = document.to_json().expect("a loaded file is written");
    let found = document
        .diagnostics()
        .iter()
        .map(|finding: &Diagnostic| {
            let code: String = finding.code.id().to_owned();
            let under = finding.location.span.map_or(String::new(), |span| {
                let start = span.offset as usize;
                text[start..start + span.len as usize].to_owned()
            });
            (code, finding.location.json_path.to_string(), under)
        })
        .collect();
    (written, found)
}

fn block(components: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:block": {{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}}}"#
    )
}

fn of(code: &str, path: &str, under: &str) -> (String, String, String) {
    (code.to_owned(), path.to_owned(), under.to_owned())
}

/// A block is a `MinecraftDocumentInput`: the game's JSON reader refuses a number no double holds in either
/// copy of a component written twice, and the file is not loaded (a block is not clamped).
#[test]
fn a_number_no_double_holds_refuses_a_block_whichever_copy_holds_it() {
    for components in [
        r#""demo:zz": {"a": 1e999}, "demo:zz": {"a": 0}"#,
        r#""demo:zz": {"a": 0}, "demo:zz": {"a": -1e999}"#,
    ] {
        let text = block(components);
        let cx = LoadContext::detached();
        let rejected = Block::parse(
            &PackPath::from_static("blocks/x.json"),
            text.as_bytes(),
            &cx,
        )
        .expect_err("the file is refused");
        let found: Vec<(String, String)> = rejected
            .diagnostics
            .iter()
            .map(|d| (d.code.id().to_owned(), d.code.to_string()))
            .collect();
        assert_eq!(
            found,
            [(
                "core/document_json".to_owned(),
                "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
                    .to_owned()
            )],
            "{components}"
        );
    }
}

fn spawn_rules(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "minecraft:spawn_rules": {{"description": {{"identifier": "demo:x", "population_control": "animal"}}, "conditions": [{{"minecraft:spawns_on_surface": {{}}}}]}}, {members}}}"#
    )
}

#[test]
fn everywhere_else_the_last_copy_stands_and_only_its_number_is_told() {
    let text = spawn_rules(r#""zz": {"a": 1e999}, "zz": {"a": 0}"#);
    let (written, found) = load::<SpawnRules>("spawn_rules/x.json", &text);
    assert!(!written.contains("e308"), "{written}");
    let invalid: Vec<_> = found
        .iter()
        .filter(|f| f.0 == "core/invalid_value")
        .collect();
    assert!(invalid.is_empty(), "{found:?}");

    let text = spawn_rules(r#""zz": {"a": 0}, "zz": {"a": -1e999}"#);
    let (written, found) = load::<SpawnRules>("spawn_rules/x.json", &text);
    assert!(written.contains("-1.7976931348623157e308"), "{written}");
    let invalid: Vec<_> = found
        .iter()
        .filter(|f| f.0 == "core/invalid_value")
        .collect();
    assert_eq!(invalid, [&of("core/invalid_value", "zz/a", "-1e999")]);
}

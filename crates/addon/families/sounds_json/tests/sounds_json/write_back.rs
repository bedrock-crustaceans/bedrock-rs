//! A `sounds.json` file written back: what it read, and what its author wrote, at the Value
//! level.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_sounds_json::SoundBindings;

use bedrock_addon_test_support::json::strict as json;

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

/// Parse, write, parse: equal values, and the second write is the first byte for byte.
#[track_caller]
fn round_trips<T: Content + PartialEq + std::fmt::Debug>(at: &str, text: &str) -> T {
    let cx = LoadContext::detached();
    let first = T::parse(&path(at), text.as_bytes(), &cx).expect("the file loads");
    let written = first.to_json().unwrap();
    let second = T::parse(&path(at), written.as_bytes(), &cx).expect("the rewrite loads");
    assert_eq!(first.value(), second.value());
    assert_eq!(second.to_json().unwrap(), written);
    first.into_value()
}

#[test]
fn bindings_write_back_what_they_read() {
    // A file the per-pack transform touches in three places: `entity_sounds` without
    // `entities`, an interactive section without `entities`, and nothing else.
    let text = r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone", "volume": [0.5, 1], "extra": 1}}, "pitch": 1}},
        "entity_sounds": {"defaults": {"events": {"ambient": "mob.pig.say"}}},
        "interactive_sounds": {"block_sounds": {}}, "unknown": [1, 2]}"#;
    let bindings: SoundBindings = round_trips("sounds.json", text);
    assert_eq!(bindings.default_path().as_str(), "sounds.json");
    assert_eq!(*bindings.json(), json(text));
    assert_eq!(SoundBindings::from_json(json(text)), bindings);
}

#[test]
fn a_single_sounds_json_writes_back_what_its_author_wrote() {
    // The per-pack transform is the merge's input and the typed read's; none of what it
    // inserts is the author's text.
    let cx = LoadContext::detached();
    for text in [
        r#"{"entity_sounds": {"defaults": {"events": {}}}}"#,
        r#"{"entity_sounds": {"entities": {"cat": {"events": {}, "variants": {"key": "'a'", "map": {"a": {"events": {}}}}}}}}"#,
        r#"{"entity_sounds": {"entities": {"cat": {"events": {}, "variants": {"map": {}}}}}}"#,
        r#"{"interactive_sounds": {"block_sounds": {}}}"#,
        r#"{"entity_sounds": {"entities": {"cat": {"__molang_version": 4, "events": {}, "variants": {"key": "'a'", "map": {"a": {"events": {}}}}}}}}"#,
    ] {
        let document = SoundBindings::parse(&path("sounds.json"), text.as_bytes(), &cx)
            .expect("the file loads");
        assert_eq!(*document.json(), json(text), "{text}");
        let written: Value = facet_json::from_str(&document.to_json().unwrap()).unwrap();
        assert_eq!(written, json(text), "{text}");
    }
    // The typed read is still the transformed copy's: the pack's version on a keyed entity,
    // over the author's.
    let keyed = r#"{"entity_sounds": {"entities": {"cat": {"__molang_version": 4, "events": {}, "variants": {"key": "'a'", "map": {"a": {"events": {}}}}}}}}"#;
    let document = SoundBindings::parse(&path("sounds.json"), keyed.as_bytes(), &cx).unwrap();
    assert_eq!(
        document.entity_sounds.entities["cat"].molang_version,
        Some(i32::from(cx.molang_version.0))
    );
    assert_ne!(cx.molang_version.0, 4);
}

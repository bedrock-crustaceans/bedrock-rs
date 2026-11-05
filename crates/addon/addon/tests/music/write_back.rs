//! A music definitions file written back: what it read, byte for byte at the Value level.

use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::music::MusicDefinitions;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::json;

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
fn music_writes_back_what_it_reads() {
    let text = r#"{"game": {"event_name": "music.game", "min_delay": 60, "max_delay": 180.5, "replace": false, "odd": null}, "format_version": "1.0.0"}"#;
    let music: MusicDefinitions = round_trips("sounds/music_definitions.json", text);
    assert_eq!(*music.json(), json(text));
    assert_eq!(
        music.default_path().as_str(),
        "sounds/music_definitions.json"
    );
    assert_eq!(MusicDefinitions::from_json(json(text)), music);
}

#[test]
fn a_single_music_file_writes_back_as_read() {
    let cx = LoadContext::detached();
    let music = r#"{"game": {"min_delay": 1}, "replace": true, "": {}}"#;
    let document = MusicDefinitions::parse(
        &path("sounds/music_definitions.json"),
        music.as_bytes(),
        &cx,
    )
    .unwrap();
    assert_eq!(*document.json(), json(music));
}

#[test]
fn a_delay_beyond_f32_is_written_as_the_largest_double() {
    // Both read as infinity through the f32 the client reads; the write gives the largest
    // double, which the client's float cast makes infinity again.
    let text =
        r#"{"game": {"min_delay": 1e39, "max_delay": 1e999}, "nether": {"min_delay": -1e39}}"#;
    let music: MusicDefinitions = round_trips("sounds/music_definitions.json", text);
    let game = music.get("game").unwrap();
    assert_eq!(game.min_delay, f32::INFINITY);
    assert_eq!(game.max_delay, f32::INFINITY);
    let written = music.to_json().unwrap();
    assert!(
        written.contains("\"min_delay\": 1.7976931348623157e308"),
        "{written}"
    );
    assert!(
        written.contains("\"min_delay\": -1.7976931348623157e308"),
        "{written}"
    );
}

#[test]
fn a_delay_no_json_holds_is_refused_at_its_member() {
    use bedrock_addon::diagnostic::JsonPath;
    use bedrock_addon::json::EncodeError;
    let mut music = MusicDefinitions::from_json(json(r#"{"game": {"min_delay": 1}}"#));
    music.definitions[0].max_delay = f32::NAN;
    let error = music.to_json().unwrap_err();
    assert!(
        matches!(&error, EncodeError::NotRepresentable { at, .. } if *at == JsonPath::of_keys(["game", "max_delay"])),
        "{error:?}"
    );
}

/// The value read from `text`, written: the text it writes, read back as JSON.
fn written_json(text: &str) -> bedrock_addon::json::Value {
    let cx = LoadContext::detached();
    let document =
        MusicDefinitions::parse(&path("sounds/music_definitions.json"), text.as_bytes(), &cx)
            .expect("the file loads");
    json(&document.to_json().expect("written"))
}

#[test]
fn an_object_form_override_interruptible_is_written_as_written() {
    // The merge recurses into an object and replaces an array whole: an object written as
    // an array would replace a lower pack's entries it merged into before.
    let text = r#"{"game": {"override_interruptible": {"b": {"name": "y"}, "a": {"name": "x"}}}}"#;
    let music: MusicDefinitions = round_trips("sounds/music_definitions.json", text);
    assert_eq!(
        music.get("game").unwrap().override_interruptible,
        ["x", "y"]
    );
    assert_eq!(written_json(text), json(text));
}

#[test]
fn a_root_that_is_no_object_is_written_as_written() {
    for text in ["[]", "[{\"game\": {}}]", "5", "\"music\"", "true"] {
        assert_eq!(written_json(text), json(text), "{text}");
    }
}

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

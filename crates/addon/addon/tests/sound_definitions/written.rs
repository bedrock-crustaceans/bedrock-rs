//! The write-back of `sounds/sound_definitions.json`, pinned in
//! `sound_definitions/written.pins`: what `to_json` writes for every fixture of the file and
//! for probes of each member and version, and after edits of the typed value
//! (`common/written_pins.rs`).

use bedrock_addon::families::sound_definitions::{SoundDefinition, SoundDefinitions};
use bedrock_addon::id::SoundEventName;

use super::written_pins::{assert_as_pinned, edited, fixtures, render, written};

const PATH: &str = "sounds/sound_definitions.json";

const PIG: &str = r#"{"format_version": "1.26.50", "sound_definitions": {"mob.pig.say": {"category": "neutral", "min_distance": 1, "max_distance": 16.0, "subtitle": "s", "sounds": ["a", {"name": "b", "volume": 0.5, "pitch": 2, "weight": 3, "is3D": false, "stream": true, "interruptible": false, "allow_concurrent_streaming": false, "load_on_low_memory": true}, null, 5]}}}"#;

/// Documents that reach every member of a definition, of an entry, of the root and every
/// version step.
const PROBES: &[&str] = &[
    PIG,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"category": "ui", "external_event_name": "e", "external_event_parameters": [{"parameter": "p", "value": 1}, {"parameter": "q", "value": 0.5, "z": 1}, 3]}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"external_event_name": "e", "sounds": ["a"]}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"external_event_name": 5, "sounds": ["a"], "external_event_parameters": [{"parameter": "p"}]}, "y": {"external_event_name": null, "sounds": ["b"]}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"actions": [{"action_type": 1, "name": "n", "target": "t", "enabled": true, "param": 0.5, "delay": 1, "o": 2}, 4, {"action_type": 1.5}]}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"category": 5, "min_distance": "3", "max_distance": null, "subtitle": [], "__use_legacy_max_distance": "true", "sounds": "a"}}}"#,
    r#"{"format_version": "1.26.50", "stray": 1, "sound_definitions": {"x": 5, "y": {}, "music.z": {"category": "ui", "volume": 1, "pitch": 1}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": 5}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": null}"#,
    r#"{"format_version": "1.26.50"}"#,
    r#"{"format_version": [1, 26, 50], "sound_definitions": {"x": {"sounds": [{"name": "a", "stream": true}]}}}"#,
    r#"{"format_version": "1.26.60", "sound_definitions": {"x": {"sounds": [{"name": "a", "stream": true}, {"name": "b", "stream": true, "allow_concurrent_streaming": true}]}}}"#,
    r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"sounds": [{"name": "a", "stream": true}], "max_distance": 16}}}"#,
    r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"sounds": [{"name": "a", "pitch": 2}], "max_distance": 16}}}"#,
    r#"{"mob.pig.say": {"category": "neutral", "sounds": ["a"], "max_distance": 16, "min_distance": 2.5}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": {"b": "y", "a": {"name": "x", "volume": 0.25}}}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"category": "music", "sounds": ["a", {"name": "b", "is3D": true}, {"name": "c", "is3D": false}]}}}"#,
    r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"min_distance": 0.1, "max_distance": 1.7976931348623157e308}}}"#,
    "[]",
    "{",
];

fn pig() -> SoundEventName {
    SoundEventName::parse("mob.pig.say")
}

#[test]
fn sound_definitions_are_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    for (name, text) in fixtures(PATH) {
        cases.push((name, written::<SoundDefinitions>(PATH, &text)));
    }
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<SoundDefinitions>(PATH, probe)));
    }
    cases.push((
        "PIG, the second entry's volume set to 0.75 and the first's weight to 2, the category to \"hostile\"".to_owned(),
        edited::<SoundDefinitions>(PATH, PIG, |sounds| {
            let pig = sounds.sound_definitions.get_mut(&pig()).expect("the pig");
            pig.sounds[1].volume = 0.75;
            pig.sounds[0].weight = 2;
            pig.category = "hostile".to_owned();
        }),
    ));
    cases.push((
        "PIG, a definition x added and the pig removed".to_owned(),
        edited::<SoundDefinitions>(PATH, PIG, |sounds| {
            let mut x = SoundDefinition::default();
            x.category = "ui".to_owned();
            x.subtitle = "subtitle.x".to_owned();
            sounds
                .sound_definitions
                .insert(SoundEventName::parse("x"), x);
            sounds.sound_definitions.shift_remove(&pig());
        }),
    ));
    assert_as_pinned("sound_definitions/written.pins", &render(&cases));
}

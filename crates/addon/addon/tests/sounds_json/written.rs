//! The write-back of `sounds.json`, pinned in `sounds_json/written.pins`: what `to_json`
//! writes for every fixture of the file and for probes of each table, and after edits of the
//! typed value (`common/written_pins.rs`).

use bedrock_addon::families::sounds_json::SoundBindings;

use super::written_pins::{assert_as_pinned, edited, fixtures, render, written};

const PATH: &str = "sounds.json";

const STONE: &str = r#"{"block_sounds": {"stone": {"volume": 1, "pitch": [0.8, 1.2], "events": {"break": "dig.stone", "Hit": {"sound": "hit.stone", "volume": "[0.5, 1]", "sounds": 1}}}, "slab": {"base": "stone"}}}"#;

/// Documents that reach every table and every range form.
const PROBES: &[&str] = &[
    STONE,
    r#"{"individual_event_sounds": {"events": {"ambient.cave": "a", "not_an_event": "b", "hurt": {"sound": "c", "pitch": {"min": 0.5, "max": 0.25}, "x": 1}}, "volume": 0.5}}"#,
    r#"{"individual_named_sounds": {"sounds": {"b": "y", "a": {"sound": "x", "volume": true}}, "pitch": [2, 1]}}"#,
    r#"{"entity_sounds": {"defaults": {"events": {"ambient": "mob.pig.say"}, "volume": 0.5}, "entities": {"cat": {"events": {"hurt": 5}, "variants": {"key": "query.is_baby ? 'b' : 'a'", "map": {"b": {"events": {"ambient": "c"}}}}}}}}"#,
    r#"{"entity_sounds": {"entities": {"cat": {"__molang_version": 4, "events": {}, "variants": {"key": "'a'", "map": {"a": {"events": {}}}}}}}}"#,
    r#"{"interactive_sounds": {"block_sounds": {"stone": {"events": {"fall": {"default": "x"}}}}, "entity_sounds": {"defaults": {"events": {"step": {"default": "s", "stone": {"sound": "t", "pitch": 1.5}}}}, "entities": {"pig": {"events": {}}}}}}"#,
    r#"{"block_sounds": {"a": {"base": "b"}, "b": {"base": "a"}, "c": {"base": "nope"}, "d": 5}, "stray": [1, 2]}"#,
    r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "x", "volume": [], "pitch": null}}}}, "entity_sounds": 5, "interactive_sounds": []}"#,
    "{}",
    "[]",
    "{",
];

#[test]
fn sounds_json_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    for (name, text) in fixtures(&format!("/{PATH}")) {
        cases.push((name, written::<SoundBindings>(PATH, &text)));
    }
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<SoundBindings>(PATH, probe)));
    }
    cases.push((
        "STONE, the break sound renamed to \"dig.edited\" and stone's base set to \"slab\""
            .to_owned(),
        edited::<SoundBindings>(PATH, STONE, |sounds| {
            let stone = sounds.block_sounds.get_mut("stone").expect("stone");
            stone.base = "slab".to_owned();
            stone
                .sounds
                .events
                .get_mut("break")
                .expect("break")
                .sound = "dig.edited".to_owned();
        }),
    ));
    assert_as_pinned("sounds_json/written.pins", &render(&cases));
}

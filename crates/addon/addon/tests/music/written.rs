//! The write-back of `sounds/music_definitions.json`, pinned in `music/written.pins`: what
//! `to_json` writes for every fixture of the file and for probes of each member, and after
//! edits of the typed value (`common/written_pins.rs`).

use bedrock_addon::families::music::{MusicDefinition, MusicDefinitions};
use bedrock_addon::id::MusicDefinitionId;

use super::written_pins::{assert_as_pinned, edited, fixtures, render, written};

const PATH: &str = "sounds/music_definitions.json";

const GAME: &str = r#"{"game": {"event_name": "music.game", "min_delay": 60, "max_delay": 180}}"#;

/// Documents that reach every member of a definition and of the root.
const PROBES: &[&str] = &[
    GAME,
    r#"{"menu": {"event_name": "music.menu", "min_delay": 0.5, "max_delay": 1e2, "play_in_order": true, "override_interruptible": [{"name": "hostile"}, {"name": "ui", "note": 1}, 3, {}], "replace": false, "odd": null}}"#,
    r#"{"game": {"max_delay": 2, "odd": 1, "event_name": "e", "min_delay": 1}}"#,
    r#"{"game": {"event_name": 5, "min_delay": "3", "max_delay": [1], "play_in_order": 1, "override_interruptible": "x", "replace": "yes"}}"#,
    r#"{"game": {"event_name": null, "min_delay": null, "play_in_order": null, "override_interruptible": null, "replace": null}}"#,
    r#"{"game": {"event_name": "", "min_delay": 0, "max_delay": 0.0, "play_in_order": false, "override_interruptible": []}}"#,
    r#"{"format_version": "1.0.0", "": {"event_name": "x"}, "game": {}, "menu": 5, "end": [], "replace": true}"#,
    r#"{"game": {"min_delay": 0.1, "max_delay": 16, "event_name": "aé"}, "nether": {"min_delay": -0.0, "max_delay": 1.5e3}}"#,
    r#"{"game": {"min_delay": 3.4e38, "max_delay": 1.7976931348623157e308}}"#,
    r#"{"game": {"override_interruptible": {"b": {"name": "y"}, "a": {"name": "x"}}}}"#,
    r#"{"game": {"min_delay": 1, "min_delay": 2}} // a comment"#,
    "{}",
    "[]",
    "5",
    "{",
];

#[test]
fn music_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    for (name, text) in fixtures(PATH) {
        cases.push((name, written::<MusicDefinitions>(PATH, &text)));
    }
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<MusicDefinitions>(PATH, probe)));
    }
    let game = MusicDefinitionId::parse("game");
    cases.push((
        "GAME, min_delay set to 5 and event_name to \"music.edited\"".to_owned(),
        edited::<MusicDefinitions>(PATH, GAME, |music| {
            let definition = music.definitions.get_mut(&game).expect("game");
            definition.min_delay = 5.0;
            definition.event_name = "music.edited".to_owned();
        }),
    ));
    cases.push((
        "GAME, a definition menu added and game removed".to_owned(),
        edited::<MusicDefinitions>(PATH, GAME, |music| {
            let mut menu = MusicDefinition::default();
            menu.event_name = "music.menu".to_owned();
            menu.play_in_order = true;
            music
                .definitions
                .insert(MusicDefinitionId::parse("menu"), menu);
            music.definitions.shift_remove(&game);
        }),
    ));
    assert_as_pinned("music/written.pins", &render(&cases));
}

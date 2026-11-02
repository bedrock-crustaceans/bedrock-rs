//! `sounds/music_definitions.json`: the rows `md01`–`md09` of the fixtures and the rules of the
//! file (the vanilla file is the smoke test's).

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Merged, RawLayer};
use bedrock_addon::families::music::{MusicCategory, MusicDefinition, MusicDefinitions};
use bedrock_addon::id::MusicDefinitionId;

use super::common::{MUSIC, copies, describe, expect_clean, expect_one, found, json, pack_of};

const GAME: &str = r#"{"game": {"event_name": "music.game", "min_delay": 60, "max_delay": 180}}"#;

#[track_caller]
fn music<'a>(sounds: &'a Merged<MusicDefinitions, RawLayer>, name: &str) -> &'a MusicDefinition {
    sounds
        .get(name)
        .unwrap_or_else(|| panic!("{name} in {:#?}", describe(sounds)))
}

#[test]
fn md01_delays_are_stored_as_ticks() {
    let stack = copies(MUSIC, &[GAME]);
    let game = music(stack.music(), "game");
    assert_eq!((game.min_delay, game.max_delay), (60.0, 180.0));
    assert_eq!(
        (game.min_delay_ticks(), game.max_delay_ticks()),
        (1200, 3600)
    );
    assert!(!game.play_in_order);
    expect_clean(stack.music());
    // Truncation, not rounding.
    let stack = copies(
        MUSIC,
        &[r#"{"game": {"min_delay": 0.99, "max_delay": 1.07}}"#],
    );
    let game = music(stack.music(), "game");
    assert_eq!((game.min_delay_ticks(), game.max_delay_ticks()), (19, 21));
}

#[test]
fn md02_merge_is_field_wise() {
    let stack = copies(MUSIC, &[GAME, r#"{"game": {"min_delay": 10}}"#]);
    let sounds = stack.music();
    let game = music(sounds, "game");
    assert_eq!(game.event_name, "music.game");
    assert_eq!(
        (game.min_delay_ticks(), game.max_delay_ticks()),
        (200, 3600)
    );
    expect_clean(sounds);
    let origin = sounds.origin(&MusicDefinitionId::parse("game")).unwrap();
    assert_eq!(origin.pack.index(), Some(0));
}

#[test]
fn md03_replace_drops_the_lower_definition() {
    let stack = copies(
        MUSIC,
        &[GAME, r#"{"game": {"replace": true, "event_name": "x"}}"#],
    );
    let sounds = stack.music();
    let game = music(sounds, "game");
    assert_eq!(game.event_name, "x");
    assert_eq!((game.min_delay, game.max_delay), (0.0, 0.0));
    assert_eq!(game.replace, Some(json("true")));
    assert_eq!(
        *sounds.json(),
        json(r#"{"game": {"replace": true, "event_name": "x"}}"#)
    );
    expect_clean(sounds);
}

#[test]
fn a_false_or_absent_replace_merges() {
    for over in [
        r#"{"game": {"replace": false, "event_name": "x"}}"#,
        r#"{"game": {"replace": 0, "event_name": "x"}}"#,
        r#"{"game": {"replace": "", "event_name": "x"}}"#,
        r#"{"game": {"replace": null, "event_name": "x"}}"#,
        r#"{"game": {"event_name": "x"}}"#,
    ] {
        let stack = copies(MUSIC, &[GAME, over]);
        let game = music(stack.music(), "game");
        assert_eq!(
            (game.event_name.as_str(), game.max_delay),
            ("x", 180.0),
            "{over}"
        );
    }
}

#[test]
fn md04_a_root_replace_drops_every_lower_definition() {
    let stack = copies(
        MUSIC,
        &[
            r#"{"game": {"event_name": "g"}, "end": {"event_name": "e"}}"#,
            r#"{"replace": true, "menu": {"event_name": "m"}}"#,
        ],
    );
    let sounds = stack.music();
    let names: Vec<&str> = sounds.definitions.keys().map(|k| k.as_str()).collect();
    assert_eq!(names, ["menu"]);
    // The `replace` member itself is no definition (rule 23).
    let skipped = found(sounds, "InvalidValue");
    assert_eq!(skipped.len(), 1, "{:#?}", describe(sounds));
    assert_eq!(skipped[0].severity, Severity::Info);
    assert_eq!(skipped[0].location.json_path.to_string(), "replace");
    assert_eq!(pack_of(skipped[0]), Some(0));
}

#[test]
fn md05_an_empty_name_is_skipped() {
    let stack = copies(
        MUSIC,
        &[r#"{"": {"event_name": "x"}, "game": {"event_name": "g"}}"#],
    );
    let sounds = stack.music();
    assert_eq!(sounds.definitions.len(), 1);
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Warning,
        "Cannot define a music definition with no name.",
    );
}

#[test]
fn md06_categories_come_from_the_name() {
    let expected = [
        ("menu", Some(MusicCategory::Menu)),
        ("game", Some(MusicCategory::Overworld)),
        ("creative", Some(MusicCategory::Overworld)),
        ("water", Some(MusicCategory::Overworld)),
        ("nether", Some(MusicCategory::Nether)),
        ("end", Some(MusicCategory::End)),
        ("endboss", Some(MusicCategory::Endboss)),
        ("credits", Some(MusicCategory::Credits)),
        ("bamboo_jungle", None),
        ("Menu", None),
    ];
    for (name, category) in expected {
        assert_eq!(MusicCategory::from_name(name), category, "{name}");
    }
    let numbers: Vec<u8> = expected[..8]
        .iter()
        .map(|(_, category)| category.unwrap() as u8)
        .collect();
    assert_eq!(numbers, [0, 1, 1, 1, 2, 3, 4, 5]);
}

#[test]
fn md07_override_interruptible() {
    let stack = copies(
        MUSIC,
        &[
            r#"{"game": {"play_in_order": true, "override_interruptible": [{"name": "record"}, "bare", {"other": 1}]}}"#,
        ],
    );
    let sounds = stack.music();
    let game = music(sounds, "game");
    assert!(game.play_in_order);
    // An entry that is no object, or has no name, is `""`.
    assert_eq!(game.override_interruptible, ["record", "", ""]);
    expect_clean(sounds);
}

#[test]
fn md08_an_entry_that_is_no_object_is_skipped_deviation() {
    // vanilla = "a Music with event_name \"\", delays 0 and play_in_order false is registered".
    let stack = copies(
        MUSIC,
        &[r#"{"format_version": "1.0.0", "game": {"event_name": "g"}}"#],
    );
    let sounds = stack.music();
    assert_eq!(sounds.definitions.len(), 1);
    assert!(sounds.get("format_version").is_none());
    let skipped = found(sounds, "InvalidValue");
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].location.json_path.to_string(), "format_version");
    assert_eq!(skipped[0].severity, Severity::Info);
}

#[test]
fn md09_a_non_empty_string_replaces() {
    let stack = copies(
        MUSIC,
        &[GAME, r#"{"game": {"replace": "false", "min_delay": 5}}"#],
    );
    let sounds = stack.music();
    let game = music(sounds, "game");
    assert_eq!(game.event_name, "");
    assert_eq!((game.max_delay, game.min_delay_ticks()), (0.0, 100));
    let paths: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    // Rule 28 for the string, and rule 24: the replaced definition has no `max_delay` left.
    assert_eq!(paths, ["game/replace", "game/min_delay"]);
    // A number replaces too.
    let stack = copies(MUSIC, &[GAME, r#"{"game": {"replace": 1}}"#]);
    assert_eq!(music(stack.music(), "game").max_delay, 0.0);
}

#[test]
fn a_minimum_above_the_maximum_is_kept_with_a_warning() {
    let stack = copies(MUSIC, &[r#"{"game": {"min_delay": 9, "max_delay": 3}}"#]);
    let sounds = stack.music();
    assert_eq!(
        (
            music(sounds, "game").min_delay,
            music(sounds, "game").max_delay
        ),
        (9.0, 3.0)
    );
    expect_one(sounds, "InvalidValue", Severity::Info, "game/min_delay");
    // Equal bounds are fine.
    expect_clean(copies(MUSIC, &[r#"{"game": {"min_delay": 3, "max_delay": 3}}"#]).music());
}

#[test]
fn music_members_coerce_as_the_fork_does() {
    let stack = copies(
        MUSIC,
        &[
            r#"{"game": {"event_name": 5, "min_delay": "1", "max_delay": null, "play_in_order": "no", "odd": 1}}"#,
        ],
    );
    let sounds = stack.music();
    let game = music(sounds, "game");
    assert_eq!(game.event_name, "");
    assert_eq!((game.min_delay, game.max_delay), (0.0, 0.0));
    assert!(game.play_in_order);
    assert_eq!(
        found(sounds, "InvalidValue").len(),
        3,
        "{:#?}",
        describe(sounds)
    );
    expect_one(sounds, "UnrecognizedField", Severity::Info, "game/odd");
}

#[test]
fn an_unparsable_music_base_names_its_path() {
    let stack = copies(MUSIC, &["nope", GAME]);
    let sounds = stack.music();
    assert!(sounds.definitions.is_empty());
    expect_one(
        sounds,
        "MalformedInput",
        Severity::Error,
        "Could not parse file at the top of the stack: sounds/music_definitions.json.",
    );
}

#[test]
fn no_file_in_the_stack_is_an_empty_document_without_findings() {
    let stack = copies("textures/readme.txt", &["x"]);
    let sounds = stack.music();
    expect_clean(sounds);
    assert!(sounds.definitions.is_empty());
}

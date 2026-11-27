//! The sound events a music definition names, judged by the link pass: the `event_name` of a
//! definition is a sound definition of the category `music`.

use bedrock_addon::diagnostic::{Kind, Phase, Severity};

use super::common::{BINDINGS, DEFINITIONS, MUSIC, found, linked_user, pack};

const TWO_EVENTS: &str = r#"{"format_version": "1.20.20", "sound_definitions": {
    "music.game": {"category": "music"}, "dig.stone": {"category": "block"}}}"#;

#[test]
fn rule_25_music_names_a_music_definition() {
    let link = linked_user(vec![pack(
        10,
        &[
            (DEFINITIONS, TWO_EVENTS),
            (BINDINGS, "{}"),
            (
                MUSIC,
                r#"{"game": {"event_name": "music.game"}, "end": {"event_name": "music.missing"},
            "menu": {"event_name": "dig.stone"}, "water": {}}"#,
            ),
        ],
    )]);
    let unknown: Vec<String> = found(&link, "core/unknown_sound")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    // Missing, and present under another category; an empty name is not checked.
    assert_eq!(unknown, ["end/event_name", "menu/event_name"]);
    assert!(
        found(&link, "core/unknown_sound")
            .iter()
            .all(|d| d.severity == Severity::Info && d.phase == Phase::Link)
    );
}

#[test]
fn nothing_is_cross_checked_without_a_sound_definitions_file() {
    let link = linked_user(vec![pack(
        10,
        &[(MUSIC, r#"{"game": {"event_name": "music.game"}}"#)],
    )]);
    assert!(
        link.iter()
            .all(|d| d.code.kind() != Kind::UnresolvedReference),
        "no unresolved reference in {link:#?}"
    );
}

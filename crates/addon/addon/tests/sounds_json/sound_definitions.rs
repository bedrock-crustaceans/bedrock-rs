//! The sounds the bindings of `sounds.json` name against the sound definitions of the stack,
//! judged by the link pass: a bound sound no definition defines is an `UnknownSound`.

use bedrock_addon::diagnostic::{Phase, Severity};

use super::common::{BINDINGS, DEFINITIONS, expect_none, found, linked_user, pack};

const TWO_EVENTS: &str = r#"{"format_version": "1.20.20", "sound_definitions": {
    "music.game": {"category": "music"}, "dig.stone": {"category": "block"}}}"#;

#[test]
fn rule_26_bound_sounds_are_sound_definitions() {
    let link = linked_user(vec![pack(
        10,
        &[
            (DEFINITIONS, TWO_EVENTS),
            (
                BINDINGS,
                r#"{"individual_event_sounds": {"events": {"break": "dig.stone", "hit": "dig.missing", "step": ""}},
            "block_sounds": {"stone": {"events": {"break": {"sound": "nope"}}}},
            "entity_sounds": {"defaults": {"events": {"ambient": "nope2"}},
                "entities": {"pig": {"events": {"ambient": "dig.stone"},
                    "variants": {"key": "'v'", "map": {"v": {"events": {"ambient": "nope3"}}}}}}},
            "interactive_sounds": {"entity_sounds": {"defaults": {"events": {"step": {"default": "nope4"}}}}}}"#,
            ),
        ],
    )]);
    let unknown: Vec<String> = found(&link, "UnknownSound")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        unknown,
        [
            "individual_event_sounds/events/hit",
            "block_sounds/stone/events/break",
            "entity_sounds/defaults/events/ambient",
            "entity_sounds/entities/pig/variants/map/v/events/ambient",
            "interactive_sounds/entity_sounds/defaults/events/step/default"
        ]
    );
    assert!(
        found(&link, "UnknownSound")
            .iter()
            .all(|d| d.severity == Severity::Info && d.phase == Phase::Link)
    );
}

#[test]
fn nothing_is_cross_checked_without_a_sound_definitions_file() {
    let link = linked_user(vec![pack(
        10,
        &[(
            BINDINGS,
            r#"{"individual_event_sounds": {"events": {"break": "dig.stone"}}}"#,
        )],
    )]);
    expect_none(&link, "UnknownSound");
}

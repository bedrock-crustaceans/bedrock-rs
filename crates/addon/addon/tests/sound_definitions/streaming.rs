//! The 1.26.50 step of `sounds/sound_definitions.json` and the entry member it serves,
//! `allow_concurrent_streaming`, against the
//! game's own files: the vanilla 1.26.60 and 1.26.40 resource packs and the ten documents the
//! 1.26.60 game embeds for its functional tests (`tests/fixtures/sound/sounds/`).

use bedrock_addon::document::Content;
use bedrock_addon::families::sound_definitions::SoundDefinitions;
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::SemVersion;

use super::common::{DEFINITIONS, copies, describe, expect_clean, json};

/// `resource_packs/vanilla_1.26.60/sounds/sound_definitions.json`, format 1.26.60: no step runs.
const VANILLA_1_26_60: &str =
    include_str!("../fixtures/sound/sounds/vanilla/sound_definitions.json");
/// `resource_packs/vanilla_1.26.40/sounds/sound_definitions.json`, format 1.20.20: the 1.26.50
/// step runs over its streamed music entries.
const VANILLA_1_26_40: &str =
    include_str!("../fixtures/sound/sounds/vanilla_1_26_40/sound_definitions.json");

/// The ten embedded functional-test documents, all format 1.14.0.
const FUNCTIONAL: &[(&str, &str)] = &[
    (
        "update_sound_data-1",
        include_str!("../fixtures/sound/sounds/functional/update_sound_data-1.json"),
    ),
    (
        "player-1",
        include_str!("../fixtures/sound/sounds/functional/player-1.json"),
    ),
    (
        "player-2",
        include_str!("../fixtures/sound/sounds/functional/player-2.json"),
    ),
    (
        "player-3",
        include_str!("../fixtures/sound/sounds/functional/player-3.json"),
    ),
    (
        "player-4",
        include_str!("../fixtures/sound/sounds/functional/player-4.json"),
    ),
    (
        "player-5",
        include_str!("../fixtures/sound/sounds/functional/player-5.json"),
    ),
    (
        "player-6",
        include_str!("../fixtures/sound/sounds/functional/player-6.json"),
    ),
    (
        "player-7",
        include_str!("../fixtures/sound/sounds/functional/player-7.json"),
    ),
    (
        "player-8",
        include_str!("../fixtures/sound/sounds/functional/player-8.json"),
    ),
    (
        "player-9",
        include_str!("../fixtures/sound/sounds/functional/player-9.json"),
    ),
];

/// Reads one pack's copy on its own, as `Content::parse` does.
#[track_caller]
fn parse(text: &str) -> bedrock_addon::document::Document<SoundDefinitions> {
    SoundDefinitions::parse(
        &PackPath::new(DEFINITIONS).unwrap(),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("the file loads: {rejected:#?}"))
}

/// The entries of every definition's `sounds`, in file order.
fn entries(document: &Value) -> Vec<&Value> {
    let definitions = document
        .as_object()
        .and_then(|root| root.get("sound_definitions"))
        .and_then(Value::as_object)
        .expect("a sound_definitions object");
    definitions
        .iter()
        .filter_map(|(_, definition)| definition.as_object()?.get("sounds")?.as_array())
        .flat_map(|sounds| sounds.as_slice().iter())
        .collect()
}

/// The member `key` of an object entry.
fn member<'a>(entry: &'a Value, key: &str) -> Option<&'a Value> {
    entry.as_object()?.get(key)
}

#[test]
fn the_1_26_50_step_turns_off_concurrent_streaming_of_an_older_streamed_entry() {
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{ "format_version": "1.20.20", "sound_definitions": { "music.x": { "category": "music",
  "sounds": [ { "name": "sounds/music/x", "stream": true }, "sounds/music/y" ] } } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(
            r#"{ "format_version": "1.26.50", "sound_definitions": { "music.x": { "category": "music",
  "sounds": [ { "name": "sounds/music/x", "stream": true, "allow_concurrent_streaming": false }, "sounds/music/y" ] } } }"#
        ),
        "{:#?}",
        describe(sounds)
    );
    assert_eq!(sounds.format_version, Some(SemVersion::new(1, 26, 50)));
    // The step says nothing ("no message is logged").
    expect_clean(sounds);
}

#[test]
fn vanilla_1_26_40_streamed_entries_get_allow_concurrent_streaming_false() {
    let written = json(VANILLA_1_26_40);
    let document = parse(VANILLA_1_26_40);
    let upgraded = document.json();
    assert_eq!(
        member(upgraded, "format_version"),
        Some(&json(r#""1.26.50""#))
    );
    let (before, after) = (entries(&written), entries(upgraded));
    assert_eq!(before.len(), after.len());
    let mut streamed = 0;
    for (old, new) in before.iter().zip(&after) {
        if member(old, "stream") == Some(&Value::from(true)) {
            streamed += 1;
            assert_eq!(
                member(new, "allow_concurrent_streaming"),
                Some(&Value::from(false)),
                "{new:?}"
            );
        } else {
            assert_eq!(old, new, "an entry that does not stream is left as it is");
        }
    }
    assert_eq!(streamed, 58, "the 1.26.40 pack streams 58 music entries");
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn vanilla_1_26_60_reads_allow_concurrent_streaming_as_a_member() {
    let document = parse(VANILLA_1_26_60);
    // Format 1.26.60 runs no step: the document is what the pack wrote.
    assert_eq!(*document.json(), json(VANILLA_1_26_60));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    for name in ["cauldron.cleanshulkerbox", "cauldron.dyeshulkerbox"] {
        let entry = &document.get(name).expect(name).sounds[0];
        // Read, so not one of the members the client ignores.
        assert!(
            entry.extra.get("allow_concurrent_streaming").is_none(),
            "{name}: {entry:?}"
        );
        assert!(entry.allow_concurrent_streaming, "{name}");
    }
    // A path string does not say: `true`.
    let click = &document.get("block.comparator.click").unwrap().sounds[0];
    assert!(click.allow_concurrent_streaming);
}

#[test]
fn vanilla_1_26_40_streamed_entries_read_as_not_streaming_concurrently() {
    let document = parse(VANILLA_1_26_40);
    let mut streamed = 0;
    for definition in document.sound_definitions.values() {
        for entry in &definition.sounds {
            assert_eq!(entry.allow_concurrent_streaming, !entry.stream, "{entry:?}");
            streamed += usize::from(entry.stream);
        }
    }
    assert_eq!(streamed, 58);
}

#[test]
fn allow_concurrent_streaming_defaults_to_true_and_reads_as_written_from_1_26_50() {
    let document = parse(
        r#"{"format_version": "1.26.50", "sound_definitions": {"music.x": {"category": "music", "sounds": [
            {"name": "a", "stream": true},
            {"name": "b", "stream": true, "allow_concurrent_streaming": false},
            "c",
            {"name": "d", "allow_concurrent_streaming": null}
        ]}}}"#,
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let flags: Vec<bool> = document
        .get("music.x")
        .unwrap()
        .sounds
        .iter()
        .map(|entry| entry.allow_concurrent_streaming)
        .collect();
    assert_eq!(flags, [true, false, true, true]);
    // Another type than a bool is read by the JSON library's bool conversion with the crate's warning, as `stream` is.
    let document = parse(
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": [{"name": "a", "allow_concurrent_streaming": 0}]}}}"#,
    );
    assert!(!document.get("x").unwrap().sounds[0].allow_concurrent_streaming);
    let findings: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|d| {
            format!(
                "{} {:?} {}",
                d.code.name(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    assert_eq!(
        findings,
        ["InvalidValue Info sound_definitions/x/sounds/0/allow_concurrent_streaming"]
    );
}

#[test]
fn the_ten_embedded_functional_documents_run_both_steps_without_a_message() {
    for (name, text) in FUNCTIONAL {
        // `player-3` is a format string: each `{}` (identifier, category,
        // sound) is saved as `FORMAT_PLACEHOLDER`; the tests fill it as these values do.
        let text = if *name == "player-3" {
            text.replacen("FORMAT_PLACEHOLDER", "test_sound_event", 1)
                .replacen("FORMAT_PLACEHOLDER", "neutral", 1)
                .replacen("FORMAT_PLACEHOLDER", "sounds/random/click", 1)
        } else {
            (*text).to_owned()
        };
        let document = parse(&text);
        assert_eq!(
            document.format_version,
            Some(SemVersion::new(1, 26, 50)),
            "{name}"
        );
        // The crate's own category lint (rule 13) is the one finding, for the
        // `ui/music` category of `player-1` (pinned by sd07); the steps
        // say nothing.
        let findings: Vec<String> = document
            .diagnostics()
            .iter()
            .map(|d| format!("{} {}", d.code.name(), d.location.json_path))
            .collect();
        let expected: &[&str] = if *name == "player-1" {
            &["InvalidValue sound_definitions/game_music_event/category"]
        } else {
            &[]
        };
        assert_eq!(findings, expected, "{name}");
    }
}

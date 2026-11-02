//! `sounds/sound_definitions.json`: the rows `sd01`–`sd24`
//! and their rules, with both ends of each rule pinned.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Merged, RawLayer};
use bedrock_addon::families::sound_definitions::{
    SoundDefinition, SoundDefinitions, SoundEntryForm,
};
use bedrock_addon::id::SoundEventName;
use bedrock_addon::version::SemVersion;

use super::common::{
    DEFINITIONS, ZOMBIE, copies, describe, expect_clean, expect_none, expect_one, found, json,
    pack_of,
};

/// The definition `name` of the merged document.
#[track_caller]
fn definition<'a>(
    sounds: &'a Merged<SoundDefinitions, RawLayer>,
    name: &str,
) -> &'a SoundDefinition {
    sounds
        .get(name)
        .unwrap_or_else(|| panic!("{name} in {:#?}", describe(sounds)))
}

/// A 1.20.20 document with the one definition `x`.
fn latest(definition: &str) -> String {
    format!(r#"{{"format_version": "1.20.20", "sound_definitions": {{"x": {definition}}}}}"#)
}

#[test]
fn sd01_legacy_root_runs_every_step() {
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{ "mob.pig.say": { "category": "neutral", "max_distance": 16, "sounds": [ { "name": "a", "pitch": 1.2, "volume": 0.5 } ] } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(
            r#"{ "format_version": "1.26.50", "sound_definitions": { "mob.pig.say": { "category": "neutral", "__use_legacy_max_distance": "true", "max_distance": 0.0, "min_distance": null, "sounds": [ { "name": "a", "volume": 0.5 } ] } } }"#
        )
    );
    let pig = definition(sounds, "mob.pig.say");
    assert!(pig.use_legacy_max_distance);
    assert_eq!(pig.max_distance, 0.0);
    assert_eq!((pig.sounds[0].pitch, pig.sounds[0].volume), (1.0, 0.5));
    assert_eq!(sounds.format_version, Some(SemVersion::new(1, 26, 50)));
    expect_one(sounds, "DroppedField", Severity::Info, "max_distance");
    // The entry's `pitch` the 1.14.0 step removes ("no message") is noted
    // at the member as the file has it.
    let dropped: Vec<String> = sounds
        .diagnostics()
        .filter(|d| d.code.name() == "DroppedField" && d.severity == Severity::Info)
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        dropped,
        ["mob.pig.say/max_distance", "mob.pig.say/sounds/0/pitch"],
        "{:#?}",
        describe(sounds)
    );
    assert_eq!(sounds.diagnostics().count(), 2, "{:#?}", describe(sounds));
}

/// The 1.14.0 step sets every definition's
/// `__use_legacy_max_distance` to `"true"`, as the game's does; a value of the file's under
/// that name that is another one is noted at the member, and `"true"` says nothing.
#[test]
fn a_legacy_distance_flag_of_the_file_the_1_14_step_writes_over_is_noted() {
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{ "a": { "__use_legacy_max_distance": "false", "sounds": [ "s" ] }, "b": { "__use_legacy_max_distance": 7, "sounds": [ "t" ] }, "c": { "__use_legacy_max_distance": "true", "sounds": [ "u" ] }, "d": { "sounds": [ "v" ] } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    let dropped: Vec<String> = sounds
        .diagnostics()
        .filter(|d| d.code.name() == "DroppedField" && d.severity == Severity::Info)
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        dropped,
        ["a/__use_legacy_max_distance", "b/__use_legacy_max_distance"],
        "the flag the step writes over is not told: {:#?}",
        describe(sounds)
    );
    assert_eq!(sounds.diagnostics().count(), 2, "{:#?}", describe(sounds));
    for name in ["a", "b", "c", "d"] {
        assert!(definition(sounds, name).use_legacy_max_distance, "{name}");
    }
}

/// An entry of a `sounds` that is an object is
/// named by its key, as the file has it, not by the index the walk gives it.
#[test]
fn a_finding_of_an_entry_of_a_keyed_sounds_names_the_key() {
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{ "a": { "sounds": { "f": { "name": "s", "volume": "x" }, "b": { "name": "t" } } } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    let paths: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["a/sounds/f/volume"], "{:#?}", describe(sounds));
    // A list is named by its indices, as before.
    let stack = copies(
        DEFINITIONS,
        &[r#"{ "a": { "sounds": [ { "name": "t" }, { "name": "s", "volume": "x" } ] } }"#],
    );
    let sounds = stack.sound_definitions();
    let paths: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["a/sounds/1/volume"], "{:#?}", describe(sounds));
}

#[test]
fn sd02_v1_14_0_zeroes_integer_distances_only() {
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"min_distance": 2, "max_distance": 32.0, "sounds": []}}}"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(
            r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"min_distance": 0.0, "max_distance": 32.0, "sounds": []}}}"#
        )
    );
    let x = definition(sounds, "x");
    assert_eq!((x.min_distance, x.max_distance), (0.0, 32.0));
    assert!(!x.use_legacy_max_distance);
    expect_one(sounds, "DroppedField", Severity::Info, "min_distance");
    assert_eq!(found(sounds, "DroppedField").len(), 1);
}

#[test]
fn sd03_v1_20_20_reads_integer_distances() {
    let text = latest(r#"{"min_distance": 2, "max_distance": 32}"#);
    let stack = copies(DEFINITIONS, &[&text]);
    let sounds = stack.sound_definitions();
    // Only the 1.26.50 step runs: the stamp, and no entry streams.
    assert_eq!(*sounds.json(), json(&text.replace("1.20.20", "1.26.50")));
    let x = definition(sounds, "x");
    assert_eq!((x.min_distance, x.max_distance), (2.0, 32.0));
    expect_clean(sounds);
}

#[test]
fn sd04_a_wrapper_without_a_version_is_wrapped_again() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{ "sound_definitions": { "x": { "sounds": ["a"] } } }"#],
    );
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    let lost = definition(sounds, "sound_definitions");
    assert!(lost.sounds.is_empty() && lost.use_legacy_max_distance);
    assert_eq!(sounds.sound_definitions.len(), 1);
    expect_one(
        sounds,
        "SoundDefinitionsWrapperWithoutVersion",
        Severity::Info,
        "",
    );
}

#[test]
fn sd05_an_unparsable_version_is_the_legacy_path() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{ "format_version": "banana", "sound_definitions": { "x": { "sounds": ["a"] } } }"#],
    );
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    assert!(sounds.get("sound_definitions").is_some());
    expect_one(
        sounds,
        "SoundDefinitionsWrapperWithoutVersion",
        Severity::Info,
        "",
    );
    // The root `format_version` became a definition too (rule 7's warning).
    assert!(sounds.get("format_version").is_some());
}

#[test]
fn a_wrapper_at_1_14_0_is_no_wrapper_trap() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{ "format_version": "1.14.0", "sound_definitions": { "x": { "sounds": ["a"] } } }"#],
    );
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_some());
    expect_none(sounds, "SoundDefinitionsWrapperWithoutVersion");
}

#[test]
fn sd06_a_future_version_is_the_newest_shape_deviation() {
    // vanilla = "no ceiling, no diagnostic"; ours adds the info.
    let stack = copies(
        DEFINITIONS,
        &[
            r#"{"format_version": "9.0.0", "sound_definitions": {"x": {"max_distance": 4, "sounds": ["a"]}}}"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(definition(sounds, "x").max_distance, 4.0);
    expect_one(sounds, "FormatVersionAboveEngine", Severity::Info, "9.0.0");
    assert_eq!(sounds.diagnostics().count(), 1);
}

#[test]
fn the_engine_version_itself_is_not_above_the_engine() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{"format_version": "1.26.60", "sound_definitions": {}}"#],
    );
    expect_clean(stack.sound_definitions());
    let stack = copies(
        DEFINITIONS,
        &[r#"{"format_version": "1.26.61", "sound_definitions": {}}"#],
    );
    expect_one(
        stack.sound_definitions(),
        "FormatVersionAboveEngine",
        Severity::Info,
        "1.26.61",
    );
}

#[test]
fn sd07_vanilla_test_inputs() {
    // The two JSON strings embedded in the game (sd07).
    let stack = copies(
        DEFINITIONS,
        &[r#"{"format_version": "1.14.0", "sound_definitions": {
            "game_music_event": {"category": "ui/music", "sounds": ["a"]},
            "subtitle_sound": {"category": "neutral", "subtitle": "a subtitle", "sounds": ["a"]},
            "no_subtitle_sound": {"category": "neutral", "sounds": ["a"]},
            "special_chars_subtitle_sound": {"category": "neutral", "subtitle": "!@#$%^&*()_+-=[]{};':,./<>?é", "sounds": ["a"]}
        }}"#],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(definition(sounds, "subtitle_sound").subtitle, "a subtitle");
    assert_eq!(definition(sounds, "no_subtitle_sound").subtitle, "");
    assert_eq!(
        definition(sounds, "special_chars_subtitle_sound").subtitle,
        "!@#$%^&*()_+-=[]{};':,./<>?\u{e9}"
    );
    let music = definition(sounds, "game_music_event");
    assert_eq!(music.category, "ui/music");
    // Not exactly `ui`: positional by default.
    assert!(music.sounds[0].is_3d);
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Info,
        "game_music_event/category",
    );
    assert_eq!(found(sounds, "InvalidValue").len(), 1);
}

#[test]
fn every_listed_category_is_accepted_and_an_absent_one_is_no_finding() {
    for category in bedrock_addon::families::sound_definitions::CATEGORIES {
        let text = latest(&format!(r#"{{"category": "{category}"}}"#));
        expect_clean(copies(DEFINITIONS, &[&text]).sound_definitions());
    }
    assert_eq!(
        bedrock_addon::families::sound_definitions::CATEGORIES.len(),
        11
    );
    expect_clean(copies(DEFINITIONS, &[&latest("{}")]).sound_definitions());
    // A category that is no string is read as none and is no rule-13 case.
    let stack = copies(DEFINITIONS, &[&latest(r#"{"category": 5}"#)]);
    assert_eq!(definition(stack.sound_definitions(), "x").category, "");
    expect_clean(stack.sound_definitions());
    let stack = copies(DEFINITIONS, &[&latest(r#"{"category": "Hostile"}"#)]);
    expect_one(
        stack.sound_definitions(),
        "InvalidValue",
        Severity::Info,
        "x/category",
    );
}

#[test]
fn sd08_merge_is_field_wise() {
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{"format_version": "1.20.20", "sound_definitions": { "mob.zombie.say": { "sounds": ["my/zombie"] } } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    let zombie = definition(sounds, "mob.zombie.say");
    assert_eq!(zombie.category, "hostile");
    assert_eq!((zombie.min_distance, zombie.max_distance), (2.0, 16.0));
    assert_eq!(zombie.subtitle, "subtitles.entity.zombie.ambient");
    let names: Vec<&str> = zombie.sounds.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["my/zombie"]);
    expect_clean(sounds);
    // The definition belongs to the top pack, which wrote it last; both copies are layers.
    let origin = sounds
        .origin(&SoundEventName::parse("mob.zombie.say"))
        .unwrap();
    assert_eq!(origin.pack.index(), Some(0));
    assert_eq!(sounds.layers().len(), 2);
    assert_eq!(sounds.layers()[1].json, json(ZOMBIE));
}

#[test]
fn sd09_a_legacy_overlay_resets_the_distances() {
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{ "mob.zombie.say": { "sounds": ["my/zombie"] } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(
            r#"{"format_version": "1.26.50", "sound_definitions": {"mob.zombie.say": {
                "category": "hostile", "max_distance": null, "min_distance": null,
                "subtitle": "subtitles.entity.zombie.ambient", "sounds": ["my/zombie"],
                "__use_legacy_max_distance": "true"}}}"#
        )
    );
    let zombie = definition(sounds, "mob.zombie.say");
    assert_eq!((zombie.min_distance, zombie.max_distance), (0.0, 0.0));
    assert!(zombie.use_legacy_max_distance);
    expect_one(
        sounds,
        "DistanceResetByLegacyOverlay",
        Severity::Info,
        "mob.zombie.say",
    );
    let reset = found(sounds, "DistanceResetByLegacyOverlay")[0];
    assert_eq!(pack_of(reset), Some(0), "the overlay's pack");
    assert_eq!(sounds.diagnostics().count(), 1, "{:#?}", describe(sounds));
}

#[test]
fn a_distance_reset_needs_a_number_below_and_a_null_above() {
    // Nothing numeric below: no reset to report.
    let base = r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"sounds": ["a"]}}}"#;
    let stack = copies(DEFINITIONS, &[base, r#"{ "x": { "sounds": ["b"] } }"#]);
    expect_none(stack.sound_definitions(), "DistanceResetByLegacyOverlay");
    // A 1.20.20 overlay inserts nothing.
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{"format_version": "1.20.20", "sound_definitions": {"mob.zombie.say": {"sounds": ["b"]}}}"#,
        ],
    );
    expect_none(stack.sound_definitions(), "DistanceResetByLegacyOverlay");
    // Only `min_distance` numeric below is enough.
    let base =
        r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"min_distance": 1.0}}}"#;
    let stack = copies(
        DEFINITIONS,
        &[
            base,
            r#"{"format_version": "1.14.0", "sound_definitions": {"x": {}}}"#,
        ],
    );
    expect_one(
        stack.sound_definitions(),
        "DistanceResetByLegacyOverlay",
        Severity::Info,
        "x",
    );
    // An overlay that writes its own float distances resets nothing.
    let stack = copies(
        DEFINITIONS,
        &[
            base,
            r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"min_distance": 3.0, "max_distance": 4.0}}}"#,
        ],
    );
    expect_none(stack.sound_definitions(), "DistanceResetByLegacyOverlay");
}

#[test]
fn sd10_null_overwrites() {
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{"format_version": "1.20.20", "sound_definitions": { "mob.zombie.say": { "subtitle": null } } }"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(definition(sounds, "mob.zombie.say").subtitle, "");
    assert_eq!(definition(sounds, "mob.zombie.say").sounds.len(), 3);
    expect_clean(sounds);
}

#[test]
fn sd11_an_unparsable_base_gives_an_empty_result_deviation() {
    // vanilla = "partial Reader::parse result used"; ours: nothing is loaded.
    let stack = copies(DEFINITIONS, &["{ not json", ZOMBIE]);
    let sounds = stack.sound_definitions();
    assert!(sounds.sound_definitions.is_empty());
    assert!(sounds.layers().is_empty());
    expect_one(
        sounds,
        "MalformedInput",
        Severity::Error,
        "Could not parse file at the top of the stack: sounds/sound_definitions.json.",
    );
    assert_eq!(pack_of(found(sounds, "MalformedInput")[0]), Some(1));
    assert_eq!(sounds.diagnostics().count(), 1);
}

#[test]
fn a_file_nested_past_the_depth_limit_is_no_json_and_overflows_nothing() {
    // The merge and the readers recurse with the document: the depth check bounds them.
    let deep = format!("{}{}", "[".repeat(200_000), "]".repeat(200_000));
    let nested = format!(
        r#"{{"format_version": "1.20.20", "sound_definitions": {{"x": {{"sounds": {deep}}}}}}}"#
    );
    let stack = copies(DEFINITIONS, &[&nested]);
    let sounds = stack.sound_definitions();
    assert!(sounds.sound_definitions.is_empty());
    expect_one(
        sounds,
        "MalformedInput",
        Severity::Error,
        "Could not parse file at the top of the stack",
    );
    // As an overlay it is skipped like any copy that is no JSON.
    let stack = copies(DEFINITIONS, &[ZOMBIE, &nested]);
    assert_eq!(stack.sound_definitions().sound_definitions.len(), 1);
    expect_one(
        stack.sound_definitions(),
        "MalformedInput",
        Severity::Info,
        "",
    );
}

#[test]
fn sd12_an_unparsable_overlay_is_skipped_deviation() {
    // vanilla: silent; ours: a warning naming the pack.
    let stack = copies(DEFINITIONS, &[ZOMBIE, "{ not json"]);
    let sounds = stack.sound_definitions();
    assert_eq!(definition(sounds, "mob.zombie.say").sounds.len(), 3);
    assert_eq!(sounds.layers().len(), 1);
    let skipped = found(sounds, "MalformedInput");
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].severity, Severity::Info);
    assert_eq!(pack_of(skipped[0]), Some(0));
    assert!(!skipped[0].to_string().contains("top of the stack"));
}

#[test]
fn sd13_external_event_and_sounds() {
    let text = latest(
        r#"{"external_event_name": "ev", "external_event_parameters": [{"parameter": "p", "value": 1.5}, {"parameter": "q", "value": 1}], "sounds": ["a"]}"#,
    );
    let stack = copies(DEFINITIONS, &[&text]);
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    assert_eq!(x.external_event_name.as_deref(), Some("ev"));
    assert!(x.sounds.is_empty());
    let parameters: Vec<(&str, f32)> = x
        .external_event_parameters
        .iter()
        .map(|p| (p.parameter.as_str(), p.value))
        .collect();
    // An integer value is not a double: 0.0.
    assert_eq!(parameters, [("p", 1.5), ("q", 0.0)]);
    // rule 8's text is gone from 1.26.60: an information of ours at the definition
    let at_x: Vec<_> = found(sounds, "InvalidValue")
        .into_iter()
        .filter(|d| {
            d.severity == Severity::Info && d.location.json_path.to_string().ends_with("/x")
        })
        .collect();
    assert_eq!(at_x.len(), 1, "{:#?}", describe(sounds));
}

#[test]
fn an_external_event_alone_or_with_null_sounds_is_no_finding() {
    for definition in [
        r#"{"external_event_name": "ev"}"#,
        r#"{"external_event_name": "ev", "sounds": null}"#,
        r#"{"external_event_name": null, "sounds": ["a"]}"#,
    ] {
        expect_clean(copies(DEFINITIONS, &[&latest(definition)]).sound_definitions());
    }
}

#[test]
fn sd14_entry_defaults() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"category": "music", "sounds": ["s", {"name": "a"}]}"#,
        )],
    );
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    let (path, object) = (&x.sounds[0], &x.sounds[1]);
    assert_eq!(path.form, SoundEntryForm::Path);
    assert_eq!(object.form, SoundEntryForm::Object);
    assert!(!path.interruptible && object.interruptible);
    assert!(!path.is_3d && !object.is_3d);
    for entry in [path, object] {
        assert_eq!((entry.weight, entry.volume, entry.pitch), (1, 1.0, 1.0));
        assert!(!entry.stream);
    }
    expect_clean(sounds);
}

#[test]
fn is_3d_defaults_to_false_for_music_and_ui_in_any_case_only() {
    for (category, expected) in [
        ("music", false),
        ("MUSIC", false),
        ("ui", false),
        ("Ui", false),
        ("hostile", true),
        ("ui/music", true),
    ] {
        let text = latest(&format!(r#"{{"category": "{category}", "sounds": ["s"]}}"#));
        let stack = copies(DEFINITIONS, &[&text]);
        assert_eq!(
            definition(stack.sound_definitions(), "x").sounds[0].is_3d,
            expected,
            "{category}"
        );
    }
    // No category: positional.
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"sounds": ["s", {"name": "a", "is3D": false}]}"#,
        )],
    );
    let x = definition(stack.sound_definitions(), "x");
    assert!(x.sounds[0].is_3d && !x.sounds[1].is_3d);
}

#[test]
fn sd15_a_music_name_of_another_category() {
    let text = r#"{"format_version": "1.20.20", "sound_definitions": {"music.game.x": {"category": "ambient", "sounds": ["a"]}}}"#;
    let stack = copies(DEFINITIONS, &[text]);
    expect_one(
        stack.sound_definitions(),
        "InvalidValue",
        Severity::Warning,
        "The sound category of 'music.game.x' is not set as music in sound_definitions.json",
    );
    assert_eq!(stack.sound_definitions().diagnostics().count(), 1);
}

#[test]
fn the_music_name_check_needs_a_dot_and_the_exact_first_segment() {
    let doc = |name: &str, category: &str| {
        format!(
            r#"{{"format_version": "1.20.20", "sound_definitions": {{"{name}": {{"category": "{category}"}}}}}}"#
        )
    };
    // No `.`: never checked. Another first segment, or another case: not `music`.
    for (name, category) in [
        ("music", "ambient"),
        ("musical.x", "ambient"),
        ("Music.x", "ambient"),
        ("record.music", "record"),
        ("music.x", "music"),
        ("music.x", "MUSIC"),
    ] {
        let stack = copies(DEFINITIONS, &[&doc(name, category)]);
        let named: Vec<_> = found(stack.sound_definitions(), "InvalidValue")
            .into_iter()
            .filter(|d| d.to_string().contains("is not set as music"))
            .collect();
        assert!(named.is_empty(), "{name} {category}");
    }
    // `ui` is checked like every other category.
    let stack = copies(DEFINITIONS, &[&doc("music.x", "ui")]);
    expect_one(
        stack.sound_definitions(),
        "InvalidValue",
        Severity::Warning,
        "is not set as music",
    );
}

#[test]
fn sd16_a_definition_that_is_no_object_is_skipped_deviation() {
    // vanilla = "empty SoundEvent registered under the name".
    let old = r#"{"format_version": "1.14.0", "sound_definitions": {"x": "not an object", "y": {"sounds": ["a"]}}}"#;
    let stack = copies(DEFINITIONS, &[old]);
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    assert!(sounds.get("y").is_some());
    let malformed: Vec<_> = found(sounds, "InvalidValue")
        .into_iter()
        .filter(|d| {
            d.to_string()
                .contains("sound definitions has sound 'x'that is malformed")
        })
        .collect();
    // Once from the step (vanilla's, a kept error) and once from the read of the merged
    // document (ours, an information).
    assert_eq!(malformed.len(), 2, "{:#?}", describe(sounds));
    assert!(
        malformed
            .iter()
            .any(|d| d.severity == Severity::Error && d.kept)
    );
    assert!(malformed.iter().any(|d| d.severity == Severity::Info));

    let new = r#"{"format_version": "1.20.20", "sound_definitions": {"x": "not an object", "y": {"sounds": ["a"]}}}"#;
    let stack = copies(DEFINITIONS, &[new]);
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    assert!(sounds.get("y").is_some());
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Info,
        "sound definitions has sound 'x'that is malformed",
    );
}

#[test]
fn sd17_known_ignored_members() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"volume": 1, "pitch": 1, "sounds": [{"name": "a", "load_on_low_memory": true, "pitch:": 1.0}]}"#,
        )],
    );
    let sounds = stack.sound_definitions();
    expect_clean(sounds);
    let x = definition(sounds, "x");
    assert!(x.extra.get("volume").is_some());
    assert!(x.sounds[0].extra.get("pitch:").is_some());
    assert!(x.sounds[0].extra.get("load_on_low_memory").is_some());
    // `allow_concurrent_streaming` is read from 1.26.60 on: the
    // `streaming` tests.
}

#[test]
fn an_unknown_member_is_reported_at_every_level() {
    let text = r#"{"format_version": "1.20.20", "stray": 1, "sound_definitions": {"x": {
        "bogus": 1, "sounds": [{"name": "a", "loud": true}],
        "actions": [{"action_type": 1, "other": 2}]}}}"#;
    let stack = copies(DEFINITIONS, &[text]);
    let sounds = stack.sound_definitions();
    let paths: Vec<String> = found(sounds, "UnrecognizedField")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "stray",
            "sound_definitions/x/bogus",
            "sound_definitions/x/sounds/0/loud",
            "sound_definitions/x/actions/0/other"
        ]
    );
    assert!(
        found(sounds, "UnrecognizedField")
            .iter()
            .all(|d| d.severity == Severity::Info)
    );
    assert_eq!(definition(sounds, "x").extra.get("bogus"), Some(&json("1")));
}

#[test]
fn sd18_a_versioned_copy_without_the_section_wipes_every_definition() {
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{"format_version": "1.16.0", "mine": {"sounds": ["a"]}}"#,
        ],
    );
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(
            r#"{"format_version": "1.26.50", "sound_definitions": null, "mine": {"sounds": ["a"]}}"#
        )
    );
    assert!(sounds.sound_definitions.is_empty());
    // Vanilla's two error lines; the document loads, so they are warnings.
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "sound definition field isn't a list of sound definitions!",
    );
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "sound_definitions section is malformed",
    );
    expect_one(sounds, "SoundDefinitionsCleared", Severity::Info, "");
    assert_eq!(
        pack_of(found(sounds, "SoundDefinitionsCleared")[0]),
        Some(0)
    );
    // The root-level definitions are unread root members.
    expect_one(sounds, "UnrecognizedField", Severity::Info, "mine");
}

#[test]
fn a_copy_without_the_section_clears_what_is_below_and_nothing_when_nothing_is_below() {
    let stack = copies(DEFINITIONS, &[r#"{"format_version": "1.16.0"}"#]);
    let sounds = stack.sound_definitions();
    expect_none(sounds, "SoundDefinitionsCleared");
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "isn't a list of sound definitions!",
    );
    // A lower copy without definitions has nothing to clear either.
    let empty = r#"{"format_version": "1.20.20", "sound_definitions": {}}"#;
    let stack = copies(DEFINITIONS, &[empty, r#"{"format_version": "1.16.0"}"#]);
    expect_none(stack.sound_definitions(), "SoundDefinitionsCleared");
    // At 1.20.20 only the 1.26.50 step runs. It reads the section with the non-const
    // `operator[]`, which inserts it as `null`; that `null` merges over the lower
    // copy as sd18's does. The step logs nothing; the merged root's read says the section is
    // malformed, and the crate tells the clearing.
    let stack = copies(DEFINITIONS, &[ZOMBIE, r#"{"format_version": "1.20.20"}"#]);
    let sounds = stack.sound_definitions();
    assert_eq!(
        *sounds.json(),
        json(r#"{"format_version": "1.26.50", "sound_definitions": null}"#)
    );
    assert!(sounds.sound_definitions.is_empty());
    expect_one(sounds, "SoundDefinitionsCleared", Severity::Info, "");
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "sound_definitions section is malformed",
    );
    assert_eq!(sounds.diagnostics().count(), 2, "{:#?}", describe(sounds));
    // With nothing below, nothing is cleared; the read still finds no section.
    let stack = copies(DEFINITIONS, &[r#"{"format_version": "1.20.20"}"#]);
    let sounds = stack.sound_definitions();
    expect_none(sounds, "SoundDefinitionsCleared");
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "sound_definitions section is malformed",
    );
}

#[test]
fn a_merged_root_that_is_no_object_or_null() {
    let stack = copies(DEFINITIONS, &["[1, 2]"]);
    let sounds = stack.sound_definitions();
    assert!(sounds.sound_definitions.is_empty());
    expect_one(
        sounds,
        "InvalidValue",
        Severity::Error,
        "sound_definitions.json contents is malformed",
    );
    // A `null` root loads nothing, silently.
    let stack = copies(DEFINITIONS, &["null"]);
    expect_clean(stack.sound_definitions());
    assert!(stack.sound_definitions().sound_definitions.is_empty());
}

#[test]
fn sd20_a_legacy_overlay_clears_the_sounds() {
    let stack = copies(
        DEFINITIONS,
        &[ZOMBIE, r#"{ "mob.zombie.say": { "category": "hostile" } }"#],
    );
    let sounds = stack.sound_definitions();
    let merged = &sounds.json();
    let zombie_json = merged
        .as_object()
        .unwrap()
        .get("sound_definitions")
        .unwrap()
        .as_object()
        .unwrap()
        .get("mob.zombie.say")
        .unwrap()
        .as_object()
        .unwrap();
    assert!(zombie_json.get("sounds").unwrap().is_null());
    assert!(zombie_json.get("min_distance").unwrap().is_null());
    assert!(zombie_json.get("max_distance").unwrap().is_null());
    assert_eq!(
        zombie_json.get("__use_legacy_max_distance"),
        Some(&json("\"true\""))
    );
    assert!(definition(sounds, "mob.zombie.say").sounds.is_empty());
    expect_one(
        sounds,
        "SoundsClearedByLegacyOverlay",
        Severity::Info,
        "mob.zombie.say",
    );
    assert_eq!(
        pack_of(found(sounds, "SoundsClearedByLegacyOverlay")[0]),
        Some(0)
    );
}

#[test]
fn a_legacy_overlay_clears_no_sounds_it_writes_or_that_were_not_there() {
    let stack = copies(
        DEFINITIONS,
        &[ZOMBIE, r#"{ "mob.zombie.say": { "sounds": ["a"] } }"#],
    );
    expect_none(stack.sound_definitions(), "SoundsClearedByLegacyOverlay");
    let stack = copies(
        DEFINITIONS,
        &[ZOMBIE, r#"{ "other": { "category": "hostile" } }"#],
    );
    expect_none(stack.sound_definitions(), "SoundsClearedByLegacyOverlay");
    // A 1.14.0 overlay inserts no `sounds`.
    let stack = copies(
        DEFINITIONS,
        &[
            ZOMBIE,
            r#"{"format_version": "1.14.0", "sound_definitions": {"mob.zombie.say": {"category": "hostile"}}}"#,
        ],
    );
    expect_none(stack.sound_definitions(), "SoundsClearedByLegacyOverlay");
    assert_eq!(
        definition(stack.sound_definitions(), "mob.zombie.say")
            .sounds
            .len(),
        3
    );
}

#[test]
fn sd21_a_legacy_root_version_member_becomes_a_definition() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{ "format_version": "1.10.0", "a": { "sounds": ["x"] } }"#],
    );
    let sounds = stack.sound_definitions();
    let merged = sounds.json().as_object().unwrap();
    assert_eq!(
        merged
            .get("sound_definitions")
            .unwrap()
            .as_object()
            .unwrap()
            .get("format_version"),
        Some(&json(
            r#"{ "__use_legacy_max_distance": "true", "sounds": null, "max_distance": null, "min_distance": null }"#
        ))
    );
    assert_eq!(definition(sounds, "a").sounds[0].name, "x");
    assert!(sounds.get("format_version").is_some());
    // At the member as the file has it, at the root: the file has no wrapper.
    expect_one(sounds, "InvalidValue", Severity::Info, "| format_version |");
    assert_eq!(sounds.diagnostics().count(), 1, "{:#?}", describe(sounds));
}

#[test]
fn sd22_an_array_format_version() {
    let stack = copies(
        DEFINITIONS,
        &[r#"{"format_version": [1, 20, 20], "sound_definitions": {"x": {"max_distance": 16}}}"#],
    );
    assert_eq!(
        definition(stack.sound_definitions(), "x").max_distance,
        16.0
    );
    expect_clean(stack.sound_definitions());
    // Two elements are no version: the legacy path.
    let stack = copies(
        DEFINITIONS,
        &[r#"{"format_version": [1, 20], "x": {"max_distance": 16}}"#],
    );
    let x = definition(stack.sound_definitions(), "x");
    assert_eq!(x.max_distance, 0.0);
    assert!(x.use_legacy_max_distance);
}

#[test]
fn sd23_bool_strings() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"__use_legacy_max_distance": "false", "sounds": [{"name": "a", "stream": "false"}]}"#,
        )],
    );
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    assert!(x.use_legacy_max_distance);
    assert!(x.sounds[0].stream);
    let paths: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "sound_definitions/x/__use_legacy_max_distance",
            "sound_definitions/x/sounds/0/stream"
        ]
    );
}

#[test]
fn the_vanilla_spelling_of_the_legacy_flag_and_real_bools_are_no_finding() {
    for definition in [
        r#"{"__use_legacy_max_distance": "true"}"#,
        r#"{"__use_legacy_max_distance": true}"#,
        r#"{"__use_legacy_max_distance": false}"#,
        r#"{"sounds": [{"name": "a", "stream": true, "is3D": false, "interruptible": null}]}"#,
    ] {
        expect_clean(copies(DEFINITIONS, &[&latest(definition)]).sound_definitions());
    }
    // A number for a bool is read by the same rule and reported.
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"sounds": [{"name": "a", "is3D": 0, "interruptible": 2}]}"#,
        )],
    );
    let entry = &definition(stack.sound_definitions(), "x").sounds[0];
    assert!(!entry.is_3d && entry.interruptible);
    assert_eq!(found(stack.sound_definitions(), "InvalidValue").len(), 2);
}

#[test]
fn sd24_int_and_string_coercions() {
    let text = r#"{"format_version": "1.20.20", "sound_definitions": {
        "x": {"sounds": [{"name": "a", "weight": 2.9}, {"name": "b", "weight": "2"}, {"name": 5}]},
        "y": {"external_event_name": 7, "sounds": ["c"]}}}"#;
    let stack = copies(DEFINITIONS, &[text]);
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    let read: Vec<(&str, i32)> = x
        .sounds
        .iter()
        .map(|s| (s.name.as_str(), s.weight))
        .collect();
    assert_eq!(read, [("a", 2), ("b", 0), ("", 1)]);
    let y = definition(sounds, "y");
    assert_eq!(y.external_event_name, None);
    assert_eq!(y.sounds[0].name, "c");
    let rule_8 = |d: &bedrock_addon::diagnostic::Diagnostic| {
        d.location.json_path.to_string() == "sound_definitions/y"
    };
    assert_eq!(
        found(sounds, "InvalidValue")
            .iter()
            .filter(|d| rule_8(d))
            .count(),
        1
    );
    let warned: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .filter(|d| d.severity == Severity::Info && !rule_8(d))
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        warned,
        [
            "sound_definitions/x/sounds/0/weight",
            "sound_definitions/x/sounds/1/weight",
            "sound_definitions/x/sounds/2/name"
        ]
    );
}

#[test]
fn a_float_member_of_another_type_reads_as_zero_with_a_warning() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"min_distance": "4", "max_distance": [4], "sounds": [{"name": "a", "volume": {"v": 1}, "pitch": true}]}"#,
        )],
    );
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    assert_eq!((x.min_distance, x.max_distance), (0.0, 0.0));
    // A bool is 0.0 or 1.0 and, not being one of the three types of rule 29, no finding.
    assert_eq!((x.sounds[0].volume, x.sounds[0].pitch), (0.0, 1.0));
    assert_eq!(
        found(sounds, "InvalidValue").len(),
        3,
        "{:#?}",
        describe(sounds)
    );
}

#[test]
fn entries_of_another_type_are_skipped_and_null_silently() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(r#"{"sounds": [null, 5, true, ["a"], "kept"]}"#)],
    );
    let sounds = stack.sound_definitions();
    let x = definition(sounds, "x");
    assert_eq!(x.sounds.len(), 1);
    assert_eq!(x.sounds[0].name, "kept");
    let paths: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "sound_definitions/x/sounds/1",
            "sound_definitions/x/sounds/2",
            "sound_definitions/x/sounds/3"
        ]
    );
}

#[test]
fn a_sound_path_of_1024_bytes_is_dropped_and_one_of_1023_kept() {
    let long = "a".repeat(1024);
    let stack = copies(
        DEFINITIONS,
        &[&latest(&format!(
            r#"{{"sounds": ["{long}", {{"name": "{long}"}}]}}"#
        ))],
    );
    let sounds = stack.sound_definitions();
    assert!(definition(sounds, "x").sounds.is_empty());
    let errors: Vec<_> = found(sounds, "InvalidValue")
        .into_iter()
        .filter(|d| d.severity == Severity::Info)
        .collect();
    assert_eq!(errors.len(), 2);
    let fits = "a".repeat(1023);
    let stack = copies(
        DEFINITIONS,
        &[&latest(&format!(r#"{{"sounds": ["{fits}"]}}"#))],
    );
    assert_eq!(definition(stack.sound_definitions(), "x").sounds.len(), 1);
    expect_none(stack.sound_definitions(), "InvalidValue");
}

#[test]
fn actions_are_read_with_the_integer_and_double_tests() {
    let stack = copies(
        DEFINITIONS,
        &[&latest(
            r#"{"actions": ["skipped", {"action_type": 3, "name": "n", "target": "t", "enabled": true, "param": 1.5, "delay": 2},
                {"action_type": 3.0}]}"#,
        )],
    );
    let sounds = stack.sound_definitions();
    let actions = &definition(sounds, "x").actions;
    assert_eq!(actions.len(), 2);
    let first = &actions[0];
    assert_eq!(
        (
            first.action_type,
            first.name.as_str(),
            first.target.as_str()
        ),
        (3, "n", "t")
    );
    // `delay` is an integer, not a double: 0.0.
    assert_eq!((first.enabled, first.param, first.delay), (true, 1.5, 0.0));
    // A real `action_type` is not an integer: −1.
    assert_eq!(actions[1].action_type, -1);
    assert!(!actions[1].enabled);
    expect_clean(sounds);
}

#[test]
fn no_file_in_the_stack_is_an_empty_document_without_findings() {
    let stack = copies("textures/readme.txt", &["x"]);
    let sounds = stack.sound_definitions();
    expect_clean(sounds);
    assert!(sounds.sound_definitions.is_empty());
}

/// A file without the wrapper has its definitions at the root, and a finding of the
/// steps or of the reading names the member there; a file with the wrapper, below it.
#[test]
fn a_finding_of_a_file_without_the_wrapper_is_at_the_root() {
    use bedrock_addon::diagnostic::Severity;
    use bedrock_addon::document::Content;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;
    let warnings = |text: &str| -> Vec<String> {
        let document = SoundDefinitions::parse(
            &PackPath::from_static("sounds/sound_definitions.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("the file loads");
        let mut at: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .map(|f| f.location.json_path.to_string())
            .collect();
        at.sort();
        at
    };
    assert_eq!(
        warnings(r#"{"x": {"category": "neutral", "sounds": [7], "zz": 1}, "y": 5}"#),
        ["x/sounds/0", "x/zz", "y"]
    );
    assert_eq!(
        warnings(
            r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"category": "neutral", "sounds": [7], "zz": 1}}}"#
        ),
        ["sound_definitions/x/sounds/0", "sound_definitions/x/zz"]
    );
}

/// In a stack each copy's finding carries the path its own file has: a copy without the
/// wrapper has its definitions at the root, a copy with it below `sound_definitions`, whatever
/// the layout of the merged document and of the other copies.
#[test]
fn a_finding_of_a_stack_is_at_the_path_its_own_copy_has() {
    use bedrock_addon::diagnostic::Severity;
    const WRAPPED: &str = r#"{"format_version": "1.14.0", "sound_definitions": {"w": {"category": "neutral", "sounds": [7], "zz": 1}}}"#;
    const BARE: &str = r#"{"b": {"category": "neutral", "sounds": [7], "zz": 1}, "n": 5}"#;
    // `(pack index, 0 = top; path)` of every warning, sorted.
    let warnings = |texts: &[&str]| -> Vec<(Option<usize>, String)> {
        let stack = copies(DEFINITIONS, texts);
        let mut at: Vec<(Option<usize>, String)> = stack
            .sound_definitions()
            .diagnostics()
            .filter(|f| f.severity == Severity::Info)
            .map(|f| (pack_of(f), f.location.json_path.to_string()))
            .collect();
        at.sort();
        at
    };
    let bare = |pack: usize| ["b/sounds/0", "b/zz", "n"].map(|path| (Some(pack), path.to_owned()));
    let wrapped = |pack: usize| {
        ["sound_definitions/w/sounds/0", "sound_definitions/w/zz"]
            .map(|path| (Some(pack), path.to_owned()))
    };
    // The copy without the wrapper on top (the last text is the top pack).
    let mut expected: Vec<_> = bare(0).into_iter().chain(wrapped(1)).collect();
    expected.sort();
    assert_eq!(warnings(&[WRAPPED, BARE]), expected);
    // And below.
    let mut expected: Vec<_> = wrapped(0).into_iter().chain(bare(1)).collect();
    expected.sort();
    assert_eq!(warnings(&[BARE, WRAPPED]), expected);
}

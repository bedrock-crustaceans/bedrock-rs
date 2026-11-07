//! The chain of `sounds/sound_definitions.json` (contiguity, struct selection, no floor and no
//! ceiling), the three `Upgrade` steps, the history path and write-back (the raw Value level).

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::sound_definitions::SoundDefinitions;
use bedrock_addon::families::sound_definitions::history::{self, SoundDefinitionsHistory};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{FormatVersion, SemVersion};
use bedrock_addon::versioning::testing::{Checks, Run, step_output};

use super::common::json;

fn v(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

/// The member at `path` of `value`.
fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |value, key| value.as_object()?.get(key))
}

/// The 1.0.0 struct's step over `input`.
fn legacy_step(input: &str) -> Value {
    step_output::<history::v1_0_0::SoundDefinitions>(
        &json(input),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs")
    .payload
}

/// The 1.14.0 struct's step over `input`.
fn distances_step(input: &str) -> bedrock_addon::versioning::testing::StepOutput {
    step_output::<history::v1_14_0::SoundDefinitions>(
        &json(input),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs")
}

/// The 1.20.20 struct's step over `input`.
fn streaming_step(input: &str) -> Value {
    step_output::<history::v1_20_20::SoundDefinitions>(
        &json(input),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs")
    .payload
}

#[test]
fn the_1_14_0_step_inserts_a_null_sounds_where_a_definition_has_none() {
    let out = legacy_step(r#"{"x": {"category": "neutral"}, "y": {"sounds": ["a"]}}"#);
    let inserted = at(&out, &["sound_definitions", "x", "sounds"]).expect("a sounds member");
    assert!(inserted.is_null(), "sounds is not null");
    assert_eq!(
        at(&out, &["sound_definitions", "y", "sounds"]),
        Some(&json(r#"["a"]"#)),
        "an existing sounds member changed"
    );
}

#[test]
fn the_1_14_0_step_drops_the_pitch_of_every_object_entry_and_of_nothing_else() {
    let out = legacy_step(
        r#"{"x": {"pitch": 2, "sounds": [{"name": "a", "pitch": 1.2, "volume": 0.5}, "s"]}}"#,
    );
    assert_eq!(
        at(&out, &["sound_definitions", "x", "sounds"]),
        Some(&json(r#"[{"name": "a", "volume": 0.5}, "s"]"#)),
        "the entry kept its pitch, or lost something else"
    );
    assert!(
        at(&out, &["sound_definitions", "x", "pitch"]).is_some(),
        "a definition-level pitch was dropped"
    );
}

#[test]
fn the_1_14_0_step_replaces_a_definition_that_is_no_object_with_the_filled_empty_object() {
    let out = legacy_step(r#"{"format_version": "1.10.0", "n": 5, "l": [1]}"#);
    let expected = json(r#"{"__use_legacy_max_distance": "true", "sounds": null}"#);
    for name in ["format_version", "n", "l"] {
        assert_eq!(
            at(&out, &["sound_definitions", name]),
            Some(&expected),
            "{name}"
        );
    }
}

#[test]
fn the_1_14_0_step_leaves_a_root_that_is_no_object_alone() {
    for root in ["[1, 2]", "null", "\"x\""] {
        assert_eq!(legacy_step(root), json(root), "{root}");
    }
}

#[test]
fn the_1_20_20_step_keeps_a_malformed_definition_as_written_and_names_it() {
    let out = distances_step(
        r#"{"format_version": "1.14.0", "sound_definitions": {"bad": "text", "ok": {}}}"#,
    );
    let logged: Vec<String> = out
        .diagnostics
        .iter()
        .map(ToString::to_string)
        .filter(|line| line.contains("that is malformed"))
        .collect();
    // Once from the step, once from the read of the newest struct.
    assert!(
        !logged.is_empty(),
        "no finding names the malformed definition"
    );
    assert!(
        logged
            .iter()
            .all(|line| line.contains("sound definitions has sound 'bad'that is malformed")),
        "{logged:#?}"
    );
    assert_eq!(
        at(&out.payload, &["sound_definitions", "bad"]),
        Some(&json("\"text\"")),
        "the malformed definition changed"
    );
}

#[test]
fn the_1_26_50_step_makes_a_streamed_object_entry_not_stream_concurrently_and_touches_no_other() {
    let out = streaming_step(
        r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"sounds": [
            {"name": "a", "stream": false}, {"name": "b"}, "c",
            {"name": "d", "stream": true, "allow_concurrent_streaming": true}
        ]}, "y": {"sounds": {"k": {"name": "e", "stream": true}}}}}"#,
    );
    assert_eq!(
        at(&out, &["sound_definitions", "x", "sounds"]),
        Some(&json(
            r#"[{"name": "a", "stream": false}, {"name": "b"}, "c",
            {"name": "d", "stream": true, "allow_concurrent_streaming": true}]"#
        )),
        "an entry that does not stream, or says so already, changed"
    );
    // The step's output is the 1.26.50 struct's typed value, which keeps an object `sounds`
    // as written: the entry is as it was, no `allow_concurrent_streaming`.
    assert_eq!(
        at(&out, &["sound_definitions", "y", "sounds"]),
        Some(&json(r#"{"k": {"name": "e", "stream": true}}"#)),
        "a `sounds` that is no list was walked"
    );
}

#[test]
fn the_chain_is_contiguous_without_floor_or_ceiling() {
    assert_eq!(
        history::VERSIONS,
        [v(1, 0, 0), v(1, 14, 0), v(1, 20, 20), v(1, 26, 50)]
    );
    assert!(history::VERSIONS.windows(2).all(|pair| pair[0] < pair[1]));
    let select = |major, minor, patch| {
        SoundDefinitionsHistory::select(&SemVersion::new(major, minor, patch))
    };
    // No floor: everything below 1.14.0 is the versionless struct.
    assert_eq!(select(0, 0, 0), v(1, 0, 0));
    assert_eq!(select(0, 9, 0), v(1, 0, 0));
    assert_eq!(select(1, 13, 0), v(1, 0, 0));
    assert_eq!(select(1, 13, 999), v(1, 0, 0));
    assert_eq!(select(1, 14, 0), v(1, 14, 0));
    assert_eq!(select(1, 20, 19), v(1, 14, 0));
    assert_eq!(select(1, 20, 20), v(1, 20, 20));
    assert_eq!(select(1, 26, 49), v(1, 20, 20));
    assert_eq!(select(1, 26, 50), v(1, 26, 50));
    // No ceiling.
    assert_eq!(select(1, 26, 60), v(1, 26, 50));
    assert_eq!(select(9, 0, 0), v(1, 26, 50));
    // A prerelease of a step's version orders before it.
    assert_eq!(
        SoundDefinitionsHistory::select(&SemVersion::parse("1.20.20-beta.1").unwrap()),
        v(1, 14, 0)
    );
    assert_eq!(
        SoundDefinitionsHistory::select(&SemVersion::parse("1.26.50-beta.1").unwrap()),
        v(1, 20, 20)
    );
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

fn as_written(text: &str) -> bedrock_addon::document::Document<SoundDefinitionsHistory> {
    SoundDefinitions::parse_as_written(
        &path("sounds/sound_definitions.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("the file reads")
}

const LEGACY: &str = r#"{"mob.pig.say": {"category": "neutral", "max_distance": 16, "sounds": [{"name": "a", "pitch": 1.2}]}}"#;
const ENVELOPE: &str = r#"{"format_version": "1.14.0", "sound_definitions": {"x": {"min_distance": 2, "sounds": ["a"]}}}"#;
const DISTANCES: &str = r#"{"format_version": "1.20.20", "sound_definitions": {"x": {"min_distance": 2, "sounds": [{"name": "a", "stream": true}]}}}"#;
const LATEST: &str = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"min_distance": 2, "max_distance": null, "sounds": ["a", {"name": "b", "weight": 3}]}}}"#;
/// [`LATEST`] as its typed value writes it: each distance as its float, the `null` as the
/// 0.0 it reads as.
const LATEST_WRITTEN: &str = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"min_distance": 2.0, "max_distance": 0.0, "sounds": ["a", {"name": "b", "weight": 3}]}}}"#;

#[test]
fn a_file_is_read_by_the_struct_of_its_version_and_upgrades_to_what_parse_gives() {
    for (text, struct_version, written, upgraded) in [
        (LEGACY, v(1, 0, 0), None, true),
        (ENVELOPE, v(1, 14, 0), Some(v(1, 14, 0)), true),
        (DISTANCES, v(1, 20, 20), Some(v(1, 20, 20)), true),
        (LATEST, v(1, 26, 50), Some(v(1, 26, 50)), false),
    ] {
        let any = as_written(text);
        assert_eq!(any.version().parsed_as, struct_version);
        assert_eq!(any.value().parsed_version(), struct_version);
        assert_eq!(any.version().written, written);
        // An older struct writes its own version's document back; the newest its typed value.
        let rewritten: Value = facet_json::from_str(&any.to_json().unwrap()).unwrap();
        let expected = if text == LATEST { LATEST_WRITTEN } else { text };
        assert_eq!(rewritten, json(expected));
        let parsed = SoundDefinitions::parse(
            &path("sounds/sound_definitions.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("the file loads");
        assert_eq!(parsed.version().upgraded, upgraded);
        assert_eq!(parsed.version().parsed_as, struct_version);
        let mut found = Diagnostics::default();
        let stepped = any
            .into_value()
            .upgrade(&LoadContext::detached(), &mut found);
        assert_eq!(&stepped, parsed.value());
        assert_eq!(stepped.format_version, Some(SemVersion::new(1, 26, 50)));
    }
    // A missing version is 1.13.0 for the gates.
    assert_eq!(as_written(LEGACY).version().effective, v(1, 13, 0));
}

#[test]
fn a_single_file_reports_what_a_stack_reports_for_its_lowest_copy() {
    let parsed = SoundDefinitions::parse(
        &path("sounds/sound_definitions.json"),
        LEGACY.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    let codes: Vec<&str> = parsed.diagnostics().iter().map(|d| d.code.id()).collect();
    // The zeroed distance, and the entry `pitch` the 1.14.0 step removes.
    assert_eq!(codes, ["core/dropped_field", "core/dropped_field"]);
    assert_eq!(
        parsed
            .diagnostics()
            .iter()
            .nth(1)
            .map(|d| (d.severity, d.location.json_path.to_string())),
        Some((
            bedrock_addon::diagnostic::Severity::Info,
            "mob.pig.say/sounds/0/pitch".to_owned()
        ))
    );
    let rejected = SoundDefinitions::parse(
        &path("sounds/sound_definitions.json"),
        b"{ not json",
        &LoadContext::detached(),
    )
    .unwrap_err();
    assert!(rejected.diagnostics.has_errors());
    assert_eq!(
        rejected.diagnostics.iter().next().unwrap().code.id(),
        "core/syntax"
    );
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
fn sound_definitions_write_the_typed_value_of_the_upgraded_document() {
    let latest: SoundDefinitions = round_trips("sounds/sound_definitions.json", LATEST);
    // `json` is the JSON that was read: integer `2`, the `null`, the unknown-free entries.
    assert_eq!(*latest.json(), json(LATEST));
    // What is written is the typed value: the distances as floats, the `null` as its reading.
    let rewritten: Value = facet_json::from_str(&latest.to_json().unwrap()).unwrap();
    assert_eq!(rewritten, json(LATEST_WRITTEN));
    let text = latest.to_json().unwrap();
    assert!(text.contains("\"min_distance\": 2.0,"), "{text}");
    assert!(text.contains("\"max_distance\": 0.0,"), "{text}");
    // An older file is written as the 1.26.50 document it upgrades to.
    let legacy: SoundDefinitions = round_trips("sounds/sound_definitions.json", LEGACY);
    assert_eq!(
        *legacy.json(),
        json(
            r#"{"format_version": "1.26.50", "sound_definitions": {"mob.pig.say": {"category": "neutral", "max_distance": 0.0, "sounds": [{"name": "a"}], "__use_legacy_max_distance": "true", "min_distance": null}}}"#
        )
    );
    // ... and what it writes is that document's typed value: the `null` distance and the
    // string flag as their readings.
    let rewritten: Value = facet_json::from_str(&legacy.to_json().unwrap()).unwrap();
    assert_eq!(
        rewritten,
        json(
            r#"{"format_version": "1.26.50", "sound_definitions": {"mob.pig.say": {"category": "neutral", "min_distance": 0.0, "max_distance": 0.0, "__use_legacy_max_distance": true, "sounds": [{"name": "a"}]}}}"#
        )
    );
    assert_eq!(
        legacy.default_path().as_str(),
        "sounds/sound_definitions.json"
    );
    // `from_json` reads a newest-shape document as it is.
    assert_eq!(SoundDefinitions::from_json(json(LATEST)), latest);
}

#[test]
fn a_member_named_extra_or_json_is_an_unknown_member_like_any_other() {
    let text = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"extra": 1, "json": 2, "sounds": [{"name": "a", "extra": {"k": 1}}]}}}"#;
    let parsed: SoundDefinitions = round_trips("sounds/sound_definitions.json", text);
    let x = parsed.get("x").unwrap();
    assert_eq!(x.extra.get("extra"), Some(&json("1")));
    assert_eq!(x.extra.get("json"), Some(&json("2")));
    assert_eq!(x.sounds[0].extra.get("extra"), Some(&json(r#"{"k": 1}"#)));
    assert_eq!(*parsed.json(), json(text));
}

#[test]
fn a_single_newest_definitions_file_keeps_the_json_it_read() {
    let cx = LoadContext::detached();
    // A 1.26.50 file runs no step: nothing is inserted (no `null` distance, no stamp).
    let newest = r#"{"format_version": [1, 26, 50], "sound_definitions": {"x": {"sounds": [{"name": "a", "stream": true}]}, "y": 5}}"#;
    let document = SoundDefinitions::parse(
        &path("sounds/sound_definitions.json"),
        newest.as_bytes(),
        &cx,
    )
    .unwrap();
    assert_eq!(*document.json(), json(newest));
}

/// The struct of a version is a public door (`VersionStruct::parse`); a text that is
/// no JSON gives no struct, and says why (it gave `None` and no finding).
#[test]
fn a_struct_given_a_text_that_is_no_json_says_so() {
    use bedrock_addon::diagnostic::{Diagnostics, JsonPath, Severity};
    use bedrock_addon::families::sound_definitions::history::{v1_0_0, v1_14_0, v1_20_20};
    use bedrock_addon::json::JsonSlice;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::version::{FormatVersion, FormatVersionInput};
    use bedrock_addon::versioning::{ParseCx, VersionStruct};
    let load = LoadContext::detached();
    let cx = ParseCx {
        load: &load,
        written: FormatVersionInput {
            version: Some(FormatVersion::new(1, 20, 20)),
            beta_literal: false,
            prerelease: false,
        },
        original_version: FormatVersion::new(1, 20, 20),
        at: JsonPath::new(),
    };
    let told = |d: &Diagnostics| -> Vec<(bool, Severity, String)> {
        d.iter()
            .map(|f| {
                (
                    f.is(&bedrock_addon::diagnostic::codes::SYNTAX),
                    f.severity,
                    f.location.json_path.to_string(),
                )
            })
            .collect()
    };
    let expected = [(true, Severity::Error, String::new())];
    let text = JsonSlice::new(r#"{"x": "#);
    let mut d = Diagnostics::default();
    assert!(<v1_0_0::SoundDefinitions as VersionStruct>::parse(text, &cx, &mut d).is_none());
    assert_eq!(told(&d), expected, "1.0.0");
    let mut d = Diagnostics::default();
    assert!(<v1_14_0::SoundDefinitions as VersionStruct>::parse(text, &cx, &mut d).is_none());
    assert_eq!(told(&d), expected, "1.14.0");
    let mut d = Diagnostics::default();
    assert!(<v1_20_20::SoundDefinitions as VersionStruct>::parse(text, &cx, &mut d).is_none());
    assert_eq!(told(&d), expected, "1.20.20");
    let mut d = Diagnostics::default();
    assert!(<SoundDefinitions as VersionStruct>::parse(text, &cx, &mut d).is_none());
    assert_eq!(told(&d), expected, "1.26.50");
    // A text that is JSON gives the struct and no such finding.
    let mut d = Diagnostics::default();
    let read = <v1_0_0::SoundDefinitions as VersionStruct>::parse(
        JsonSlice::new(r#"{"x": {"sounds": []}}"#),
        &cx,
        &mut d,
    );
    assert!(read.is_some() && d.is_empty(), "{d:?}");
}

#[test]
fn a_float_beyond_f32_is_written_as_the_largest_double() {
    // Both read as infinity through the f32 the client reads; the write gives the largest
    // double, which the client's float cast makes infinity again.
    let text = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"min_distance": 1e39, "max_distance": 1e999, "sounds": [{"name": "a", "volume": -1e39}]}}}"#;
    let parsed: SoundDefinitions = round_trips("sounds/sound_definitions.json", text);
    let x = parsed.get("x").unwrap();
    assert_eq!(x.min_distance, f32::INFINITY);
    assert_eq!(x.max_distance, f32::INFINITY);
    assert_eq!(x.sounds[0].volume, f32::NEG_INFINITY);
    let written = parsed.to_json().unwrap();
    assert!(
        written.contains("\"max_distance\": 1.7976931348623157e308"),
        "{written}"
    );
    assert!(
        written.contains("\"volume\": -1.7976931348623157e308"),
        "{written}"
    );
}

#[test]
fn a_float_no_json_holds_is_refused_at_its_member() {
    use bedrock_addon::diagnostic::JsonPath;
    use bedrock_addon::json::EncodeError;
    let mut parsed = SoundDefinitions::from_json(json(
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": [{"name": "a", "pitch": 2}]}}}"#,
    ));
    parsed.sound_definitions[0].sounds[0].pitch = f32::NAN;
    let error = parsed.to_json().unwrap_err();
    let mut expected = JsonPath::of_keys(["sound_definitions", "x", "sounds"]);
    expected.push_index(0);
    expected.push("pitch");
    assert!(
        matches!(&error, EncodeError::NotRepresentable { at, .. } if *at == expected),
        "{error:?}"
    );
}

/// The value read from `text`, written: the text it writes, read back as JSON.
fn written_json(text: &str) -> Value {
    let cx = LoadContext::detached();
    let document =
        SoundDefinitions::parse(&path("sounds/sound_definitions.json"), text.as_bytes(), &cx)
            .expect("the file loads");
    json(&document.to_json().expect("written"))
}

#[test]
fn an_object_form_list_is_written_as_written() {
    // The merge recurses into an object and replaces an array whole: an object written as
    // an array would replace a lower pack's entries it merged into before.
    let text = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": {"b": "y", "a": {"name": "x", "volume": 0.5}}}, "y": {"actions": {"k": {"name": "n"}}}, "z": {"external_event_name": "e", "external_event_parameters": {"p": {"parameter": "q"}}}}}"#;
    let parsed: SoundDefinitions = round_trips("sounds/sound_definitions.json", text);
    let x = parsed.get("x").unwrap();
    assert_eq!(x.sounds.len(), 2);
    assert_eq!(x.sounds[0].name, "x");
    assert_eq!(parsed.get("y").unwrap().actions[0].name, "n");
    assert_eq!(
        parsed.get("z").unwrap().external_event_parameters[0].parameter,
        "q"
    );
    assert_eq!(written_json(text), json(text));
}

#[test]
fn a_root_that_is_no_object_is_written_as_written() {
    for text in [
        "[]",
        "[{\"sound_definitions\": {}}]",
        "5",
        "\"sounds\"",
        "true",
    ] {
        assert_eq!(written_json(text), json(text), "{text}");
    }
}

/// What a copy writes merges over a lower pack's copy as the copy it was read from: the client
/// merges the copies as JSON, so a member written that was absent, or an object written as an
/// array, would change the stack.
#[test]
fn a_written_copy_merges_over_a_lower_pack_as_the_copy_it_was_read_from() {
    use super::common::{DEFINITIONS, copies};
    const LOWER: &str = r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"category": "neutral", "min_distance": 2.0, "max_distance": 16.0, "sounds": {"a": {"name": "a", "volume": 0.5}}}, "y": {"category": "ui", "sounds": ["b"]}}}"#;
    for upper in [
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"max_distance": 8.0}}}"#,
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": {"b": "c"}}}}"#,
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"sounds": ["c", {"name": "d", "stream": true}]}}}"#,
        r#"{"format_version": "1.26.50", "sound_definitions": {"y": {"min_distance": 0, "subtitle": "", "sounds": []}}}"#,
        r#"{"format_version": "1.26.50", "sound_definitions": {"x": {"odd": 1, "min_distance": 1e39}, "z": 5}, "top": true}"#,
        r#"{"sound_definitions": {"x": {"sounds": [{"name": "e", "pitch": 2}]}}}"#,
        "[]",
        "{}",
    ] {
        let cx = LoadContext::detached();
        let written = SoundDefinitions::parse(&path(DEFINITIONS), upper.as_bytes(), &cx)
            .expect("the copy loads")
            .to_json()
            .expect("written");
        let read = copies(DEFINITIONS, &[LOWER, upper]);
        let rewritten = copies(DEFINITIONS, &[LOWER, &written]);
        assert_eq!(
            read.sound_definitions().value(),
            rewritten.sound_definitions().value(),
            "{upper} written as {written}"
        );
    }
}

//! The file: every rule of one document, no stack.

use bedrock_addon_core::diagnostic::Severity;

use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_server_sound_definitions::{DurationMode, SoundDuration, TagValueList};

use super::common::*;

const FULL: &str = r#"{
    "sound_event_name": "demo:boss_theme",
    "duration_info": { "mode": "real_time", "seconds": 184.5 },
    "music_info": { "genres": ["orchestral"], "moods": ["tense"], "artist": "demo.artist", "title": "Boss Theme" },
    "tags": { "demo:area": "arena", "demo:phase": ["one", "two"] },
    "broadcast_range": 64.0
}"#;

#[test]
fn ssd02_full_definition() {
    let document = loads(&file(FULL));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        lines(document.diagnostics())
    );
    let theme = &document.0[0];
    assert_eq!(theme.id(), SoundEventName::parse("demo:boss_theme"));
    assert_eq!(theme.duration(), Some(SoundDuration::Millis(184_500)));
    assert_eq!(
        theme.duration_info.as_ref().unwrap().mode,
        DurationMode::RealTime
    );
    let tags = theme.tags.as_ref().unwrap();
    assert_eq!(tags["demo:area"], TagValueList::One("arena".to_owned()));
    assert_eq!(tags["demo:area"].as_slice(), ["arena"]);
    assert_eq!(tags["demo:phase"].as_slice(), ["one", "two"]);
    let music = theme.music_info.as_ref().unwrap();
    assert_eq!(music.title.as_ref().unwrap(), "Boss Theme");
    assert_eq!(music.genres.as_ref().unwrap()[0], "orchestral");
    assert_eq!(theme.broadcast_range.unwrap(), 64.0);
}

#[test]
fn the_document_writes_back_what_it_read() {
    let document = loads(&file(&format!("{CLICK}, {FULL}")));
    let written = document.to_json().unwrap();
    let again = loads(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().unwrap(), written);
    // Nothing the author did not write is written.
    let minimal = loads(&file(CLICK)).to_json().unwrap();
    let json: bedrock_addon_core::json::Value = facet_json::from_str(&minimal).unwrap();
    let expected: bedrock_addon_core::json::Value = facet_json::from_str(&file(CLICK)).unwrap();
    assert_eq!(json, expected);
}

fn duration(mode: &str, seconds: &str) -> SoundDuration {
    let text = file(&format!(
        r#"{{"sound_event_name": "demo:x", "duration_info": {{"mode": "{mode}", "seconds": {seconds}}}}}"#
    ));
    loads(&text).0[0].duration().unwrap()
}

#[test]
fn ssd03_duration_conversion_truncates() {
    assert_eq!(duration("game_time", "2.5"), SoundDuration::Ticks(50));
    assert_eq!(duration("game_time", "0.07"), SoundDuration::Ticks(1));
    assert_eq!(duration("real_time", "1.2345"), SoundDuration::Millis(1234));
}

#[test]
fn ssd24_duration_overflow_is_the_minimum_not_saturation() {
    assert_eq!(
        duration("game_time", "1.0e9"),
        SoundDuration::Ticks(i32::MIN)
    );
    assert_eq!(
        duration("real_time", "1.0e17"),
        SoundDuration::Millis(i64::MIN)
    );
    // `0.1f32 * 20.0f32` is exactly 2.0.
    assert_eq!(duration("game_time", "0.1"), SoundDuration::Ticks(2));
    // A product of exactly 2^31 is already out of range (107374184 × 20 rounds to it in f32).
    assert_eq!(
        duration("game_time", "107374184"),
        SoundDuration::Ticks(i32::MIN)
    );
    // The largest products that still fit.
    assert_eq!(
        duration("game_time", "100000000"),
        SoundDuration::Ticks(2_000_000_000)
    );
    assert_eq!(
        duration("game_time", "-1.0e9"),
        SoundDuration::Ticks(i32::MIN)
    );
    assert_eq!(
        duration("real_time", "-1.0e17"),
        SoundDuration::Millis(i64::MIN)
    );
    // No `seconds` lint for a positive, finite value.
    let text = file(
        r#"{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 1.0e9}}"#,
    );
    assert!(loads(&text).diagnostics().is_empty());
}

#[test]
fn ssd04_empty_array() {
    let document =
        loads(r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": []}"#);
    assert!(document.0.is_empty() && document.diagnostics().is_empty());
}

#[test]
fn ssd05_a_version_other_than_1_26_60_has_no_parser() {
    for (version, shown) in [
        ("\"1.26.40\"", "1.26.40"),
        ("\"1.26.50\"", "1.26.50"),
        ("[1, 26, 50]", "1.26.50"),
        ("\"1.26.61\"", "1.26.61"),
        ("\"9999.9999.9999\"", "9999.9999.9999"),
    ] {
        let text = format!(
            r#"{{"format_version": {version}, "minecraft:server_sound_definitions": [{CLICK}]}}"#
        );
        one_error(
            &rejected(&text),
            "core/no_parser_for_version",
            &format!("unsupported version: no parser available for version {shown}"),
        );
    }
}

#[test]
fn a_prerelease_version_is_an_ordinary_unsupported_version() {
    let text = format!(
        r#"{{"format_version": "1.26.40-beta.1", "minecraft:server_sound_definitions": [{CLICK}]}}"#
    );
    let diagnostics = rejected(&text);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.id() == "core/no_parser_for_version" && d.severity == Severity::Error),
        "{:#?}",
        lines(&diagnostics)
    );
}

#[test]
fn a_beta_document_has_no_parser_whatever_the_permission() {
    // The game registers the 1.26.60
    // slice and closes: no Beta slice, so selection finds no parser for the sentinel before
    // any permission is asked. The sentinel written out is the
    // same literal.
    for version in ["beta", "9999.9999.9999-beta"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:server_sound_definitions": [{CLICK}]}}"#
        );
        for cx in [
            context(true, false),
            context(false, true),
            context(false, false),
        ] {
            let refused = parse_with(&text, &cx).unwrap_err();
            one_error(
                &refused.diagnostics,
                "core/no_parser_for_version",
                "unsupported version: no parser available for version beta",
            );
        }
    }
}

#[test]
fn ssd06_a_missing_version_is_an_error_deviation() {
    // vanilla = "error: no valid version passed by the caller or found in the input"; ours does
    // not fall back to a floor, there is none.
    let text = format!(r#"{{"minecraft:server_sound_definitions": [{CLICK}]}}"#);
    one_error(
        &rejected(&text),
        "core/no_valid_version",
        "no valid version passed by the caller or found in the input",
    );
}

#[test]
fn ssd07_a_1_26_60_document_needs_no_toggle() {
    // The 1.26.45 beta gate is gone: `parseDefinitionDocument` builds a
    // `BedrockLoadContext` with no permission policy, and the slice is stable.
    let text = file(CLICK);
    for (beta_apis, base_game) in [(false, false), (false, true), (true, false)] {
        let document = parse_with(&text, &context(beta_apis, base_game))
            .unwrap_or_else(|rejected| panic!("{:#?}", lines(&rejected.diagnostics)));
        assert!(document.diagnostics().is_empty());
    }
}

#[test]
fn a_member_written_as_null_rejects_the_file() {
    // The loader's input is a MinecraftDocumentInput, so
    // a `null` member is present and fails its type (0x8), the `std::optional` ones too.
    for (definition, at) in [
        (
            r#"{"sound_event_name": "demo:x", "duration_info": null}"#,
            "minecraft:server_sound_definitions/0/duration_info",
        ),
        (
            r#"{"sound_event_name": "demo:x", "music_info": null}"#,
            "minecraft:server_sound_definitions/0/music_info",
        ),
        (
            r#"{"sound_event_name": "demo:x", "tags": null}"#,
            "minecraft:server_sound_definitions/0/tags",
        ),
        (
            r#"{"sound_event_name": "demo:x", "broadcast_range": null}"#,
            "minecraft:server_sound_definitions/0/broadcast_range",
        ),
        (
            r#"{"sound_event_name": "demo:x", "music_info": {"artist": null}}"#,
            "minecraft:server_sound_definitions/0/music_info/artist",
        ),
        (
            r#"{"sound_event_name": "demo:x", "music_info": {"genres": null}}"#,
            "minecraft:server_sound_definitions/0/music_info/genres",
        ),
    ] {
        let diagnostics = rejected(&file(definition));
        one_error(&diagnostics, "core/invalid_type", "");
        assert_eq!(
            diagnostics
                .iter()
                .next()
                .unwrap()
                .location
                .json_path
                .to_string(),
            at,
            "{definition}"
        );
    }
    // A required member written as `null` was refused already.
    for definition in [
        r#"{"sound_event_name": null}"#,
        r#"{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": null}}"#,
    ] {
        let diagnostics = rejected(&file(definition));
        assert!(diagnostics.has_errors(), "{definition}");
    }
}

#[test]
fn ssd08_a_third_member() {
    let text = format!(
        r#"{{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{CLICK}], "sound_definitions": {{}}}}"#
    );
    one_error(
        &rejected(&text),
        "core/document_member_count",
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
    );
    // Below the slice's version the envelope is not looked at.
    let text = format!(
        r#"{{"format_version": "1.26.40", "minecraft:server_sound_definitions": [{CLICK}], "sound_definitions": {{}}}}"#
    );
    one_error(&rejected(&text), "core/no_parser_for_version", "1.26.40");
}

#[test]
fn ssd09_an_unknown_member_rejects_the_file_and_is_reported() {
    // The strict mask is 0xfffc; vanilla writes the load failure to the content log, area
    // Sound.
    let text = file(&format!(
        r#"{CLICK}, {{"sound_event_name": "demo:two", "volume": 1.0}}"#
    ));
    let diagnostics = rejected(&text);
    one_error(&diagnostics, "core/unrecognized_field", "");
    let at = diagnostics
        .iter()
        .next()
        .unwrap()
        .location
        .json_path
        .to_string();
    assert_eq!(at, "minecraft:server_sound_definitions/1/volume");
    // At every level.
    for nested in [
        r#"{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 1, "extra": 1}}"#,
        r#"{"sound_event_name": "demo:x", "music_info": {"album": "a"}}"#,
    ] {
        one_error(&rejected(&file(nested)), "core/unrecognized_field", "");
    }
}

#[test]
fn ssd10_broadcast_range_needs_a_1_26_50_engine_deviation() {
    // Ours: the member is accepted from engine 1.26.50; retail 1.26.45 does not bind it.
    let text = file(r#"{"sound_event_name": "demo:x", "broadcast_range": 32}"#);
    assert_eq!(loads(&text).0[0].broadcast_range.unwrap(), 32.0);
    let mut old = context(true, false);
    old.engine_version = FormatVersion::new(1, 26, 45);
    let refused = parse_with(&text, &old).unwrap_err();
    one_error(
        &refused.diagnostics,
        "core/unrecognized_field",
        "broadcast_range",
    );
    // The engine version itself, and a file without the member, are fine.
    old.engine_version = FormatVersion::new(1, 26, 50);
    assert!(parse_with(&text, &old).is_ok());
    old.engine_version = FormatVersion::new(1, 26, 45);
    assert!(parse_with(&file(CLICK), &old).is_ok());
}

#[test]
fn ssd11_a_bad_mode() {
    let text = file(
        r#"{"sound_event_name": "demo:x", "duration_info": { "mode": "wall_time", "seconds": 3 }}"#,
    );
    one_error(&rejected(&text), "core/invalid_enum_value", "");
}

#[test]
fn ssd12_missing_required_members() {
    for text in [
        r#"{"sound_event_name": "demo:x", "duration_info": { "mode": "game_time" }}"#,
        r#"{"sound_event_name": "demo:x", "duration_info": { "seconds": 1 }}"#,
        r#"{"duration_info": { "mode": "game_time", "seconds": 1 }}"#,
    ] {
        one_error(&rejected(&file(text)), "core/missing_required_field", "");
    }
}

#[test]
fn the_payload_must_be_an_array_of_objects() {
    for payload in [r#"{"a": 1}"#, "5", r#"["demo:x"]"#] {
        let text = format!(
            r#"{{"format_version": "1.26.60", "minecraft:server_sound_definitions": {payload}}}"#
        );
        one_error(&rejected(&text), "core/invalid_type", "");
    }
    for definition in [
        r#"{"sound_event_name": 5}"#,
        r#"{"sound_event_name": "demo:x", "broadcast_range": "far"}"#,
        r#"{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": "3"}}"#,
        r#"{"sound_event_name": "demo:x", "music_info": {"genres": "rock"}}"#,
    ] {
        one_error(&rejected(&file(definition)), "core/invalid_type", "");
    }
}

#[test]
fn ssd13_a_name_with_whitespace_or_empty() {
    let diagnostics = rejected(&file(r#"{"sound_event_name": "demo:boss theme"}"#));
    one_error(
        &diagnostics,
        "core/string_mismatch",
        "expected string to match the \"^\\S+$\" regex",
    );
    // The empty name fails the namespace rule as well.
    let diagnostics = rejected(&file(r#"{"sound_event_name": ""}"#));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.id() == "core/string_mismatch"
                && d.to_string()
                    .contains("expected string to match the \"^\\S+$\" regex")),
        "{:#?}",
        lines(&diagnostics)
    );
    for spaced in ["demo:tab\\there", "demo:x\\n", "demo:\u{a0}x"] {
        let text = file(&format!(r#"{{"sound_event_name": "{spaced}"}}"#));
        assert!(
            rejected(&text)
                .iter()
                .any(|d| d.code.id() == "core/string_mismatch"),
            "{spaced}"
        );
    }
}

#[test]
fn ssd14_the_namespace_rule() {
    let prefix = "string must be prefixed with a namespace (eg. namespace:value)";
    let invalid = "string has invalid namespace (minecraft)";
    for (name, text) in [
        ("click", prefix),
        (":click", prefix),
        ("demo:", prefix),
        ("minecraft:", prefix),
        ("minecraft:click", invalid),
        ("Minecraft:click", invalid),
        ("MINECRAFT:click", invalid),
    ] {
        let document = file(&format!(r#"{{"sound_event_name": "{name}"}}"#));
        let code = if text == prefix {
            "server_sound_definitions/needs_namespace"
        } else {
            "server_sound_definitions/invalid_namespace"
        };
        one_error(&rejected(&document), code, text);
    }
    // A longer namespace that only starts like `minecraft`, and a second colon, pass.
    for name in ["minecrafty:click", "xminecraft:click", "demo:a:b", ":a:b"] {
        let document = file(&format!(r#"{{"sound_event_name": "{name}"}}"#));
        assert!(loads(&document).diagnostics().is_empty(), "{name}");
    }
}

#[test]
fn ssd15_minecraft_is_refused_in_a_base_game_pack_too() {
    let text = file(r#"{"sound_event_name": "minecraft:click"}"#);
    let refused = parse_with(&text, &context(false, true)).unwrap_err();
    one_error(
        &refused.diagnostics,
        "server_sound_definitions/invalid_namespace",
        "string has invalid namespace (minecraft)",
    );
}

#[test]
fn ssd16_tag_patterns() {
    let regex = "expected string to match the \"^\\S+$\" regex";
    for tags in [
        r#"{ "demo:k": "a b" }"#,
        r#"{ "demo:k": ["ok", "no way"] }"#,
        r#"{ "bad key": "v" }"#,
        r#"{ "demo:k": "" }"#,
        r#"{ "": "v" }"#,
    ] {
        let text = file(&format!(
            r#"{{"sound_event_name": "demo:x", "tags": {tags}}}"#
        ));
        one_error(&rejected(&text), "core/string_mismatch", regex);
    }
    for tags in [
        r#"{ "demo:k": 3 }"#,
        r#"{ "demo:k": {"a": 1} }"#,
        r#"{ "demo:k": ["a", 3] }"#,
        r#"{ "demo:k": true }"#,
    ] {
        let text = file(&format!(
            r#"{{"sound_event_name": "demo:x", "tags": {tags}}}"#
        ));
        one_error(&rejected(&text), "core/invalid_type", "");
    }
}

#[test]
fn ssd26_tag_containers_accept_empty_and_repeats() {
    for tags in ["{}", r#"{ "demo:k": [] }"#, r#"{ "demo:k": ["a", "a"] }"#] {
        let text = file(&format!(
            r#"{{"sound_event_name": "demo:x", "tags": {tags}}}"#
        ));
        assert!(loads(&text).diagnostics().is_empty(), "{tags}");
    }
}

#[test]
fn ssd22_seconds_and_range_lints_deviation() {
    // vanilla = "accepted without check".
    for (definition, at) in [
        (
            r#""duration_info": {"mode": "game_time", "seconds": 0}"#,
            "duration_info/seconds",
        ),
        (
            r#""duration_info": {"mode": "game_time", "seconds": -1}"#,
            "duration_info/seconds",
        ),
        (r#""broadcast_range": 0"#, "broadcast_range"),
        (r#""broadcast_range": -4"#, "broadcast_range"),
    ] {
        let text = file(&format!(
            r#"{{"sound_event_name": "demo:x", {definition}}}"#
        ));
        let document = loads(&text);
        let found: Vec<_> = document.diagnostics().iter().collect();
        assert_eq!(found.len(), 1, "{definition}");
        // ours, an information
        assert_eq!(
            (found[0].code.id(), found[0].severity),
            ("core/invalid_value", Severity::Info)
        );
        assert_eq!(
            found[0].location.json_path.to_string(),
            format!("minecraft:server_sound_definitions/0/{at}")
        );
    }
    for fine in [
        r#""duration_info": {"mode": "game_time", "seconds": 0.001}"#,
        r#""broadcast_range": 0.001"#,
    ] {
        let text = file(&format!(r#"{{"sound_event_name": "demo:x", {fine}}}"#));
        assert!(loads(&text).diagnostics().is_empty(), "{fine}");
    }
}

/// A `duration_info.seconds` or a `broadcast_range` at or below 0: the game converts and binds
/// any value without a check, so the crate's note is an information.
#[test]
fn a_value_at_or_below_zero_is_an_information_of_ours() {
    for (members, at) in [
        (
            r#""duration_info": {"mode": "game_time", "seconds": 0}"#,
            "minecraft:server_sound_definitions/0/duration_info/seconds",
        ),
        (
            r#""broadcast_range": -1"#,
            "minecraft:server_sound_definitions/0/broadcast_range",
        ),
    ] {
        let document = loads(&file(&format!(
            r#"{{"sound_event_name": "demo:x", {members}}}"#
        )));
        let found: Vec<(String, Severity)> = document
            .diagnostics()
            .iter()
            .filter(|d| d.code.id() == "core/invalid_value")
            .map(|d| (d.location.json_path.to_string(), d.severity))
            .collect();
        assert_eq!(found, [(at.to_owned(), Severity::Info)], "{members}");
    }
}

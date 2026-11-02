//! Rules that the fixture rows and `edges.rs` did not reach.

use bedrock_addon::diagnostic::{DiagnosticCode as C, Diagnostics, Severity, VanillaText};
use bedrock_addon::pack::settings::SettingValue;

use super::common::{WORLD, document, loads_text, names, one_entry, refused_text};
use super::document::closes;

/// The value `banana:x` of a 1.21.100 document holding `value` (a JSON value's text).
fn value_of(value: &str) -> SettingValue {
    let text = document(
        "\"1.21.100\"",
        &one_entry(&format!(r#"{{"banana:x": {value}}}"#)),
    );
    loads_text(&text).settings[0].values["banana:x"].clone()
}

// ---- a JSON integer is read as `f32` ---------------------------------------------

#[test]
fn an_integer_of_any_size_reads_as_the_nearest_float() {
    #[allow(clippy::cast_precision_loss)]
    for (text, expected) in [
        ("16777217", 16_777_216.0_f32),
        ("9007199254740993", 9_007_199_254_740_993_u64 as f32),
        ("18446744073709551615", u64::MAX as f32),
        ("-9223372036854775808", i64::MIN as f32),
        ("123456789012345678901234567890", 1.234_567_9e29),
        ("-123456789012345678901234567890", -1.234_567_9e29),
        ("0", 0.0),
        ("-7", -7.0),
        ("2.5e3", 2500.0),
    ] {
        assert_eq!(value_of(text), SettingValue::Float(expected), "{text}");
    }
}

#[test]
fn a_string_of_digits_stays_a_string() {
    assert_eq!(
        value_of("\"123456789012345678901234567890\""),
        SettingValue::String("123456789012345678901234567890".to_owned())
    );
    assert_eq!(value_of("\"5\""), SettingValue::String("5".to_owned()));
}

#[test]
fn a_list_takes_strings_only_whatever_the_size_of_a_number_in_it() {
    for list in [
        r#"["apple", 5]"#,
        r#"["apple", 123456789012345678901234567890]"#,
        r#"[123456789012345678901234567890]"#,
        r#"["apple", ["cherry"]]"#,
    ] {
        let text = document(
            "\"1.26.40\"",
            &one_entry(&format!(r#"{{"banana:x": {list}}}"#)),
        );
        let found = refused_text(WORLD, &text);
        assert_eq!(
            names(&found, Severity::Error),
            ["InvalidType", "DocumentRejected"],
            "{list}"
        );
    }
    let text = document(
        "\"1.26.40\"",
        &one_entry(r#"{"banana:x": ["123456789012345678901234567890", "5"]}"#),
    );
    assert_eq!(
        loads_text(&text).settings[0].values["banana:x"],
        SettingValue::StringList(vec![
            "123456789012345678901234567890".to_owned(),
            "5".to_owned()
        ])
    );
}

#[test]
fn a_pack_id_that_is_a_number_of_any_size_is_a_type_error() {
    for id in ["5", "123456789012345678901234567890"] {
        let text = document(
            "\"1.21.100\"",
            &format!(r#"{{"settings": [{{"pack_id": {id}, "values": {{}}}}]}}"#),
        );
        let found = refused_text(WORLD, &text);
        assert_eq!(
            names(&found, Severity::Error),
            ["InvalidType", "DocumentRejected"],
            "{id}"
        );
        assert!(names(&found, Severity::Warning).is_empty(), "{id}");
    }
}

// ---- the foreign payload key (rule D5b) ---------------------------------------

/// A payload with one empty entry.
const EMPTY_ENTRY: &str =
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}}]}"#;

/// The `PayloadKeyMismatch` findings of `found`.
fn mismatches(found: &Diagnostics) -> usize {
    found
        .iter()
        .filter(|d| matches!(d.code, C::PayloadKeyMismatch))
        .count()
}

#[test]
fn a_comment_between_a_foreign_key_and_its_value_does_not_hide_the_key() {
    for text in [
        format!("{{\"format_version\": \"1.21.100\", \"x\" /* a */ : {EMPTY_ENTRY}}}"),
        format!("{{\"format_version\": \"1.21.100\", \"x\": /* b */ {EMPTY_ENTRY}}}"),
        format!("{{\"format_version\": \"1.21.100\", \"x\" // a\n : // b: \"y\"\n {EMPTY_ENTRY}}}"),
        format!(
            "{{// \"z\":\n \"x\": {EMPTY_ENTRY}, /* \"w\": */ \"format_version\": \"1.21.100\"}}"
        ),
    ] {
        let loaded = super::common::parse_text(WORLD, &text)
            .unwrap_or_else(|r| panic!("{text}: {:#?}", r.diagnostics));
        assert_eq!(loaded.settings.len(), 1, "{text}");
        assert_eq!(mismatches(loaded.diagnostics()), 1, "{text}");
        let warning = loaded
            .diagnostics()
            .iter()
            .find(|d| matches!(d.code, C::PayloadKeyMismatch))
            .unwrap();
        let span = warning.location.span.unwrap();
        let start = usize::try_from(span.offset).unwrap();
        let end = start + usize::try_from(span.len).unwrap();
        assert_eq!(&text[start..end], "\"x\"", "{text}");
    }
}

#[test]
fn a_foreign_key_under_a_version_without_a_parser_is_the_version_finding_alone() {
    for (version, written) in [
        ("\"1.21.100-beta.1\"", "1.21.100-beta.1"),
        ("\"beta\"", "beta"),
        ("\"1.27.0\"", "1.27.0"),
        ("\"1.26.41\"", "1.26.41"),
        ("[1, 26, 41]", "1.26.41"),
        ("\"1.26.40-beta.1\"", "1.26.40-beta.1"),
    ] {
        let text = format!(r#"{{"format_version": {version}, "x": {EMPTY_ENTRY}}}"#);
        let found = refused_text(WORLD, &text);
        assert_eq!(mismatches(&found), 0, "{version}: {found:#?}");
        let errors: Vec<&C> = found
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| &d.code)
            .collect();
        assert_eq!(errors.len(), 2, "{version}: {found:#?}");
        assert_eq!(
            errors[0],
            &C::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: written.to_owned()
                })
            },
            "{version}"
        );
        closes(&found, WORLD);
    }
}

#[test]
fn a_foreign_key_warns_at_both_ends_of_the_versions_with_a_parser() {
    for version in ["1.21.100", "1.26.39", "1.26.40"] {
        let text = format!(r#"{{"format_version": "{version}", "x": {EMPTY_ENTRY}}}"#);
        let loaded = loads_text(&text);
        assert_eq!(mismatches(loaded.diagnostics()), 1, "{version}");
    }
}

// ---- the admin API (network and script updates) ------------------------------

#[test]
fn a_slider_cannot_be_set_to_a_number_that_is_not_finite_deviation() {
    use bedrock_addon::manifest::Setting;
    use bedrock_addon::pack::settings::{PackSettings, SetError};
    let definitions = [
        Setting::Slider {
            name: "banana:slider".to_owned(),
            text: "Slider".to_owned(),
            min: 1.0,
            max: 10.0,
            step: 1.0,
            default: 3.0,
        },
        Setting::Toggle {
            name: "banana:toggle".to_owned(),
            text: "Toggle".to_owned(),
            default: false,
        },
    ];
    let mut settings = PackSettings::new(
        uuid::Uuid::nil(),
        &definitions,
        None,
        &mut Diagnostics::default(),
    );
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            settings.set("banana:slider", SettingValue::Float(x)),
            Err(SetError::NotFinite),
            "{x}"
        );
        assert_eq!(
            settings.value("banana:slider"),
            Some(&SettingValue::Float(3.0)),
            "{x}"
        );
        // For a setting of another kind the kind is what is wrong.
        assert_eq!(
            settings.set("banana:toggle", SettingValue::Float(x)),
            Err(SetError::TypeMismatch)
        );
    }
    assert!(matches!(
        SetError::NotFinite.code(),
        C::InvalidValue { text: None }
    ));
    // The largest finite numbers are clamped like any other.
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(f32::MAX)),
        Ok(true)
    );
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(10.0))
    );
    assert_eq!(
        settings.set("banana:slider", SettingValue::Float(f32::MIN)),
        Ok(true)
    );
    assert_eq!(
        settings.value("banana:slider"),
        Some(&SettingValue::Float(1.0))
    );
}

// ---- a number beyond `f32` ---------------------------------------

/// A number whose nearest `f32` is infinite has no JSON form to write back: it is read as the
/// largest finite `f32` of its sign, with an information at the value (the game reads the
/// float without a line), and the file still loads and writes. Both
/// ends: the largest finite `f32` reads as itself without a finding.
#[test]
fn a_number_beyond_f32_is_read_as_the_largest_finite_float_with_an_information_deviation() {
    for (number, expected, warned) in [
        ("1e39", f32::MAX, true),
        ("-1e39", f32::MIN, true),
        ("1e400", f32::MAX, true),
        ("-1e400", f32::MIN, true),
        ("3.5e38", f32::MAX, true),
        // The largest integer the text reader takes (2^128 - 1) is nearer to 2^128 than to
        // the largest finite `f32`.
        ("340282366920938463463374607431768211455", f32::MAX, true),
        (
            "-170141183460469231731687303715884105728",
            -1.701_411_8e38,
            false,
        ),
        ("3.4028235e38", f32::MAX, false),
        ("-3.4028235e38", f32::MIN, false),
        ("340282350000000000000000000000000000000", f32::MAX, false),
    ] {
        let text = document(
            "\"1.21.100\"",
            &one_entry(&format!(r#"{{"banana:x": {number}, "banana:y": 2}}"#)),
        );
        let loaded = loads_text(&text);
        let values = &loaded.settings[0].values;
        assert_eq!(
            values["banana:x"],
            SettingValue::Float(expected),
            "{number}"
        );
        assert_eq!(values["banana:y"], SettingValue::Float(2.0), "{number}");
        let warnings: Vec<_> = loaded
            .diagnostics()
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .map(|d| (d.code.name(), d.location.json_path.to_string()))
            .collect();
        let expected_warnings: &[_] = if warned {
            &[(
                "InvalidValue",
                "minecraft:pack_settings/settings/0/values/banana:x".to_owned(),
            )]
        } else {
            &[]
        };
        assert_eq!(warnings, expected_warnings, "{number}");
        assert!(names(loaded.diagnostics(), Severity::Error).is_empty());
        // The value read is one the writer can write, and it reads back without a finding.
        let written = loaded.value().to_json().unwrap();
        let again = loads_text(&written);
        assert_eq!(again.value(), loaded.value(), "{number}");
        assert!(
            names(again.diagnostics(), Severity::Warning).is_empty(),
            "{number}"
        );
    }
}

/// An integer beyond 128 bits (2^128, below -2^127, 400 digits) is the double it spells, as
/// any integer too large for 64 bits is to the text reader (core): the value is read like the same number written as a
/// real, and clamped with the warning where its nearest `f32` is infinite.
#[test]
fn an_integer_beyond_128_bits_is_read_as_the_double_it_spells() {
    let digits = "9".repeat(400);
    for (number, expected, warned) in [
        (
            "340282366920938463463374607431768211456".to_owned(),
            f32::MAX,
            true,
        ),
        (
            "-170141183460469231731687303715884105729".to_owned(),
            -1.701_411_8e38,
            false,
        ),
        (digits.clone(), f32::MAX, true),
        (format!("-{digits}"), f32::MIN, true),
    ] {
        let text = document(
            "\"1.21.100\"",
            &one_entry(&format!(r#"{{"banana:x": {number}, "banana:y": 2}}"#)),
        );
        let loaded = loads_text(&text);
        let values = &loaded.settings[0].values;
        assert_eq!(
            values["banana:x"],
            SettingValue::Float(expected),
            "{number}"
        );
        assert_eq!(values["banana:y"], SettingValue::Float(2.0), "{number}");
        let warnings: Vec<_> = loaded
            .diagnostics()
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .map(|d| (d.code.name(), d.location.json_path.to_string()))
            .collect();
        let expected_warnings: &[_] = if warned {
            &[(
                "InvalidValue",
                "minecraft:pack_settings/settings/0/values/banana:x".to_owned(),
            )]
        } else {
            &[]
        };
        assert_eq!(warnings, expected_warnings, "{number}");
        assert!(names(loaded.diagnostics(), Severity::Error).is_empty());
    }
}

//! The pack settings "Document" fixture rows about members and the envelope: `ps09`,
//! `ps10`, `ps11`, `ps14`, `ps15` (rules D5–D10).

use bedrock_addon::diagnostic::{DiagnosticCode as C, Diagnostics, Severity, VanillaText};
use uuid::Uuid;

use super::common::{
    WORLD, at, document, fixture, loads, loads_text, names, one_entry, parse_text, refused,
    refused_text,
};
use super::document::closes;

/// The vanilla unit tests' pack.
const PACK: Uuid = Uuid::from_u128(0xd334_9771_6782_4420_875a_fcd6_179a_8915);

/// The only error before the closing one is `code` at `path`.
fn fails_at(found: &Diagnostics, name: &str, path: &str, code: fn(&C) -> bool) {
    let errors: Vec<_> = found
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 2, "{found:#?}");
    assert!(code(&errors[0].code), "{found:#?}");
    assert_eq!(errors[0].location.json_path.to_string(), path);
    closes(found, name);
}

#[test]
fn ps09_unknown_member_is_an_error_and_drops_the_file() {
    let name = "ps09_unknown_member.json";
    fails_at(
        &refused(name),
        name,
        "minecraft:pack_settings/settings/0/comment",
        |c| matches!(c, C::UnrecognizedField { .. }),
    );
}

#[test]
fn an_unknown_member_of_the_document_is_an_error_too() {
    let text = document("\"1.21.100\"", r#"{"settings": [], "comment": "x"}"#);
    fails_at(
        &refused_text(WORLD, &text),
        WORLD,
        "minecraft:pack_settings/comment",
        |c| matches!(c, C::UnrecognizedField { .. }),
    );
}

#[test]
fn a_member_named_like_a_kept_field_is_an_unknown_member() {
    let text = document(
        "\"1.21.100\"",
        r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}, "extra": 1}]}"#,
    );
    fails_at(
        &refused_text(WORLD, &text),
        WORLD,
        "minecraft:pack_settings/settings/0/extra",
        |c| matches!(c, C::UnrecognizedField { .. }),
    );
}

#[test]
pub fn ps10_bad_uuid_loads_as_the_nil_uuid_with_an_information_deviation() {
    let loaded = loads("ps10_bad_uuid.json");
    assert_eq!(loaded.settings[0].pack_id, Uuid::nil());
    // The game's setter cannot report: an information of ours.
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["InvalidValue"]
    );
    assert!(names(loaded.diagnostics(), Severity::Warning).is_empty());
    let information = at(
        loaded.diagnostics(),
        "minecraft:pack_settings/settings/0/pack_id",
    );
    assert_eq!(information.len(), 1);
    // The nil UUID applies to no pack.
    assert!(loaded.get(PACK).is_none());
}

#[test]
fn ps10_a_uuid_without_hyphens_or_in_upper_case_is_a_uuid() {
    for id in [
        "d33497716782442087 5afcd6179a8915".replace(' ', ""),
        "D3349771-6782-4420-875A-FCD6179A8915".to_owned(),
    ] {
        let text = document(
            "\"1.21.100\"",
            &format!(r#"{{"settings": [{{"pack_id": "{id}", "values": {{}}}}]}}"#),
        );
        let loaded = loads_text(&text);
        assert_eq!(loaded.settings[0].pack_id, PACK, "{id}");
        assert!(loaded.diagnostics().is_empty(), "{id}");
    }
    // Braced and URN forms are not `mce::UUID::fromString` forms.
    for id in [
        "{d3349771-6782-4420-875a-fcd6179a8915}",
        "urn:uuid:d3349771-6782-4420-875a-fcd6179a8915",
    ] {
        let text = document(
            "\"1.21.100\"",
            &format!(r#"{{"settings": [{{"pack_id": "{id}", "values": {{}}}}]}}"#),
        );
        assert_eq!(loads_text(&text).settings[0].pack_id, Uuid::nil(), "{id}");
    }
}

#[test]
fn ps10_a_non_string_pack_id_is_a_type_error() {
    let name = "ps10_bad_uuid.json";
    let text = String::from_utf8(fixture(name))
        .unwrap()
        .replace("\"BadPackID\"", "5");
    let found = match parse_text(name, &text) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    fails_at(
        &found,
        name,
        "minecraft:pack_settings/settings/0/pack_id",
        |c| matches!(c, C::InvalidType { .. }),
    );
}

#[test]
pub fn ps11_duplicate_pack_id_loads_with_a_warning_and_the_first_entry_wins_deviation() {
    let loaded = loads("ps11_duplicate_pack_id.json");
    assert_eq!(loaded.settings.len(), 2, "the later entry is kept");
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["DuplicateIdentifier"]
    );
    let warning = at(
        loaded.diagnostics(),
        "minecraft:pack_settings/settings/1/pack_id",
    );
    assert_eq!(warning.len(), 1);
    let first = loaded.get(PACK).expect("an entry");
    assert_eq!(
        first.values["banana:slider"],
        bedrock_addon::pack::settings::SettingValue::Float(5.0)
    );
}

#[test]
fn ps14_third_root_member_is_the_envelope_error() {
    let name = "ps14_third_root_member.json";
    fails_at(&refused(name), name, "", |c| {
        matches!(
            c,
            C::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            }
        )
    });
}

#[test]
pub fn ps15_foreign_payload_key_loads_with_a_warning_deviation() {
    let loaded = loads("ps15_foreign_payload_key.json");
    assert_eq!(loaded.settings.len(), 1);
    assert_eq!(loaded.settings[0].pack_id, PACK);
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["PayloadKeyMismatch"]
    );
    assert_eq!(at(loaded.diagnostics(), "minecraft:settings").len(), 1);
}

#[test]
fn ps15_findings_under_a_foreign_payload_key_keep_their_file_offsets() {
    let text =
        r#"{"format_version": "1.21.100", "x": {"settings": [{"pack_id": 5, "values": {}}]}}"#;
    let found = refused_text(WORLD, text);
    let typed = found
        .iter()
        .find(|d| matches!(d.code, C::InvalidType { .. }))
        .expect("the type error");
    let span = typed.location.span.expect("a span");
    let start = usize::try_from(span.offset).unwrap();
    assert_eq!(
        &text[start..start + usize::try_from(span.len).unwrap()],
        "5"
    );
}

#[test]
fn d6_a_missing_required_member_is_an_error() {
    for (payload, path) in [
        (r"{}".to_owned(), "minecraft:pack_settings/settings"),
        (
            r#"{"settings": [{"values": {}}]}"#.to_owned(),
            "minecraft:pack_settings/settings/0/pack_id",
        ),
        (
            r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915"}]}"#.to_owned(),
            "minecraft:pack_settings/settings/0/values",
        ),
    ] {
        let found = refused_text(WORLD, &document("\"1.21.100\"", &payload));
        fails_at(&found, WORLD, path, |c| {
            matches!(c, C::MissingRequiredField { .. })
        });
    }
}

#[test]
fn d5a_a_missing_payload_is_the_envelope_error_not_a_missing_field() {
    // A missing `minecraft:pack_settings` is also rule D6; with one root
    // member the envelope check of D5a comes first.
    let found = refused_text(WORLD, r#"{"format_version": "1.21.100"}"#);
    fails_at(&found, WORLD, "", |c| {
        matches!(
            c,
            C::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            }
        )
    });
}

#[test]
fn d8_a_null_object_or_mixed_array_value_is_a_type_error() {
    for value in ["null", "{}", r#"["apple", 1]"#] {
        let text = document(
            "\"1.26.40\"",
            &one_entry(&format!(r#"{{"banana:x": {value}}}"#)),
        );
        let found = refused_text(WORLD, &text);
        fails_at(
            &found,
            WORLD,
            "minecraft:pack_settings/settings/0/values/banana:x",
            |c| matches!(c, C::InvalidType { .. }),
        );
    }
}

#[test]
fn d8_last_used_must_be_an_unsigned_integer() {
    for value in ["-1", "1.5", "5.0", "\"5\"", "null", "[5]"] {
        let text = document(
            "\"1.21.100\"",
            &format!(
                r#"{{"settings": [{{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": {value}, "values": {{}}}}]}}"#
            ),
        );
        let found = refused_text(WORLD, &text);
        fails_at(
            &found,
            WORLD,
            "minecraft:pack_settings/settings/0/last_used",
            |c| matches!(c, C::InvalidType { .. }),
        );
    }
}

#[test]
fn d8_settings_must_be_an_array() {
    for value in ["{}", "null", "5", "\"x\"", "true"] {
        let payload = format!(r#"{{"settings": {value}}}"#);
        let found = refused_text(WORLD, &document("\"1.21.100\"", &payload));
        fails_at(&found, WORLD, "minecraft:pack_settings/settings", |c| {
            matches!(c, C::InvalidType { .. })
        });
    }
}

#[test]
fn d8_an_entry_values_or_payload_of_another_type_is_a_type_error_at_its_path() {
    for (payload, path) in [
        (r#"{"settings": [5]}"#, "minecraft:pack_settings/settings/0"),
        (
            r#"{"settings": [null]}"#,
            "minecraft:pack_settings/settings/0",
        ),
        (
            r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": []}]}"#,
            "minecraft:pack_settings/settings/0/values",
        ),
        (
            r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": null}]}"#,
            "minecraft:pack_settings/settings/0/values",
        ),
        ("[]", "minecraft:pack_settings"),
        ("null", "minecraft:pack_settings"),
    ] {
        let found = refused_text(WORLD, &document("\"1.21.100\"", payload));
        fails_at(&found, WORLD, path, |c| matches!(c, C::InvalidType { .. }));
    }
}

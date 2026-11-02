//! The envelope probe: `{ "format_version": …, "<payload key>": {…} }` read from the
//! text into a shell whose only typed member is `format_version` and whose other members stay raw
//! slices of the text, so the payload is deserialised from its own slice.

use bedrock_addon_core::json::{EnvelopeVersion, JsonSlice, probe_envelope};

const BLOCK: &str = r#"{
  "format_version": "1.21.60",
  "minecraft:block": { "description": { "identifier": "x:y" } }
}"#;

#[test]
fn a_modern_document_has_a_version_and_one_payload_slice() {
    let envelope = probe_envelope(JsonSlice::new(BLOCK)).unwrap();
    assert_eq!(
        envelope.format_version,
        EnvelopeVersion::String("1.21.60".into())
    );
    assert_eq!(envelope.member_count(), 2);
    let payload = envelope.payload("minecraft:block").unwrap();
    assert_eq!(
        payload.text,
        r#"{ "description": { "identifier": "x:y" } }"#
    );
    assert_eq!(payload.offset, BLOCK.find("{ \"description\"").unwrap());
}

#[test]
fn the_payload_keeps_its_file_offset_inside_a_larger_slice() {
    let file = format!("    {BLOCK}");
    let envelope = probe_envelope(JsonSlice {
        text: &file[4..],
        offset: 4,
    })
    .unwrap();
    let payload = envelope.payload("minecraft:block").unwrap();
    assert_eq!(payload.offset, file.find("{ \"description\"").unwrap());
}

#[test]
fn members_keep_document_order_and_count_with_the_version() {
    let text = r#"{ "b": 1, "format_version": "1.0.0", "a": [], "c": {} }"#;
    let envelope = probe_envelope(JsonSlice::new(text)).unwrap();
    let keys: Vec<&str> = envelope.members.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["b", "a", "c"]);
    assert_eq!(envelope.member_count(), 4);
    assert_eq!(envelope.payload("a").unwrap().text, "[]");
}

#[test]
fn a_missing_version_is_absent() {
    let envelope = probe_envelope(JsonSlice::new(r#"{ "minecraft:item": {} }"#)).unwrap();
    assert_eq!(envelope.format_version, EnvelopeVersion::Absent);
    assert_eq!(envelope.member_count(), 1);
}

#[test]
fn an_array_of_three_non_negative_integers_is_a_version() {
    let envelope = probe_envelope(JsonSlice::new(
        r#"{ "format_version": [1, 20, 0], "p": {} }"#,
    ))
    .unwrap();
    assert_eq!(envelope.format_version, EnvelopeVersion::Array([1, 20, 0]));
}

#[test]
fn any_other_version_value_is_malformed() {
    for version in [
        "[1, 20]",
        "[1, 20, 0, 0]",
        "[1.0, 20, 0]",
        "[-1, 20, 0]",
        "[\"1\", 20, 0]",
        "1.2",
        "true",
        "{}",
        "null",
    ] {
        let text = format!(r#"{{ "format_version": {version}, "p": {{}} }}"#);
        let envelope = probe_envelope(JsonSlice::new(&text)).unwrap();
        assert_eq!(
            envelope.format_version,
            EnvelopeVersion::Malformed,
            "{version}"
        );
        assert_eq!(envelope.member_count(), 2, "{version}");
    }
}

/// Deviation: the game's text scan reads the first `format_version`, while the root
/// is read as a map, where the last value of a duplicate key wins silently.
#[test]
fn a_duplicate_format_version_takes_the_last_value_deviation() {
    let text = r#"{ "format_version": "1.0.0", "p": {}, "format_version": "1.2.0" }"#;
    let envelope = probe_envelope(JsonSlice::new(text)).unwrap();
    assert_eq!(
        envelope.format_version,
        EnvelopeVersion::String("1.2.0".into())
    );
    assert_eq!(envelope.member_count(), 2);
}

#[test]
fn a_duplicate_payload_key_keeps_the_last_value_at_the_first_position() {
    let text = r#"{ "p": 1, "q": 2, "p": 3 }"#;
    let envelope = probe_envelope(JsonSlice::new(text)).unwrap();
    let members: Vec<(&str, &str)> = envelope
        .members
        .iter()
        .map(|(k, s)| (k.as_str(), s.text))
        .collect();
    assert_eq!(members, [("p", "3"), ("q", "2")]);
}

#[test]
fn a_syntax_error_is_an_error_at_its_file_offset() {
    let text = r#"{ "format_version": "1.0.0", "p": { ] }"#;
    let error = probe_envelope(JsonSlice::new(text)).unwrap_err();
    assert_eq!(error.span.offset as usize, text.find(']').unwrap());
}

#[test]
fn a_root_that_is_not_an_object_is_an_error() {
    assert!(probe_envelope(JsonSlice::new("[1, 2]")).is_err());
    assert!(probe_envelope(JsonSlice::new("\"x\"")).is_err());
}

#[test]
fn comments_inside_the_payload_stay_in_its_slice() {
    let text = "{ \"format_version\": \"1.0.0\", \"p\": { /* c */ \"a\": 1 } }";
    let envelope = probe_envelope(JsonSlice::new(text)).unwrap();
    assert_eq!(envelope.payload("p").unwrap().text, "{ /* c */ \"a\": 1 }");
}

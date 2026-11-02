//! Round trips (of the newest shape, and of older documents once upgraded) and
//! what write-back keeps and never invents.

use bedrock_addon::document::Content;
use bedrock_addon::families::attachable::{Attachable, Description};
use bedrock_addon::id::AttachableId;
use bedrock_addon::json::Value;
use bedrock_addon::version::FormatVersion;

use super::common::*;

const FAMILY: &str = "attachable";

fn json(text: &str) -> Value {
    bedrock_addon::json::from_str_jsonc(text).expect("test JSON")
}

#[test]
fn the_full_1_26_shape_round_trips() {
    let text = fixture(FAMILY, "s02_1_10_to_1_26_types.after.json");
    let document = loaded(attachable(&text, &cx()));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert_eq!(json(&written), json(&text));
}

#[test]
fn write_back_writes_no_member_the_author_did_not_write() {
    // a minimal document writes its identifier and nothing else.
    let document = loaded(attachable(&ghost("1.26.0", ""), &cx()));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert_eq!(
        json(&written),
        json(
            r#"{"format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#
        )
    );
    // So does one built in code.
    let built = Attachable::new(AttachableId::parse("demo:ghost"));
    assert_eq!(built.to_json().unwrap(), written);
    assert_eq!(
        built.description,
        Description::new(AttachableId::parse("demo:ghost"))
    );
    // A written `false` and an empty object the schema allows are kept where the member has
    // a presence of its own.
    let members = r#""particle_effects": {}, "enable_attachables": false, "spawn_egg": {}, "scripts": {}, "render_controllers": []"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert_eq!(json(&written), json(&ghost("1.26.0", members)));
}

#[test]
fn held_item_scale_is_written_as_the_shortest_number_that_reads_back() {
    // 2.678 must not come back as 2.677999973297119.
    let document = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 2.678"#),
        &cx(),
    ));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert!(
        written.contains("\"held_item_scale\": 2.678\n"),
        "{written}"
    );
}

#[test]
fn held_item_scale_deviation_an_integer_token_is_written_as_a_real() {
    // the variant keeps no token kind.
    let document = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 2"#),
        &cx(),
    ));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert!(written.contains("\"held_item_scale\": 2.0\n"), "{written}");
}

#[test]
fn the_version_written_is_the_document_s_own_and_never_below_1_26_0() {
    // A newer stamp stays.
    let document = loaded(attachable(&ghost("1.26.30", ""), &cx()));
    assert_eq!(document.format_version, FormatVersion::new(1, 26, 30));
    assert!(document.to_json().unwrap().contains(r#""1.26.30""#));
    // An upgraded document is written at 1.26.0 ("Upgraded").
    let old = loaded(attachable(&ghost("1.10.0", ""), &cx()));
    assert_eq!(old.format_version, FormatVersion::new(1, 26, 0));
    assert!(old.to_json().unwrap().contains(r#""1.26.0""#));
    // A value built with an older stamp is still written as the shape it is.
    let stale = Attachable {
        format_version: FormatVersion::new(1, 10, 0),
        ..Attachable::new(AttachableId::parse("demo:ghost"))
    };
    assert!(stale.to_json().unwrap().contains(r#""1.26.0""#));
}

#[test]
fn an_upgraded_document_written_at_1_26_reads_back_to_the_same_value() {
    for name in [
        "vanilla/diamond_helmet.json",
        "vanilla/diamond_helmet.player.json",
        "s02_1_10_to_1_26_types.before.json",
    ] {
        let document = loaded(attachable(&fixture(FAMILY, name), &cx()));
        round_trip(&document, |text| attachable(text, &cx()));
    }
    // Lower-cased scripts are written lower-cased, so the 1.26.0 text means what the old one
    // did.
    let members = r#""scripts": {"pre_animation": ["Variable.X = 1;"], "scale": "Query.Is_Baby"}"#;
    let document = loaded(attachable(&ghost("1.16.0", members), &cx()));
    let written = round_trip(&document, |text| attachable(text, &cx()));
    assert!(written.contains("variable.x = 1;"), "{written}");
    assert!(written.contains("query.is_baby"), "{written}");
}

#[test]
fn a_member_named_extra_is_an_undeclared_member_like_any_other() {
    // The structs have no `extra` field for facet to bind the name to.
    let d = rejected(attachable(&ghost("1.26.0", r#""extra": 1"#), &cx()));
    expect(
        &d,
        "UnrecognizedField",
        bedrock_addon::diagnostic::Severity::Error,
        "child 'extra' not valid here.",
    );
}

#[test]
fn of_a_member_written_twice_the_last_stands() {
    // Of a member written twice, the last value stands.
    let text = r#"{"format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:first", "identifier": "demo:ghost", "hide_armor": true, "hide_armor": false}}}"#;
    let document = loaded(attachable(text, &cx()));
    assert_eq!(document.description.identifier.as_str(), "demo:ghost");
    assert!(!document.hide_armor());
    // Also for the version the loader's gate reads.
    let text = r#"{"format_version": "0.1.0", "format_version": "1.26.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#;
    loaded(attachable(text, &cx()));
    // The other way round the gate reads the last one too, and refuses it.
    let text = r#"{"format_version": "1.26.0", "format_version": "0.1.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#;
    let d = rejected(attachable(text, &cx()));
    assert!(
        find(
            &d,
            "UnsupportedVersion",
            bedrock_addon::diagnostic::Severity::Error
        )
        .is_some()
    );
}

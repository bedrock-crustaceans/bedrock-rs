//! The side-file kind on its own: `Content::parse`, the round trip (the JSON as written, read
//! back equal and rewritten identically) and what a write gives for a value built by hand.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::ui_texture_info::{Nineslice, SliceSize, UiTextureInfo};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::json;

fn parse<T: Content>(path: &str, text: &str) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::new(path).unwrap(),
        text.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Reads `text`, writes it, and requires the written text to read back equal and to rewrite
/// identically; returns the document and the written text.
fn round_trip<T: Content + PartialEq + std::fmt::Debug>(
    path: &str,
    text: &str,
) -> (Document<T>, String) {
    let document = parse::<T>(path, text).unwrap_or_else(|e| panic!("{path}: {e}"));
    let written = document.to_json().expect("it writes");
    let again = parse::<T>(path, &written).unwrap_or_else(|e| panic!("{path} rewritten: {e}"));
    assert_eq!(again.value(), document.value(), "{path}: read back");
    assert_eq!(again.to_json().unwrap(), written, "{path}: rewritten");
    assert!(again.diagnostics().is_empty() || !document.diagnostics().is_empty());
    (document, written)
}

/// A file of the vanilla side files kept beside the JSON UI fixtures.
fn vanilla(name: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui/ui");
    std::fs::read_to_string(root.join("vanilla").join(name)).unwrap()
}

/// `text` as the writer lays JSON out: what a write-back that changes nothing gives.
fn json_text(text: &str) -> String {
    bedrock_addon::json::to_json(&json(text)).unwrap()
}

/// The determinism fragment's slice loads.
#[test]
fn the_determinism_slice_loads() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ui/ui/determinism/resource_pack/textures/ui/determinism_slice.json");
    let text = std::fs::read_to_string(path).unwrap();
    assert!(parse::<UiTextureInfo>("textures/ui/d.json", &text).is_ok());
}

#[test]
fn a_vanilla_nine_slice_file_round_trips() {
    let text = vanilla("book_pageedge_left.json");
    let (info, written) = round_trip::<UiTextureInfo>("textures/ui/book_pageedge_left.json", &text);
    assert!(info.diagnostics().is_empty());
    let UiTextureInfo::Nineslice(nineslice) = info.value() else {
        panic!("a nine-slice file")
    };
    assert_eq!(
        nineslice.nineslice_size,
        Some(SliceSize::Sides([40.0, 40.0, 0.0, 40.0]))
    );
    assert_eq!(nineslice.base_size, Some([40.0, 120.0]));
    assert_eq!(nineslice.tiled, Some(true));
    assert_eq!(info.base_size(), [40.0, 120.0]);
    assert_eq!(json(&written), json(&text));
}

#[test]
fn a_vanilla_aseprite_sheet_round_trips() {
    let text = vanilla("jump-mouse.json");
    let (info, written) = round_trip::<UiTextureInfo>("textures/ui/jump-mouse.json", &text);
    assert!(info.diagnostics().is_empty());
    assert!(matches!(info.value(), UiTextureInfo::Aseprite(_)));
    // the base size is the whole sheet's, `meta.size`: five frames of 84 × 64
    assert_eq!(info.base_size(), [420.0, 64.0]);
    assert_eq!(info.frames().len(), 5);
    assert_eq!(info.frames()[1].position, [84, 0]);
    assert_eq!(info.frames()[0].duration, 1000);
    assert_eq!(json(&written), json(&text));
}

#[test]
fn texture_info_writes_what_it_was_given() {
    // fractions as the shortest number, whole numbers as integers, nothing invented
    let built = UiTextureInfo::Nineslice(Nineslice {
        nineslice_size: Some(SliceSize::All(0.1)),
        base_size: Some([16.0, 2.5]),
        ..Nineslice::default()
    });
    assert_eq!(
        built.to_json().unwrap(),
        "{\n  \"nineslice_size\": 0.1,\n  \"base_size\": [\n    16,\n    2.5\n  ]\n}\n"
    );
    assert_eq!(
        UiTextureInfo::Nineslice(Nineslice::default())
            .to_json()
            .unwrap(),
        "{}\n"
    );
    // a value the game ignores, and a rejected sheet, are written back as they were
    for text in [
        r#"{ "nineslice_size": [1, 2, 3], "base_size": 4, "tiled": "yes" }"#,
        r#"{ "frames": {}, "meta": { "image": "a.png", "size": { "w": 1, "h": 1 } } }"#,
    ] {
        let (_, written) = round_trip::<UiTextureInfo>("textures/ui/x.json", text);
        assert_eq!(json(&written), json(text));
    }
}

#[test]
fn a_side_file_that_is_no_json_object_is_rejected() {
    // T1: the game ignores the result of the parse and reads nothing; there is nothing to
    // write back either, so the document is rejected
    for text in ["{ \"nineslice_size\": ", "", "[]", "4", "null"] {
        let rejected = parse::<UiTextureInfo>("textures/ui/x.json", text).unwrap_err();
        let finding = rejected.diagnostics.iter().next().unwrap();
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(finding.code.name(), "MalformedInput"),
            "{text}"
        );
        assert_eq!(finding.severity, Severity::Error);
    }
    // every finding of a file that is read is a warning: the document loads
    let info = parse::<UiTextureInfo>(
        "textures/ui/x.json",
        r#"{ "nineslice_size": -1, "base_size": [-1, 0] }"#,
    )
    .unwrap();
    assert_eq!(info.diagnostics().len(), 2);
    assert!(
        info.diagnostics()
            .iter()
            .all(|d| d.severity == Severity::Info)
    );
    let first = info.diagnostics().iter().next().unwrap();
    assert_eq!(
        first.location.path.as_ref().unwrap().as_str(),
        "textures/ui/x.json"
    );
}

#[test]
fn a_nine_slice_member_the_typed_field_cannot_say_is_written_back_as_written() {
    let slice = |text: &str| {
        let (info, written) = round_trip::<UiTextureInfo>("textures/ui/x.json", text);
        assert_eq!(written, json_text(text), "{text}");
        let UiTextureInfo::Nineslice(nineslice) = info.into_value() else {
            panic!("a nine-slice file")
        };
        nineslice
    };
    // a number written with a fraction part stays that number
    assert_eq!(
        slice(r#"{ "nineslice_size": 4.0 }"#).nineslice_size,
        Some(SliceSize::All(4.0))
    );
    // elements the game does not read, or reads as 0 or 1, stay
    assert_eq!(
        slice(r#"{ "base_size": [3, 5, 9] }"#).base_size,
        Some([3.0, 5.0])
    );
    assert_eq!(
        slice(r#"{ "nineslice_size": ["a", true, null, 2] }"#).nineslice_size,
        Some(SliceSize::Sides([0.0, 1.0, 0.0, 2.0]))
    );
    assert_eq!(
        slice(r#"{ "nineslice_size": true }"#).nineslice_size,
        Some(SliceSize::All(1.0))
    );
    // a null member is no value and stays a member
    let nulls =
        slice(r#"{ "nineslice_size": null, "base_size": null, "tiled": null, "other": null }"#);
    assert_eq!(
        (nulls.nineslice_size, nulls.base_size, nulls.tiled),
        (None, None, None)
    );
    // what the typed fields say exactly needs no copy beside them
    let exact =
        slice(r#"{ "nineslice_size": [1, 2, 3, 4], "base_size": [16, 2.5], "tiled": false }"#);
    assert!(exact.extra.is_empty());
    assert!(slice(r#"{ "nineslice_size": 0.5 }"#).extra.is_empty());
}

#[test]
fn a_changed_typed_field_is_written_instead_of_the_kept_member() {
    let read = |text: &str| {
        let info = parse::<UiTextureInfo>("textures/ui/x.json", text).unwrap();
        let UiTextureInfo::Nineslice(nineslice) = info.into_value() else {
            panic!("a nine-slice file")
        };
        nineslice
    };
    let mut nineslice =
        read(r#"{ "nineslice_size": 4.0, "base_size": [3, 5, 9], "tiled": "yes" }"#);
    nineslice.nineslice_size = Some(SliceSize::All(2.0));
    nineslice.base_size = Some([8.0, 8.0]);
    nineslice.tiled = Some(true);
    assert_eq!(
        UiTextureInfo::Nineslice(nineslice).to_json().unwrap(),
        json_text(r#"{ "nineslice_size": 2, "base_size": [8, 8], "tiled": true }"#)
    );
    // a field cleared by hand is not written, kept member or not
    let mut cleared = read(r#"{ "nineslice_size": 4.0, "base_size": [3, 5, 9] }"#);
    cleared.nineslice_size = None;
    cleared.base_size = None;
    assert_eq!(UiTextureInfo::Nineslice(cleared).to_json().unwrap(), "{}\n");
}

#[test]
fn a_sheet_number_written_with_a_fraction_part_is_written_as_an_integer() {
    // recorded: jsoncpp's `isInt` accepts `100.0`; the typed sheet holds integers
    let text = r#"{ "frames": [ { "frame": { "x": 0.0, "y": 0 }, "duration": 100.0 } ],
        "meta": { "image": "a.png", "size": { "w": 8.0, "h": 8 } } }"#;
    let (info, written) = round_trip::<UiTextureInfo>("textures/ui/x.json", text);
    assert!(matches!(info.value(), UiTextureInfo::Aseprite(_)));
    assert_eq!(
        json(&written),
        json(
            r#"{ "frames": [ { "frame": { "x": 0, "y": 0 }, "duration": 100 } ],
            "meta": { "image": "a.png", "size": { "w": 8, "h": 8 } } }"#
        )
    );
    assert!(written.contains("\"duration\": 100\n"), "{written}");
}

/// A nine-slice value built by hand with a negative zero is written with its sign: the writer
/// writes a whole number as an integer, and an integer zero has none.
#[test]
fn a_negative_zero_of_a_nine_slice_is_written_with_its_sign() {
    let info = UiTextureInfo::Nineslice(Nineslice {
        nineslice_size: Some(SliceSize::Sides([-0.0, 4.0, 0.0, 2.5])),
        base_size: Some([-0.0, 16.0]),
        ..Nineslice::default()
    });
    let written = info.to_json().expect("written");
    let line: String = written.split_whitespace().collect();
    assert!(
        line.contains(r#""nineslice_size":[-0.0,4,0,2.5]"#),
        "{written}"
    );
    assert!(line.contains(r#""base_size":[-0.0,16]"#), "{written}");
}

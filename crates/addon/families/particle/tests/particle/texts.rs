//! The legacy parser's two texts: "offset field
//! required!" for a box without `half_dimensions` and "The direction sub section is not
//! properly defined!" for a billboard whose direction fails.

use super::common::{
    CEREAL, LEGACY, assert_lacks, effect, findings, fixture, loaded, rejected, texts,
};

const BOX: &str = "particle_effect/components/minecraft:emitter_shape_box";
const BILLBOARD: &str = "particle_effect/components/minecraft:particle_appearance_billboard";

#[test]
fn a_legacy_box_without_half_dimensions_says_offset_field_required_and_skips_the_rest() {
    // The box component reads `offset`, then the missing
    // `half_dimensions` logs the text and the parse returns (`direction`, `surface_only`
    // unread).
    let text = effect(
        LEGACY,
        r#""minecraft:emitter_shape_box": {"offset": [1, 2, 3], "surface_only": true, "direction": "inwards"}"#,
    );
    // The text is a content-log error after which the parse returns and the effect loads:
    // a kept error; the members the game leaves unread are crate-own lines, Info.
    let document = loaded(&text);
    assert_kept_errors(document.diagnostics(), 1);
    assert_eq!(
        findings(document.diagnostics()),
        [
            format!("Error {BOX}/half_dimensions particle/offset_required: offset field required!"),
            format!("Info {BOX}/surface_only core/invalid_value: invalid value"),
            format!("Info {BOX}/direction core/invalid_value: invalid value"),
        ]
    );
    // With `half_dimensions` the box reads on, without a line.
    let whole = effect(
        LEGACY,
        r#""minecraft:emitter_shape_box": {"half_dimensions": [1, 1, 1], "surface_only": true, "direction": "inwards"}"#,
    );
    assert_eq!(findings(loaded(&whole).diagnostics()), Vec::<String>::new());
    // The cereal slice requires the member (unchanged).
    let cereal = rejected(&effect(
        CEREAL,
        r#""minecraft:emitter_shape_box": {"offset": [1, 2, 3]}"#,
    ));
    assert_eq!(
        texts(&cereal),
        [format!(
            "Error {BOX}/half_dimensions core/missing_required_field: missing required field"
        )]
    );
}

#[test]
fn a_legacy_billboard_whose_direction_fails_says_so_and_skips_its_uv() {
    // The billboard component's direction read returned false (no object, or `custom` without `custom_direction`), so the text, and the
    // parse returns before `uv`.
    const PROPERLY: &str = "The direction sub section is not properly defined!";
    let uv =
        r#""uv": {"texture_width": 16, "texture_height": 16, "uv": [1, 1], "uv_size": [2, 2]}"#;
    for (direction, id, first) in [
        (
            r#""x""#,
            "direction_not_object",
            "The direction subsection is not defined as a proper json object!",
        ),
        (
            r#"{"mode": "custom"}"#,
            "direction_custom",
            "The 'direction' sub section is not defined as a proper json object!",
        ),
    ] {
        let text = effect(
            LEGACY,
            &format!(
                r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "direction": {direction}, {uv}}}"#
            ),
        );
        // The line is a content-log error after which the
        // parse returns and the effect loads: a kept error; the unread uv is a
        // crate-own line, Info.
        let document = loaded(&text);
        // and the not-object `direction` line of the legacy parser, kept too
        assert_kept_errors(document.diagnostics(), 2);
        assert_eq!(
            findings(document.diagnostics()),
            [
                format!("Error {BILLBOARD}/direction particle/{id}: {first}"),
                format!(
                    "Error {BILLBOARD}/direction particle/direction_not_properly_defined: {PROPERLY}"
                ),
                format!("Info {BILLBOARD}/uv core/invalid_value: invalid value"),
            ],
            "{direction}"
        );
    }
    // A mode outside the two logs its line and returns true: the uv is read.
    let mode = effect(
        LEGACY,
        &format!(
            r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "direction": {{"mode": "sideways"}}, {uv}}}"#
        ),
    );
    let document = loaded(&mode);
    assert_lacks(document.diagnostics(), PROPERLY);
    assert_lacks(document.diagnostics(), "invalid value");
    // The cereal slice has no such line.
    let cereal = effect(
        CEREAL,
        &format!(
            r#""minecraft:particle_appearance_billboard": {{"size": [1, 1], "direction": {{"mode": "custom"}}, {uv}}}"#
        ),
    );
    assert_lacks(loaded(&cereal).diagnostics(), PROPERLY);
}

#[test]
fn the_oracle_documents_load_without_either_text() {
    // The engine's embedded 1.10.0 documents (each must load) and the vanilla 1.26.50 poplar
    // leaves (a box with `half_dimensions`, a billboard with a direction).
    for file in [
        "texts/atomic_entity-1.json",
        "texts/atomic_entity-2.json",
        "texts/emitter-1.json",
        "texts/red_poplar_leaves_particle.json",
        "texts/orange_poplar_leaves_particle.json",
        "texts/yellow_poplar_leaves_particle.json",
    ] {
        let document = loaded(&fixture(file));
        assert_lacks(document.diagnostics(), "offset field required!");
        assert_lacks(
            document.diagnostics(),
            "The direction sub section is not properly defined!",
        );
        assert!(
            document
                .diagnostics()
                .iter()
                .all(|d| d.severity < bedrock_addon_core::diagnostic::Severity::Error),
            "{file}: {:#?}",
            texts(document.diagnostics())
        );
    }
}

/// Every error of `found` is a kept error, and there are `count` of them.
#[track_caller]
fn assert_kept_errors(found: &bedrock_addon_core::diagnostic::Diagnostics, count: usize) {
    let errors: Vec<_> = found
        .iter()
        .filter(|d| d.severity == bedrock_addon_core::diagnostic::Severity::Error)
        .collect();
    assert_eq!(errors.len(), count, "{found:?}");
    assert!(errors.iter().all(|d| d.kept), "{found:?}");
}

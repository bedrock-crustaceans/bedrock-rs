//! `FormatVersionInput::pick` and the helpers on what it found: one result per way a text can
//! state (or fail to state) its version, read through `with(..).or_absent(..)` and
//! `or_else(absent, malformed)` as the hand matches read it.

use bedrock_addon_core::version::{
    FormatVersion, FormatVersionInput, MappedVersion, PickedVersion,
};

const ABSENT: &str = r#"{"minecraft:toy": {}}"#;
/// `"format_version"` only as a string value, never a key: passed over, so absent.
const ONLY_A_VALUE: &str = r#"{"name": "format_version", "minecraft:toy": {}}"#;
const MALFORMED: &str = r#"{"format_version": "one", "minecraft:toy": {}}"#;
const UNCLOSED: &str = r#"{"format_version": [1, 2"#;
const VERSION: &str = r#"{"format_version": "1.21.20", "minecraft:toy": {}}"#;
const BETA: &str = r#"{"format_version": "beta", "minecraft:toy": {}}"#;

fn written(text: &str) -> String {
    FormatVersionInput::pick(text)
        .with(|_, written| written.to_owned())
        .or_absent(String::new())
}

#[test]
fn the_four_outcomes_of_pick() {
    assert_eq!(FormatVersionInput::pick(ABSENT), PickedVersion::Absent);
    assert_eq!(
        FormatVersionInput::pick(ONLY_A_VALUE),
        PickedVersion::Absent
    );
    assert_eq!(
        FormatVersionInput::pick(MALFORMED),
        PickedVersion::Malformed
    );
    assert_eq!(FormatVersionInput::pick(UNCLOSED), PickedVersion::Malformed);
    let PickedVersion::Version { input, written } = FormatVersionInput::pick(VERSION) else {
        panic!("a version");
    };
    assert_eq!(input.version, Some(FormatVersion::new(1, 21, 20)));
    assert_eq!(written, "1.21.20");
    let PickedVersion::Version { input, .. } = FormatVersionInput::pick(BETA) else {
        panic!("the beta literal");
    };
    assert!(input.beta_literal);
    assert_eq!(input.version, None);
}

#[test]
fn with_maps_the_version_and_keeps_absent_and_malformed_apart() {
    let mapped = |text| FormatVersionInput::pick(text).with(|input, _| input.version);
    assert_eq!(mapped(ABSENT), MappedVersion::Absent);
    assert_eq!(mapped(MALFORMED), MappedVersion::Malformed);
    assert_eq!(
        mapped(VERSION),
        MappedVersion::Version(Some(FormatVersion::new(1, 21, 20)))
    );
    assert_eq!(mapped(BETA), MappedVersion::Version(None));
}

/// The five hand matches that give Absent and Malformed one answer (particle, poi and
/// `voxel_shape` take the text as written, `crafting_catalog` the input, trade a test on it).
#[test]
fn or_absent_answers_absent_and_malformed_alike() {
    assert_eq!(written(ABSENT), "");
    assert_eq!(written(MALFORMED), "");
    assert_eq!(written(VERSION), "1.21.20");
    let below = |text| {
        FormatVersionInput::pick(text)
            .with(|input, _| {
                input
                    .version
                    .is_some_and(|v| v < FormatVersion::new(1, 21, 30))
            })
            .or_absent(false)
    };
    assert!(!below(ABSENT));
    assert!(!below(MALFORMED));
    assert!(below(VERSION));
    // crafting_catalog's input, or none.
    for text in [ABSENT, MALFORMED, VERSION, BETA] {
        let input = FormatVersionInput::pick(text)
            .with(|input, _| Some(*input))
            .or_absent(None);
        assert_eq!(
            input,
            FormatVersionInput::pick(text).version().copied(),
            "{text}"
        );
    }
}

#[test]
fn or_else_answers_absent_and_malformed_each() {
    let say = |text| {
        FormatVersionInput::pick(text)
            .with(|_, written| format!("version {written}"))
            .or_else(|| "absent".to_owned(), || "malformed".to_owned())
    };
    assert_eq!(say(ABSENT), "absent");
    assert_eq!(say(ONLY_A_VALUE), "absent");
    assert_eq!(say(MALFORMED), "malformed");
    assert_eq!(say(UNCLOSED), "malformed");
    assert_eq!(say(VERSION), "version 1.21.20");
    assert_eq!(say(BETA), "version beta");
}

#[test]
fn the_accessors_read_the_three_arms() {
    let version = FormatVersionInput::pick(VERSION);
    assert_eq!(
        version.version().and_then(|input| input.version),
        Some(FormatVersion::new(1, 21, 20))
    );
    assert_eq!(version.written(), Some("1.21.20"));
    assert!(!version.is_malformed());
    let malformed = FormatVersionInput::pick(MALFORMED);
    assert!(malformed.is_malformed());
    assert_eq!(malformed.version(), None);
    assert_eq!(malformed.written(), None);
    let absent = FormatVersionInput::pick(ABSENT);
    assert!(!absent.is_malformed());
    assert_eq!(absent.version(), None);
}

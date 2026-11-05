//! The modern document envelope and the version handling around it: each case with its
//! diagnostic code, matched on variants.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::{DocumentSource, Experiment};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, BetaPermission, ComponentTable, Envelope, EnvelopeCheck, ExperimentalField,
    Family, InputKind, LoadOutcome, Loaded, MissingVersion, ParseCx, Reading, StructIndex,
    UnknownMask, VersionStruct, Versioned, check_experimental_fields, file_version, load,
    parse_payload, write_payload,
};
use facet::Facet;

use crate::toy::{Toy, Toy120, ToyBeta, ToyComponent, v1_0_0};

pub fn load_with<F: Family>(text: &str, cx: &LoadContext) -> LoadOutcome<F> {
    load::<F>(JsonSlice::new(text), cx)
}

pub fn load_toy(text: &str) -> LoadOutcome<Toy> {
    load_with::<Toy>(text, &LoadContext::detached())
}

/// The codes of an outcome, in order, with their severities.
pub fn codes<F: Family>(outcome: &LoadOutcome<F>) -> Vec<(DiagnosticCode, Severity)> {
    outcome
        .diagnostics
        .iter()
        .map(|d| (d.code.clone(), d.severity))
        .collect()
}

fn the_only_error<F: Family>(outcome: &LoadOutcome<F>) -> DiagnosticCode {
    assert!(outcome.value.is_none(), "the document is rejected");
    let errors: Vec<_> = outcome
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "one error: {:?}", outcome.diagnostics);
    errors[0].code.clone()
}

/// Families that differ from the toy in one envelope rule; they share its structs.
macro_rules! toy_variant {
    ($name:ident, $module:ident, beta = $beta:ty, { $($rule:tt)* }) => {
        pub struct $name;
        pub mod $module {
            use super::*;
            bedrock_addon_core::version_chain! {
                family = $name;
                /// The toy's chain for another family.
                pub enum Any {
                    V1_0_0(v1_0_0::Toy),
                    V1_1_0(crate::toy::v1_1_0::Toy),
                    V1_2_0(Toy120),
                }
                beta = Beta(ToyBeta);
            }
        }
        impl Family for $name {
            const NAME: &'static str = "toy";
            const PAYLOAD_KEY: &'static str = "minecraft:toy";
            const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
            type Latest = Toy120;
            type Beta = $beta;
            type Component = ToyComponent;
            type Any = $module::Any;
            fn components() -> &'static ComponentTable<Self> {
                &ComponentTable::EMPTY
            }
            $($rule)*
        }
    };
}

toy_variant!(Strict, strict, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };
});
toy_variant!(Quiet, quiet, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        missing: MissingVersion::Floor {
            text: None,
            severity: None,
        },
        ..Envelope::DEFAULT
    };
});
toy_variant!(Texted, texted, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        // Client entity, attachable: the game's text, logged at level 2.
        missing: MissingVersion::Floor {
            text: Some(VanillaText::NoFormatVersionField),
            severity: Some(Severity::Warning),
        },
        ..Envelope::DEFAULT
    };
});
toy_variant!(TextedInfo, texted_info, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        // Entity: the crate's own text, and the game is silent.
        missing: MissingVersion::Floor {
            text: Some(VanillaText::NoFormatVersionField),
            severity: Some(Severity::Info),
        },
        ..Envelope::DEFAULT
    };
});
toy_variant!(StrictTexted, strict_texted, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        // Item, fog, the catalog: rejected with the family's own text.
        missing: MissingVersion::Reject {
            text: Some(VanillaText::NoFormatVersionField),
        },
        ..Envelope::DEFAULT
    };
});
toy_variant!(Open, open, beta = ToyBeta, {
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
});
toy_variant!(ToggleOnly, toggle_only, beta = ToyBeta, {
    const BETA_PERMISSION: BetaPermission = BetaPermission::BetaApis;
});
toy_variant!(Uncounted, uncounted, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        check: EnvelopeCheck::Uncounted,
        ..Envelope::DEFAULT
    };
});
toy_variant!(Mixed, mixed, beta = ToyBeta, {
    // The item's split: the whole file a `MinecraftDocumentInput`, a JsonCpp chain below
    // the first modern slice (N1: a `null` member reads as absent), cereal slices from it (N2: a
    // `null` member is rejected); the beta struct is N2.
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(i) if i < 2 => Reading::NULL_ABSENT,
            _ => Reading::DOCUMENT.with_unknown(UnknownMask::Warn),
        }
    }
});
toy_variant!(Sourced, sourced, beta = ToyBeta, {
    // The item's split: JSON gates on the toggle alone, MCB meets the game's own gate.
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        Reading::DOCUMENT.with_unknown(UnknownMask::Warn)
    }
    fn beta_permission(_: JsonSlice<'_>, cx: &LoadContext) -> BetaPermission {
        match cx.source {
            DocumentSource::Mcb => BetaPermission::BetaApisOrBaseGame,
            _ => BetaPermission::BetaApis,
        }
    }
});
toy_variant!(Masked, masked, beta = ToyBeta, {
    // The legacy actor's split: unstrict below 1.2.0, strict from it (the strict mask turns unknown
    // members and components into errors).
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(i) if i < 2 => Reading::OWN,
            _ => Reading::OWN.with_unknown(UnknownMask::Error),
        }
    }
});
toy_variant!(Document, document, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        Reading::DOCUMENT.with_unknown(UnknownMask::Warn)
    }
});
toy_variant!(JsonCppToy, json_cpp_toy, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::JsonCpp,
        ..Envelope::DEFAULT
    };
});
toy_variant!(DocumentStrict, document_strict, beta = ToyBeta, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
});

/// A family without a beta struct.
pub struct Stable;
pub mod stable {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Stable;
        /// The toy's chain without the beta struct.
        pub enum Any {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
    }
}
impl Family for Stable {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Toy120;
    type Beta = std::convert::Infallible;
    type Component = ToyComponent;
    type Any = stable::Any;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
}

/// The payload of a raw-path document.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RawPayload {
    pub description: crate::toy::Description,
    #[facet(skip)]
    pub extra: Extra,
}

/// The root of a raw-path document: the struct is read from the whole text.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RawRoot {
    pub format_version: String,
    #[facet(rename = "minecraft:toy")]
    pub toy: RawPayload,
    #[facet(skip)]
    pub extra: Extra,
}

impl Versioned for RawRoot {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for RawRoot {
    fn parse(
        payload: JsonSlice<'_>,
        cx: &ParseCx<'_>,
        d: &mut bedrock_addon_core::diagnostic::Diagnostics,
    ) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

macro_rules! raw_family {
    ($name:ident, $module:ident, $check:expr $(, $molang:ident)?) => {
        pub struct $name;
        pub mod $module {
            use super::*;
            bedrock_addon_core::version_chain! {
                family = $name;
                /// One raw struct.
                pub enum Any {
                    V1_0_0(RawRoot),
                }
                $(molang = $molang;)?
            }
        }
        impl Family for $name {
            const NAME: &'static str = "raw toy";
            const PAYLOAD_KEY: &'static str = "minecraft:toy";
            const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
            const ENVELOPE: Envelope = Envelope {
                check: $check,
                ..Envelope::DEFAULT
            };
            type Latest = RawRoot;
            type Beta = std::convert::Infallible;
            type Component = ToyComponent;
            type Any = $module::Any;
            fn components() -> &'static ComponentTable<Self> {
                &ComponentTable::EMPTY
            }
        }
    };
}

raw_family!(Raw, raw, EnvelopeCheck::Never, none);
raw_family!(
    RawBelow,
    raw_below,
    EnvelopeCheck::From(FormatVersion::new(1, 1, 0))
);

const OK_1_2_0: &str =
    r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#;

#[test]
fn a_two_member_envelope_loads_its_payload() {
    let outcome = load_toy(OK_1_2_0);
    assert!(
        matches!(outcome.value, Some(Loaded::Stable(_))),
        "{:?}",
        outcome.diagnostics
    );
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.original_version, FormatVersion::new(1, 2, 0));
}

#[test]
fn a_syntax_error_is_malformed_input_with_the_document_text() {
    let code = the_only_error(&load_toy(
        r#"{"format_version": "1.2.0", "minecraft:toy": {"#,
    ));
    assert!(
        matches!(
            code,
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentJson { .. }),
                ..
            }
        ),
        "{code:?}"
    );
}

/// A syntax fault in a copy of a member that loses to a later one is the document's
/// fault: the game's parser reads the whole text, whichever copy it keeps.
#[test]
fn a_syntax_error_in_a_copy_that_loses_is_malformed_input() {
    const TOY: &str = r#"{"description": {"identifier": "demo:x"}}"#;
    for (faulty, inner) in [
        // The payload written twice, the first with the fault.
        (r#"{"description": {"identifier": tru}}"#, false),
        (r#"{"description": [1 2]}"#, false),
        (r#"{"description": {"identifier": }}"#, false),
        (r"[1,, 2]", false),
        (r#"{"a" 1}"#, false),
        // A member of the payload written twice.
        (r#"{"identifier": tru}"#, true),
        (r#"{"identifier": "demo:y" "x": 1}"#, true),
        (r"[1 2]", true),
    ] {
        let text = if inner {
            format!(
                r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {faulty}, "description": {{"identifier": "demo:x"}}}}}}"#
            )
        } else {
            format!(
                r#"{{"format_version": "1.2.0", "minecraft:toy": {faulty}, "minecraft:toy": {TOY}}}"#
            )
        };
        let code = the_only_error(&load_toy(&text));
        assert!(
            matches!(
                code,
                DiagnosticCode::MalformedInput {
                    text: Some(VanillaText::DocumentJson { .. }),
                    ..
                }
            ),
            "{text}: {code:?}"
        );
    }
    // A copy that loses and is well formed loses in silence.
    let text = format!(
        r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": [1, 2]}}, "minecraft:toy": {TOY}}}"#
    );
    let outcome = load_toy(&text);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[test]
fn a_root_that_is_not_an_object_is_malformed_input() {
    for text in [r"[1, 2]", r#""text""#, "3"] {
        let code = the_only_error(&load_toy(text));
        assert!(
            matches!(code, DiagnosticCode::MalformedInput { .. }),
            "{text}: {code:?}"
        );
    }
}

#[test]
fn an_empty_or_null_file_is_an_empty_document() {
    for text in ["", "  \n", "// only a comment", "null"] {
        let code = the_only_error(&load_toy(text));
        assert!(
            matches!(code, DiagnosticCode::EmptyDocument { .. }),
            "{text:?}: {code:?}"
        );
    }
}

#[test]
fn a_third_root_member_is_malformed_input_with_the_member_count_text() {
    let code = the_only_error(&load_toy(
        r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}, "other": 1}"#,
    ));
    assert!(
        matches!(
            code,
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            }
        ),
        "{code:?}"
    );
}

/// The item loader never counts the root's members; a third one
/// (`use_beta_features` included) is read by nothing: told, and neither rejects nor gates
/// the document. The counting envelope rejects the same text.
#[test]
fn a_third_root_member_passes_the_uncounted_envelope_and_fails_the_counted_one() {
    for third in [r#""other": 1"#, r#""use_beta_features": true"#] {
        let text = format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}, {third}}}"#
        );
        let outcome = load_with::<Uncounted>(&text, &LoadContext::detached());
        assert!(
            matches!(outcome.value, Some(Loaded::Stable(_))),
            "{third}: {:?}",
            outcome.diagnostics
        );
        let found = codes(&outcome);
        // The game says nothing of it; the crate reports it as an information.
        assert_eq!(
            found,
            [(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info
            )],
            "{third}"
        );
        assert!(matches!(
            the_only_error(&load_toy(&text)),
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            }
        ));
    }
    // The payload is still read from under its key.
    let code = the_only_error(&load_with::<Uncounted>(
        r#"{"format_version": "1.2.0", "other": {}}"#,
        &LoadContext::detached(),
    ));
    assert_eq!(code, DiagnosticCode::MissingPayload { text: None });
}

#[test]
fn a_payload_under_another_key_is_a_missing_payload() {
    let code = the_only_error(&load_toy(
        r#"{"format_version": "1.2.0", "minecraft:other": {"description": {"identifier": "demo:x"}}}"#,
    ));
    // The game's own text for a payload under the wrong key.
    assert_eq!(
        code.to_string(),
        "The payload key does not match the expected one. Expected: 'minecraft:toy'. Actual: 'minecraft:other'."
    );
    assert_eq!(
        code,
        bedrock_addon_core::diagnostic::cereal::payload_key_mismatch(
            "minecraft:toy",
            "minecraft:other"
        )
    );
}

#[test]
fn an_unreadable_format_version_is_malformed() {
    for version in [
        "5",
        "null",
        "true",
        r#"{"a": 1}"#,
        "[1, 2]",
        "[1, 2, -3]",
        "[1, 2, 3.5]",
        "[1, 2, 4294967296]",
        r#""x.y""#,
        r#""01.2.0""#,
        r#""*""#,
    ] {
        let text = format!(
            r#"{{"format_version": {version}, "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}}}"#
        );
        let code = the_only_error(&load_toy(&text));
        assert_eq!(
            code,
            DiagnosticCode::MalformedFormatVersion { text: None },
            "{version}"
        );
    }
}

#[test]
fn an_array_of_three_integers_is_a_version() {
    let outcome = load_toy(
        r#"{"format_version": [1, 1, 0], "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 1, 0));
    assert_eq!(outcome.version.written, Some(FormatVersion::new(1, 1, 0)));
}

// ---- the version read of a `MinecraftDocumentInput` family ----

fn toy_at(version: &str) -> String {
    format!(
        r#"{{"format_version": {version}, "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}}}"#
    )
}

/// The game stores each element of an array version in 16 bits, so an element wraps modulo
/// 65 536, on either input.
#[test]
fn an_array_element_above_65535_is_truncated() {
    let text = toy_at("[1, 2, 70000]");
    let root = load_toy(&text);
    assert!(root.value.is_some(), "{:?}", root.diagnostics);
    assert_eq!(root.version.written, Some(FormatVersion::new(1, 2, 4464)));
    let document = load_with::<Document>(&text, &LoadContext::detached());
    assert!(document.value.is_some(), "{:?}", document.diagnostics);
    assert_eq!(
        document.version.written,
        Some(FormatVersion::new(1, 2, 4464))
    );
}

/// The game reads the version value with numbers as strings, so on the
/// document input `1` is 1.0.0 and `1.2` is 1.2.0; the root reading keeps refusing a number.
#[test]
fn a_document_input_reads_a_number_as_its_text() {
    for (version, parsed) in [
        ("1", FormatVersion::new(1, 0, 0)),
        ("1.1", FormatVersion::new(1, 1, 0)),
        ("1.2", FormatVersion::new(1, 2, 0)),
    ] {
        let text = toy_at(version);
        let outcome = load_with::<Document>(&text, &LoadContext::detached());
        assert!(
            outcome.value.is_some(),
            "{version}: {:?}",
            outcome.diagnostics
        );
        assert_eq!(outcome.version.written, Some(parsed), "{version}");
        assert_eq!(
            the_only_error(&load_toy(&text)),
            DiagnosticCode::MalformedFormatVersion { text: None },
            "{version} on the root reading"
        );
    }
}

/// A present value that gives no version is a malformed format version on the document
/// input, with no fallback.
#[test]
fn a_document_input_with_an_unreadable_version_is_malformed() {
    for version in [
        "null",
        "true",
        "{}",
        r#""*""#,
        r#""01.2.0""#,
        r#""1.2.3.4""#,
        "-1",
        "1e2",
        "01",
        "[1, 2]",
        "[1.0, 2, 0]",
        "[1, -2, 0]",
    ] {
        let code = the_only_error(&load_with::<Document>(
            &toy_at(version),
            &LoadContext::detached(),
        ));
        assert_eq!(
            code,
            DiagnosticCode::MalformedFormatVersion { text: None },
            "{version}"
        );
    }
}

/// The game scans the file as written for the version: of two root `format_version` members
/// the first is the version, though the last one is the member that stands.
#[test]
fn a_document_input_takes_the_first_of_two_root_format_versions() {
    let text = r#"{"format_version": "1.0.0", "format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#;
    let document = load_with::<Document>(text, &LoadContext::detached());
    assert!(document.value.is_some(), "{:?}", document.diagnostics);
    assert_eq!(document.version.written, Some(FormatVersion::new(1, 0, 0)));
    let root = load_toy(text);
    assert_eq!(root.version.written, Some(FormatVersion::new(1, 2, 0)));
}

/// `MinecraftDocumentInput::initAsJson` takes the version from the first textual
/// `"format_version"` of the file, wherever it is; the root reading takes the root member.
#[test]
fn a_document_input_reads_the_first_textual_format_version() {
    let text = r#"{"minecraft:toy": {"format_version": "1.0.0", "description": {"identifier": "demo:x"}}, "format_version": "1.2.0"}"#;
    let document = load_with::<Document>(text, &LoadContext::detached());
    assert!(document.value.is_some(), "{:?}", document.diagnostics);
    assert_eq!(document.version.written, Some(FormatVersion::new(1, 0, 0)));
    let root = load_toy(text);
    assert_eq!(root.version.written, Some(FormatVersion::new(1, 2, 0)));
    // A version found only inside the payload makes the document versioned: its root must
    // then hold exactly the version and the payload.
    let code = the_only_error(&load_with::<Document>(
        r#"{"minecraft:toy": {"format_version": "1.2.0", "description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    ));
    assert!(
        matches!(
            code,
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            }
        ),
        "{code:?}"
    );
    // No `"format_version"` anywhere: the missing-version rule, as on the root reading.
    let outcome = load_with::<Document>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.version.written, None);
}

/// Without a text of the game the note is an information of the crate's own.
#[test]
fn a_missing_version_is_told_and_parses_at_the_floor() {
    let outcome = load_toy(r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::MissingFormatVersion { text: None },
            Severity::Info
        )]
    );
    assert_eq!(outcome.version.written, None);
    assert_eq!(outcome.version.effective, FormatVersion::new(1, 0, 0));
    assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 0, 0));
    assert!(outcome.version.upgraded);
}

/// Without a version the root's members are not counted; one beside the payload
/// is read by nothing and held nowhere, and is told where it is.
#[test]
fn a_root_member_beside_the_payload_of_a_document_without_a_version_is_told() {
    let outcome =
        load_toy(r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}, "third": 3}"#);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    let third = outcome
        .diagnostics
        .iter()
        .find(|d| d.location.json_path.to_string() == "third")
        .expect("a finding at the member");
    assert!(
        matches!(third.code, DiagnosticCode::UnrecognizedField { .. }),
        "{:?}",
        third.code
    );
    // The game says nothing here; the crate reports an information.
    assert_eq!(third.severity, Severity::Info);
}

#[test]
fn a_missing_version_is_silent_where_the_family_says_so() {
    let outcome = load_with::<Quiet>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    );
    assert!(outcome.value.is_some());
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[test]
fn a_missing_version_is_an_error_in_a_rejecting_family() {
    let code = the_only_error(&load_with::<Strict>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    ));
    assert_eq!(
        code,
        DiagnosticCode::MissingFormatVersion {
            text: Some(VanillaText::NoValidVersion)
        }
    );
}

/// A family with a text of the game for the missing version is told it as a warning.
#[test]
fn a_missing_version_with_the_family_s_text_is_a_warning() {
    let outcome = load_with::<Texted>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::NoFormatVersionField)
            },
            Severity::Warning
        )]
    );
    assert_eq!(outcome.version.effective, FormatVersion::new(1, 0, 0));
}

/// The family's text at the severity the family names.
#[test]
fn a_missing_version_with_the_family_s_text_takes_its_severity() {
    let outcome = load_with::<TextedInfo>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::NoFormatVersionField)
            },
            Severity::Info
        )]
    );
}

#[test]
fn a_missing_version_is_rejected_with_the_family_s_text() {
    let code = the_only_error(&load_with::<StrictTexted>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
        &LoadContext::detached(),
    ));
    assert_eq!(
        code,
        DiagnosticCode::MissingFormatVersion {
            text: Some(VanillaText::NoFormatVersionField)
        }
    );
}

#[test]
fn the_raw_path_reads_the_root_struct_from_the_whole_text() {
    let outcome = load_with::<Raw>(
        r#"{"format_version": "1.0.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}, "third": 3}"#,
        &LoadContext::detached(),
    );
    let Some(Loaded::Stable(root)) = &outcome.value else {
        panic!("loaded: {:?}", outcome.diagnostics);
    };
    assert_eq!(root.toy.description.identifier, "demo:x");
    assert!(
        outcome
            .diagnostics
            .unknown()
            .get(&bedrock_addon_core::diagnostic::JsonPath::new(), "third")
            .is_some(),
        "the third member is the root's own"
    );
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning
        )]
    );
}

#[test]
fn the_envelope_is_checked_from_the_family_minimum_on() {
    let three = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}, "third": 3}}"#
        )
    };
    let below = load_with::<RawBelow>(&three("1.0.0"), &LoadContext::detached());
    assert!(
        below.value.is_some(),
        "raw below the minimum: {:?}",
        below.diagnostics
    );
    let code = the_only_error(&load_with::<RawBelow>(
        &three("1.2.0"),
        &LoadContext::detached(),
    ));
    assert!(
        matches!(code, DiagnosticCode::MalformedInput { .. }),
        "{code:?}"
    );
}

// `select` sorts `X-beta.n` below `X`, and so does the envelope
// minimum: a pre-release of the minimum is read on the raw path, the release is not.
#[test]
fn a_prerelease_of_the_envelope_minimum_is_below_the_minimum() {
    let three = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}, "third": 3}}"#
        )
    };
    let prerelease = load_with::<RawBelow>(&three("1.1.0-beta.1"), &LoadContext::detached());
    assert!(
        prerelease.value.is_some(),
        "a pre-release of the minimum is raw: {:?}",
        prerelease.diagnostics
    );
    // The minimum itself and a pre-release of a later version are checked.
    for version in ["1.1.0", "1.2.0-beta.1"] {
        let code = the_only_error(&load_with::<RawBelow>(
            &three(version),
            &LoadContext::detached(),
        ));
        assert!(
            matches!(code, DiagnosticCode::MalformedInput { .. }),
            "{version}: {code:?}"
        );
    }
}

#[test]
fn file_version_reads_the_envelope_without_loading() {
    assert_eq!(
        file_version(JsonSlice::new(OK_1_2_0)),
        Some(FormatVersion::new(1, 2, 0))
    );
    assert_eq!(
        file_version(JsonSlice::new(r#"{"minecraft:toy": {}}"#)),
        None
    );
    assert_eq!(
        file_version(JsonSlice::new(r#"{"format_version": "beta"}"#)),
        None
    );
    assert_eq!(file_version(JsonSlice::new("[")), None);
}

// ---- content after the root value ----

/// An integer too large for 64 bits is the double it spells, whatever its size: the
/// versioned load reads the file, a float member takes the double, and a member the struct
/// does not read keeps a number.
#[test]
fn a_document_with_integers_beyond_64_bits_loads() {
    for digits in [20, 39, 40, 400] {
        let big = "9".repeat(digits);
        let text = format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {{"toy:speed": {{"value": {big}}}}}, "unread": [-{big}]}}}}"#
        );
        let outcome = load_toy(&text);
        assert!(
            matches!(outcome.value, Some(Loaded::Stable(_))),
            "{digits} digits: {:?}",
            outcome.diagnostics
        );
        assert!(
            outcome
                .diagnostics
                .iter()
                .all(|d| d.severity < Severity::Error),
            "{digits} digits: {:?}",
            outcome.diagnostics
        );
        let shown = format!("{:?}", outcome.value);
        assert!(!shown.contains(&format!("\"{big}\"")), "{shown}");
        assert!(!shown.contains(&format!("\"-{big}\"")), "{shown}");
        // A float member holds the nearest `f32`; beyond it the largest finite one, with the
        // information of a clamped number on the token.
        let speed: f64 = big.parse().unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let nearest = speed as f32;
        let speed = if nearest.is_infinite() {
            f32::MAX
        } else {
            nearest
        };
        assert!(shown.contains(&format!("value: {speed:?}")), "{shown}");
        let clamps: Vec<_> = outcome
            .diagnostics
            .iter()
            .filter(|d| d.code.name() == "InvalidValue")
            .collect();
        assert_eq!(
            clamps.len(),
            usize::from(nearest.is_infinite()),
            "{clamps:?}"
        );
        if let Some(clamp) = clamps.first() {
            assert_eq!(clamp.severity, Severity::Info);
            assert_eq!(
                clamp.location.json_path.to_string(),
                "minecraft:toy/components/toy:speed/value"
            );
            let span = clamp.location.span.expect("on the token");
            assert_eq!(
                &text[span.offset as usize..(span.offset + span.len) as usize],
                big
            );
        }
    }
}

/// The game's jsoncpp reader stops at the end of the root value: the document loads as if nothing
/// followed, with one information of the crate's own on what follows, whatever that is.
#[test]
fn content_after_the_root_value_loads_with_one_information_at_its_span() {
    for tail in ["garbage", "é", "{}", "}", r#""\é""#, "/* c */ x // y"] {
        let text = format!("{OK_1_2_0}\n {tail} \n");
        let outcome = load_toy(&text);
        assert!(
            matches!(outcome.value, Some(Loaded::Stable(_))),
            "`{tail}`: {:?}",
            outcome.diagnostics
        );
        let found: Vec<_> = outcome.diagnostics.iter().collect();
        assert_eq!(found.len(), 1, "`{tail}`: {found:?}");
        assert!(
            matches!(
                &found[0].code,
                DiagnosticCode::MalformedInput { message, text: None }
                    if message == "content after the JSON value is ignored"
            ),
            "{:?}",
            found[0].code
        );
        assert_eq!(found[0].severity, Severity::Info);
        assert!(found[0].location.json_path.is_empty());
        let span = found[0].location.span.expect("a span");
        let content = tail.trim_start_matches("/* c */ ");
        assert_eq!(
            &text[span.offset as usize..(span.offset + span.len) as usize],
            content
        );
    }
}

/// White space and comments after the root are no content; a document that fails for another
/// reason is not also warned about, unless its JSON is whole.
#[test]
fn comments_after_the_root_value_are_not_content() {
    let outcome = load_toy(&format!("{OK_1_2_0}\n// done\n/* really */\n"));
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let cut = load_toy(r#"{"format_version": "1.2.0", "minecraft:toy": {"#);
    assert_eq!(cut.diagnostics.len(), 1, "{:?}", cut.diagnostics);
    let third = load_toy(r#"{"format_version": "1.2.0", "minecraft:toy": 1} x"#);
    assert!(third.value.is_none());
    assert_eq!(
        third
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count(),
        1,
        "{:?}",
        third.diagnostics
    );
}

/// A `JsonCpp` family reads content after the root as the toy does: one information.
#[test]
fn content_after_the_root_value_of_a_json_cpp_family_is_an_information() {
    let outcome = load_with::<JsonCppToy>(&format!("{OK_1_2_0} x"), &LoadContext::detached());
    assert!(matches!(outcome.value, Some(Loaded::Stable(_))));
    assert_eq!(
        codes(&outcome)
            .iter()
            .map(|(_, severity)| *severity)
            .collect::<Vec<_>>(),
        [Severity::Info]
    );
}

/// A Minecraft Document is parsed with `RapidJSON` (iterative, with comments and trailing commas,
/// and without stopping once the root is done), so content after the root is a parse error
/// and the document is refused with "Failed to parse the Minecraft Document JSON: The
/// document root must not be followed by other values."
#[test]
fn content_after_the_root_value_refuses_a_minecraft_document() {
    for tail in ["garbage", "{}", "/* c */ x"] {
        let text = format!("{OK_1_2_0}\n {tail} \n");
        let outcome = load_with::<Document>(&text, &LoadContext::detached());
        let code = the_only_error(&outcome);
        assert!(
            matches!(
                &code,
                DiagnosticCode::MalformedInput {
                    text: Some(VanillaText::DocumentJson { message }),
                    ..
                } if message == "The document root must not be followed by other values."
            ),
            "`{tail}`: {code:?}"
        );
        let error = outcome
            .diagnostics
            .iter()
            .find(|d| d.severity == Severity::Error)
            .unwrap();
        assert!(!error.kept, "the document is refused");
        let span = error.location.span.expect("a span");
        let content = tail.trim_start_matches("/* c */ ");
        assert_eq!(
            &text[span.offset as usize..(span.offset + span.len) as usize],
            content
        );
    }
    // From an MCB file there is no JSON text to look past.
    let mut mcb = LoadContext::detached();
    mcb.source = DocumentSource::Mcb;
    let binary = load_with::<Document>(&format!("{OK_1_2_0} x"), &mcb);
    assert!(binary.value.is_some(), "{:?}", binary.diagnostics);
}

/// `RapidJSON` refuses a number no double holds, so a Minecraft Document holding one is
/// refused with "Failed to parse the Minecraft Document JSON: Number too big to be stored in
/// double." on the number.
#[test]
fn a_number_no_double_holds_refuses_a_minecraft_document() {
    for big in ["1e999".to_owned(), "-1E400".to_owned(), "9".repeat(400)] {
        let text = format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "unread": [{big}]}}}}"#
        );
        let outcome = load_with::<Document>(&text, &LoadContext::detached());
        let code = the_only_error(&outcome);
        assert!(
            matches!(
                &code,
                DiagnosticCode::MalformedInput {
                    text: Some(VanillaText::DocumentJson { message }),
                    ..
                } if message == "Number too big to be stored in double."
            ),
            "`{big}`: {code:?}"
        );
        let error = outcome
            .diagnostics
            .iter()
            .find(|d| d.severity == Severity::Error)
            .unwrap();
        let span = error.location.span.expect("a span");
        assert_eq!(
            &text[span.offset as usize..(span.offset + span.len) as usize],
            big
        );
    }
}

/// The span is a file offset when the document is a slice of a larger text.
#[test]
fn the_warning_s_span_is_rebased_to_the_file() {
    let file = format!("    {OK_1_2_0} tail");
    let outcome = load::<Toy>(
        JsonSlice {
            text: &file[4..],
            offset: 4,
        },
        &LoadContext::detached(),
    );
    let span = outcome.diagnostics.iter().next().unwrap().location.span;
    assert_eq!(span.map(|s| s.offset as usize), file.find("tail"));
}

/// A file that starts with a byte-order mark: read as `json::blank_bom` gives it, every span
/// is an offset into the file as written.
#[test]
fn the_spans_of_a_file_with_a_byte_order_mark_are_of_the_file_as_written() {
    let text = r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}, "components": {"toy:speed": {"value": -1e39}}}}"#;
    let file = [&b"\xEF\xBB\xBF"[..], text.as_bytes(), b" tail"].concat();
    let read = bedrock_addon_core::json::blank_bom(&file);
    let read = std::str::from_utf8(&read).expect("UTF-8");
    let parsed = bedrock_addon_core::versioning::parse_as_written::<Toy>(
        JsonSlice::new(read),
        &LoadContext::detached(),
    );
    assert!(parsed.value.is_some(), "{:?}", parsed.diagnostics);
    let under: Vec<(&str, &[u8])> = parsed
        .diagnostics
        .iter()
        .filter_map(|finding| {
            let span = finding.location.span?;
            let start = span.offset as usize;
            Some((finding.code.name(), &file[start..start + span.len as usize]))
        })
        .collect();
    assert!(
        under.contains(&("InvalidValue", &b"-1e39"[..])),
        "{under:?}"
    );
    assert!(
        under.contains(&("MalformedInput", &b"tail"[..])),
        "{under:?}"
    );
}

// ---- versions and the beta literal ----

const BETA: &str = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": true}}"#;

/// A beta document in a pack that is not a base-game pack fails without the Beta APIs toggle,
/// with the game's own text.
#[test]
fn a_beta_document_needs_the_beta_apis_toggle_or_a_base_game_pack() {
    let outcome = load_toy(BETA);
    assert_eq!(outcome.diagnostics.len(), 1, "{:?}", outcome.diagnostics);
    let code = the_only_error(&outcome);
    assert_eq!(
        code,
        DiagnosticCode::BetaFeaturesNotEnabled {
            text: Some(VanillaText::BetaFormatVersion)
        }
    );
    assert_eq!(
        code.to_string(),
        "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled."
    );

    let mut toggle = LoadContext::detached();
    toggle.beta_apis = true;
    let mut base_game = LoadContext::detached();
    base_game.is_base_game_pack = true;
    for cx in [toggle, base_game] {
        let outcome = load_with::<Toy>(BETA, &cx);
        let Some(Loaded::Beta(beta)) = outcome.value else {
            panic!("a beta value: {:?}", outcome.diagnostics);
        };
        assert_eq!(beta.beta_only, Some(true));
        assert!(outcome.version.beta);
        assert_eq!(outcome.version.parsed_as, BETA_VERSION);
        assert!(!outcome.version.upgraded);
    }
}

#[test]
fn a_family_that_always_permits_beta_loads_it_without_the_toggle() {
    let outcome = load_with::<Open>(BETA, &LoadContext::detached());
    assert!(
        matches!(outcome.value, Some(Loaded::Beta(_))),
        "{:?}",
        outcome.diagnostics
    );
}

/// The slice is selected before the permission is checked, so a
/// family without a beta struct says "no parser available for version beta" with or without
/// the toggle.
#[test]
fn a_beta_document_without_a_beta_struct_is_unsupported_whatever_the_permission() {
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    for cx in [cx, LoadContext::detached()] {
        let code = the_only_error(&load_with::<Stable>(BETA, &cx));
        assert_eq!(
            code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: "beta".to_owned()
                })
            }
        );
    }
}

/// `"9999.9999.9999-beta"` is the beta sentinel: it selects the beta
/// struct, under the beta permission.
#[test]
fn the_spelled_out_sentinel_is_a_beta_document() {
    let text = BETA.replace(r#""beta""#, r#""9999.9999.9999-beta""#);
    let mut toggle = LoadContext::detached();
    toggle.beta_apis = true;
    let outcome = load_with::<Toy>(&text, &toggle);
    assert!(
        matches!(outcome.value, Some(Loaded::Beta(_))),
        "{:?}",
        outcome.diagnostics
    );
    assert!(outcome.version.beta);
    assert!(matches!(
        the_only_error(&load_toy(&text)),
        DiagnosticCode::BetaFeaturesNotEnabled { .. }
    ));
    let code = the_only_error(&load_with::<Stable>(&text, &toggle));
    assert_eq!(
        code.to_string(),
        "unsupported version: no parser available for version beta"
    );
}

/// The game holds the JSON item path's gate too, the toggle alone, with the
/// text "Turn on the Beta APIs toggle in settings to allow beta documents."; a base-game pack
/// does not pass it.
#[test]
fn a_toggle_only_family_refuses_a_base_game_pack_with_the_beta_documents_text() {
    let mut base_game = LoadContext::detached();
    base_game.is_base_game_pack = true;
    let code = the_only_error(&load_with::<ToggleOnly>(BETA, &base_game));
    assert_eq!(
        code,
        DiagnosticCode::BetaFeaturesNotEnabled {
            text: Some(VanillaText::BetaDocuments)
        }
    );
    let mut toggle = LoadContext::detached();
    toggle.beta_apis = true;
    let outcome = load_with::<ToggleOnly>(BETA, &toggle);
    assert!(
        matches!(outcome.value, Some(Loaded::Beta(_))),
        "{:?}",
        outcome.diagnostics
    );
}

/// A `-beta.N` pre-release is an ordinary version: no toggle, the struct below its release,
/// a `PrereleaseFormatVersion` info.
#[test]
fn a_prerelease_version_loads_by_the_struct_below_with_an_info() {
    let outcome = load_toy(
        r#"{"format_version": "1.1.0-beta.2", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::PrereleaseFormatVersion {
                version: "1.1.0-beta.2".to_owned()
            },
            Severity::Info
        )]
    );
    assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 0, 0));
    assert!(!outcome.version.beta);
}

#[test]
fn below_the_floor_is_unsupported_naming_the_identifier_and_the_floor() {
    let code = the_only_error(&load_toy(
        r#"{"format_version": "0.9.0", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
    ));
    let DiagnosticCode::UnsupportedVersion {
        text:
            Some(VanillaText::UseFormatVersion {
                what,
                name,
                version,
            }),
    } = code
    else {
        panic!("{code:?}");
    };
    assert_eq!(
        (what, name.as_str(), version),
        ("toy", "demo:x", FormatVersion::new(1, 0, 0))
    );
}

/// An open-ended family takes every version up to the engine version.
#[test]
fn a_document_between_1_26_51_and_the_engine_version_loads() {
    for version in ["1.26.51", "1.26.55", "1.26.60"] {
        let outcome = load_toy(&format!(
            r#"{{"format_version": "{version}", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}}}"#
        ));
        assert!(
            matches!(outcome.value, Some(Loaded::Stable(_))),
            "{version}: {:?}",
            outcome.diagnostics
        );
        assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 2, 0));
    }
}

#[test]
fn above_the_engine_is_unsupported_with_the_puv_text() {
    let code = the_only_error(&load_toy(
        r#"{"format_version": "1.26.61", "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#,
    ));
    assert_eq!(
        code,
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::NoParserForVersion {
                version: "1.26.61".to_owned()
            })
        }
    );
}

/// The game's text prints the version as the input parsed it: a partial version gets its
/// zeros (`1.27` → `1.27.0`, as the game prints `1.26.0` for `1.26`), build metadata is
/// dropped; a pre-release keeps its text.
#[test]
fn the_no_parser_text_prints_the_version_as_parsed() {
    for (written, shown) in [
        (r#""1.27""#, "1.27.0"),
        (r#""1.27.0+build.5""#, "1.27.0"),
        (r#""1.27.0""#, "1.27.0"),
        (r#""1.27.0-rc.1""#, "1.27.0-rc.1"),
    ] {
        let text = format!(
            r#"{{"format_version": {written}, "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}}}"#
        );
        assert_eq!(
            the_only_error(&load_toy(&text)),
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: shown.to_owned()
                })
            },
            "{written}"
        );
    }
}

#[test]
fn a_failed_typed_parse_rejects_the_document_at_the_member() {
    let outcome = load_toy(
        r#"{"format_version": "1.0.0", "minecraft:toy": {"description": {"identifier": "demo:x"}, "old_speed": "fast"}}"#,
    );
    assert!(outcome.value.is_none());
    let error = outcome.diagnostics.first_error().expect("an error");
    assert!(
        matches!(error.code, DiagnosticCode::InvalidType { .. }),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:toy/old_speed"
    );
    let span = error.location.span.expect("a span");
    assert!(
        span.offset > 30,
        "a file offset inside the payload: {span:?}"
    );
}

#[test]
fn an_unknown_payload_member_is_kept_and_reported_below_the_payload_key() {
    let outcome = load_toy(
        r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x", "note": 1}, "extra_member": true}}"#,
    );
    let Some(Loaded::Stable(_)) = outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    let unknown = outcome.diagnostics.unknown();
    let payload = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:toy"]);
    assert!(unknown.get(&payload, "extra_member").is_some());
    assert!(unknown.get(&payload.child("description"), "note").is_some());
    let paths: Vec<String> = outcome
        .diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "minecraft:toy/description/note",
            "minecraft:toy/extra_member"
        ]
    );
}

// ---- the error mask of a slice ----

/// Under the game's unstrict error mask an unknown member or component is a warning and the
/// document is kept.
#[test]
fn an_unknown_member_is_a_warning_in_an_unstrict_slice() {
    let text = r#"{"format_version": "1.0.0", "minecraft:toy": {"description": {"identifier": "demo:x", "is_spawnable": true}}}"#;
    let outcome = load_with::<Masked>(text, &LoadContext::detached());
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(
        codes(&outcome),
        [(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning
        )]
    );
}

/// Under the strict mask the unknown member
/// and the unknown component are errors and the document is rejected; a family
/// that sets no mask keeps the warning.
#[test]
fn an_unknown_member_or_component_rejects_a_strict_slice() {
    for (payload, code) in [
        (
            r#"{"description": {"identifier": "demo:x"}, "extra_member": true}"#,
            DiagnosticCode::UnrecognizedField { text: None },
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:foot_size": {}}}"#,
            // The warning carries the game's own text for a member the schema does not list.
            bedrock_addon_core::diagnostic::cereal::component_not_in_schema(),
        ),
    ] {
        let text = format!(r#"{{"format_version": "1.2.0", "minecraft:toy": {payload}}}"#);
        let strict = load_with::<Masked>(&text, &LoadContext::detached());
        assert_eq!(the_only_error(&strict), code, "{payload}");
        let default = load_toy(&text);
        assert!(default.value.is_some(), "{:?}", default.diagnostics);
        assert_eq!(codes(&default), [(code, Severity::Warning)], "{payload}");
    }
}

/// A root member outside the payload is no member of the struct: not an unknown member, so the
/// strict mask leaves its finding an information of the crate's own (a document without a
/// version).
#[test]
fn a_root_member_outside_the_payload_stays_an_information_in_a_strict_slice() {
    let outcome = load_with::<Masked>(
        r#"{"minecraft:toy": {"description": {"identifier": "demo:x"}}, "third": 3}"#,
        &LoadContext::detached(),
    );
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    let unknown: Vec<_> = codes(&outcome)
        .into_iter()
        .filter(|(code, _)| matches!(code, DiagnosticCode::UnrecognizedField { .. }))
        .collect();
    assert_eq!(
        unknown,
        [(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Info
        )]
    );
}

// ---- experiment-gated fields of a beta struct ----

/// A beta struct with one experiment-gated field, as the game's beta loader binds it: the
/// test component's `experimental_flag` needs the test experiment.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct GatedBeta {
    pub description: crate::toy::Description,
    #[facet(default)]
    pub components: Option<Value>,
    #[facet(skip)]
    pub extra: Extra,
}

impl Versioned for GatedBeta {
    const VERSION: FormatVersion = BETA_VERSION;
}

const GATED: &[ExperimentalField] = &[ExperimentalField {
    name: "components.puv_beta_load_tests:test_component.experimental_flag",
    experiment: Experiment::TestExperiment,
}];

impl VersionStruct for GatedBeta {
    fn parse(
        payload: JsonSlice<'_>,
        cx: &ParseCx<'_>,
        d: &mut bedrock_addon_core::diagnostic::Diagnostics,
    ) -> Option<Self> {
        let parsed: Self = parse_payload(payload, cx, d)?;
        let present = |name: &str| {
            let mut parts = name.split('.');
            let Some("components") = parts.next() else {
                return false;
            };
            let mut value = parsed.components.as_ref();
            for part in parts {
                value = value.and_then(|v| v.as_object()?.get(part));
            }
            value.is_some()
        };
        check_experimental_fields(GATED, present, "minecraft:toy", &cx.at, cx.load, d)
            .then_some(parsed)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl bedrock_addon_core::molang::MolangFields for GatedBeta {
    fn visit_molang(&mut self, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}
}

pub struct Gated;
pub mod gated {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Gated;
        /// The toy's chain with a gated beta struct.
        pub enum Any {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(GatedBeta);
    }
}
impl Family for Gated {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Toy120;
    type Beta = GatedBeta;
    type Component = ToyComponent;
    type Any = gated::Any;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
}

/// A non-base pack with the Beta APIs loads the beta document with the experimental field
/// only when the experiment is on; without it the load fails with an entry that
/// names the experiment. Without the field the experiment is not needed.
#[test]
fn an_experimental_beta_field_loads_only_with_its_experiment() {
    let document = |experimental: bool| {
        let flag = if experimental {
            r#", "experimental_flag": true"#
        } else {
            ""
        };
        format!(
            r#"{{"format_version": "beta", "minecraft:toy": {{"description": {{"identifier": "puv_beta_load_tests:test_actor"}}, "components": {{"puv_beta_load_tests:test_component": {{"flag": true{flag}}}}}}}}}"#
        )
    };
    let mut without = LoadContext::detached();
    without.beta_apis = true;
    let refused = load_with::<Gated>(&document(true), &without);
    let code = the_only_error(&refused);
    assert!(
        matches!(code, DiagnosticCode::MissingToggles { .. }),
        "{code:?}"
    );
    assert!(
        code.to_string().contains("test_experiment"),
        "the entry names the experiment: {code}"
    );
    let error = refused.diagnostics.first_error().expect("the error");
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:toy/components/puv_beta_load_tests:test_component/experimental_flag"
    );
    assert!(
        load_with::<Gated>(&document(false), &without)
            .value
            .is_some()
    );
    let mut with = without.clone();
    with.experiments.insert(Experiment::TestExperiment);
    let loaded = load_with::<Gated>(&document(true), &with);
    assert!(
        matches!(loaded.value, Some(Loaded::Beta(_))),
        "{:?}",
        loaded.diagnostics
    );
}

// ---- the input kind's null rule ----

/// A `MinecraftDocumentInput` family rejects a payload member written as `null` (N2); a `JsonCpp`
/// family (N1) reads it as absent;
/// the first run's kind keeps facet's reading (`None`, silently).
#[test]
fn a_null_payload_member_follows_the_family_s_input_kind() {
    let text = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": null}}"#;
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    let rejected = load_with::<Document>(text, &cx);
    assert_eq!(
        the_only_error(&rejected),
        DiagnosticCode::InvalidType { text: None }
    );
    assert_eq!(
        rejected
            .diagnostics
            .first_error()
            .map(|e| e.location.json_path.to_string()),
        Some("minecraft:toy/beta_only".to_owned())
    );
    let absent = load_with::<JsonCppToy>(text, &cx);
    let first_run = load_with::<Toy>(text, &cx);
    for (value, diagnostics, told) in [
        (
            absent
                .value
                .map(|v| matches!(v, Loaded::Beta(b) if b.beta_only.is_none())),
            absent.diagnostics,
            // The member the write-back writes as its default is marked.
            vec!["minecraft:toy/beta_only".to_owned()],
        ),
        (
            first_run
                .value
                .map(|v| matches!(v, Loaded::Beta(b) if b.beta_only.is_none())),
            first_run.diagnostics,
            Vec::new(),
        ),
    ] {
        assert_eq!(value, Some(true), "{diagnostics:?}");
        assert_eq!(read_as_absent(&diagnostics), told, "{diagnostics:?}");
    }
}

/// The paths of `diagnostics`, which must all be the information that a `null` was read as absent
/// (N1).
fn read_as_absent(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| {
            assert!(
                d.severity == Severity::Info
                    && matches!(d.code, DiagnosticCode::ValueRewritten { .. }),
                "{d:?}"
            );
            d.location.json_path.to_string()
        })
        .collect()
}

/// The same through `parse_component`: a `null` optional member of a component rejects the
/// document of a `MinecraftDocumentInput` family; a bare member written as `null` is absent
/// for a `JsonCpp` family (it takes the struct's default), where the first run's kind refuses
/// the `null` as a value of the wrong type.
#[test]
fn a_null_component_member_follows_the_family_s_input_kind() {
    let doc = |component: &str| {
        format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {{{component}}}}}}}"#
        )
    };
    let light = doc(r#""toy:light": {"condition": null}"#);
    let rejected = load_with::<Document>(&light, &LoadContext::detached());
    assert_eq!(
        the_only_error(&rejected),
        DiagnosticCode::InvalidType { text: None }
    );
    let at = rejected.diagnostics.first_error().unwrap();
    assert_eq!(
        at.location.json_path.to_string(),
        "minecraft:toy/components/toy:light/condition"
    );
    let absent = load_with::<JsonCppToy>(&light, &LoadContext::detached());
    assert!(absent.value.is_some(), "{:?}", absent.diagnostics);
    assert_eq!(
        read_as_absent(&absent.diagnostics),
        ["minecraft:toy/components/toy:light/condition"]
    );

    let glow = doc(r#""toy:glow": {"level": null}"#);
    let json_cpp = load_with::<JsonCppToy>(&glow, &LoadContext::detached());
    let first_run = load_toy(&glow);
    // Absent, the member takes the struct's default: only the N1 information.
    assert!(json_cpp.value.is_some(), "{:?}", json_cpp.diagnostics);
    assert_eq!(
        read_as_absent(&json_cpp.diagnostics),
        ["minecraft:toy/components/toy:glow/level"]
    );
    assert!(
        first_run.diagnostics.iter().any(|d| d
            .location
            .json_path
            .to_string()
            .starts_with("minecraft:toy/components/toy:glow")),
        "the first run's kind refuses the null: {:?}",
        first_run.diagnostics
    );
}

/// The null rule is the selected slice's. A family whose file is one input kind but
/// whose slices read `null` apart (the item: N1 below 1.26.30, N2 from it) says so per slice,
/// as it says its error mask.
#[test]
fn the_null_rule_is_chosen_after_the_slice() {
    let beta = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": null}}"#;
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    assert_eq!(
        the_only_error(&load_with::<Mixed>(beta, &cx)),
        DiagnosticCode::InvalidType { text: None },
        "the beta slice is N2"
    );
    let glow = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {{"toy:light": {{"condition": null}}}}}}}}"#
        )
    };
    let modern = load_with::<Mixed>(&glow("1.2.0"), &LoadContext::detached());
    assert_eq!(
        the_only_error(&modern),
        DiagnosticCode::InvalidType { text: None },
        "1.2.0 is N2"
    );
    // Below the modern slices the same family reads a `null` as absent; a family whose every
    // slice is N2 refuses it there too.
    let legacy = r#"{"format_version": "1.0.0", "minecraft:toy": {"description": {"identifier": "demo:x"}, "old_speed": null}}"#;
    let absent = load_with::<Mixed>(legacy, &LoadContext::detached());
    assert!(
        absent.value.is_some(),
        "1.0.0 is N1: {:?}",
        absent.diagnostics
    );
    assert_eq!(
        the_only_error(&load_with::<Document>(legacy, &LoadContext::detached())),
        DiagnosticCode::InvalidType { text: None }
    );
}

/// A rule a context already carries does not leak into a family of the first run's kind:
/// the load sets the slice's rule, `None` included.
#[test]
fn a_first_run_family_ignores_a_rule_the_context_carries() {
    let text = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": null}}"#;
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    cx.null_rule = Some(NullRule::Rejected);
    let outcome = load_with::<Toy>(text, &cx);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
}

// ---- the source of the bytes ----

/// The stack marks a document that was a binary `\x7FMCB` one. The item beta
/// split reads it: a base-game pack without the toggle loads a beta MCB document (the game's own
/// gate) and not a beta JSON one (the toggle alone).
#[test]
fn a_family_chooses_its_beta_gate_by_the_source_of_the_bytes() {
    let mut json = LoadContext::detached();
    json.is_base_game_pack = true;
    assert_eq!(
        the_only_error(&load_with::<Sourced>(BETA, &json)),
        DiagnosticCode::BetaFeaturesNotEnabled {
            text: Some(VanillaText::BetaDocuments)
        }
    );
    let mut mcb = json.clone();
    mcb.source = DocumentSource::Mcb;
    let outcome = load_with::<Sourced>(BETA, &mcb);
    assert!(
        matches!(outcome.value, Some(Loaded::Beta(_))),
        "{:?}",
        outcome.diagnostics
    );
}

/// An MCB source is read by the binary reader (kind 0): no null rule and no JSON size check.
#[test]
fn an_mcb_source_has_no_null_rule_and_no_size_check() {
    let text = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": null}}"#;
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    cx.max_json_document_bytes = 40;
    let mut mcb = cx.clone();
    mcb.source = DocumentSource::Mcb;
    let outcome = load_with::<Document>(text, &mcb);
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    // The same text as JSON is refused for its size before its `null` is read.
    let code = the_only_error(&load_with::<Document>(text, &cx));
    assert!(
        matches!(
            code,
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentTooLarge { max: 40 }),
                ..
            }
        ),
        "{code:?}"
    );
}

/// With the limit lowered on the context: a document
/// above it is refused with exactly the too-large entry; from three quarters of it a size
/// warning is given and the document loads; a family of the root reading has no size check.
#[test]
fn a_document_input_refuses_a_text_above_the_limit_and_warns_near_it() {
    let len = OK_1_2_0.len();
    let mut cx = LoadContext::detached();
    cx.max_json_document_bytes = len - 1;
    let refused = load_with::<Document>(OK_1_2_0, &cx);
    assert!(refused.value.is_none());
    assert_eq!(refused.diagnostics.len(), 1, "{:?}", refused.diagnostics);
    assert_eq!(
        refused.diagnostics.iter().next().unwrap().code.to_string(),
        format!(
            "Minecraft Document JSON exceeds the maximum size of {} bytes",
            len - 1
        )
    );
    cx.max_json_document_bytes = len;
    let near = load_with::<Document>(OK_1_2_0, &cx);
    assert!(near.value.is_some(), "{:?}", near.diagnostics);
    assert_eq!(
        codes(&near)
            .into_iter()
            .map(|(code, severity)| (code.name(), severity))
            .collect::<Vec<_>>(),
        [("MalformedInput", Severity::Warning)]
    );
    cx.max_json_document_bytes = len - 1;
    assert!(load_toy_with(OK_1_2_0, &cx).value.is_some());
}

fn load_toy_with(text: &str, cx: &LoadContext) -> LoadOutcome<Toy> {
    load_with::<Toy>(text, cx)
}

/// In a cereal slice of a `MinecraftDocumentInput` family a component whose typed read fails
/// or whose required member is missing rejects the whole document under both masks; a `JsonCpp`
/// family does the same, as the game's JSON schema readers fail it the same way.
#[test]
fn a_component_that_fails_its_typed_read_rejects_a_document_input_under_both_masks() {
    let doc = |component: &str| {
        format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {{"toy:speed": {{"value": 1.0}}, {component}}}}}}}"#
        )
    };
    let wrong_type = doc(r#""toy:glow": {"level": "bright"}"#);
    let missing = doc(r#""toy:needs": {}"#);
    for (text, code, at) in [
        (
            &wrong_type,
            "InvalidType",
            "minecraft:toy/components/toy:glow/level",
        ),
        (
            &missing,
            "MissingRequiredField",
            "minecraft:toy/components/toy:needs/amount",
        ),
    ] {
        for (mask, outcome) in [
            (
                "unstrict",
                codes_at(&load_with::<Document>(text, &LoadContext::detached())),
            ),
            (
                "strict",
                codes_at(&load_with::<DocumentStrict>(text, &LoadContext::detached())),
            ),
        ] {
            assert!(
                !outcome.0,
                "{mask}: the document is rejected: {:?}",
                outcome.1
            );
            assert_eq!(
                outcome.1,
                [(code.to_owned(), Severity::Error, at.to_owned())],
                "{mask}"
            );
        }
        let json_cpp = codes_at(&load_with::<JsonCppToy>(text, &LoadContext::detached()));
        assert!(
            !json_cpp.0,
            "JsonCpp: the document is rejected: {:?}",
            json_cpp.1
        );
        assert!(
            json_cpp
                .1
                .iter()
                .any(|(_, severity, _)| *severity == Severity::Error),
            "JsonCpp: an error: {:?}",
            json_cpp.1
        );
    }
}

/// Whether the document loaded, and every finding as (code name, severity, path).
fn codes_at<F: Family>(outcome: &LoadOutcome<F>) -> (bool, Vec<(String, Severity, String)>) {
    let found = outcome
        .diagnostics
        .iter()
        .map(|d| {
            let name: String = format!("{:?}", d.code)
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            (name, d.severity, d.location.json_path.to_string())
        })
        .collect();
    (outcome.value.is_some(), found)
}

/// A kept error is an error-level finding (the gate counts it) that keeps the
/// document; a plain error beside it still rejects.
#[test]
fn a_kept_error_loads_the_document_and_a_plain_error_still_rejects() {
    let doc = |components: &str| {
        format!(
            r#"{{"format_version": "1.2.0", "minecraft:toy": {{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}}}"#
        )
    };
    let kept = load_with::<Document>(&doc(r#""toy:kept": {}"#), &LoadContext::detached());
    assert!(
        kept.value.is_some(),
        "the document is kept: {:?}",
        kept.diagnostics
    );
    let errors: Vec<_> = kept
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{:?}", kept.diagnostics);
    assert!(errors[0].kept, "the error is marked kept");
    assert_eq!(
        errors[0].location.json_path.to_string(),
        "minecraft:toy/components/toy:kept"
    );
    assert!(
        !kept.diagnostics.has_errors(),
        "a kept error rejects nothing"
    );
    let rejected = load_with::<Document>(
        &doc(r#""toy:kept": {}, "toy:glow": {"level": "bright"}"#),
        &LoadContext::detached(),
    );
    assert!(rejected.value.is_none(), "{:?}", rejected.diagnostics);
}

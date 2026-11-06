//! `Family::reading` and `Family::ENVELOPE`'s text reading: the per-slice reading and the
//! family's text reading, over toy families that cover every combination the families use,
//! pinned in one reading table. It covers what the pipeline reads by, not the local overrides a
//! family makes in its readers (block's and item's forced strict components, camera's coerced
//! slice 0, dimension's version-filtered null rule): those are listed on `Family::reading`.

use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::testing::slice_reading;
use bedrock_addon_core::versioning::{
    BeyondWidth, ComponentTable, Envelope, Family, InputKind, Loaded, NullHandling, Reading,
    Scalars, StructIndex, UnknownMask, VersionInput,
};

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::load::LoadContext;

use crate::envelope::{
    Document, DocumentStrict, JsonCppToy, Masked, Mixed, Stable, codes, load_with,
};
use crate::toy::{Toy, Toy120, ToyBeta, ToyComponent, v1_0_0};

/// Toy families for the knob combinations `envelope` has no variant of.
macro_rules! reading_toy {
    ($name:ident, $module:ident, { $($rule:tt)* }) => {
        pub struct $name;
        pub mod $module {
            use super::*;
            bedrock_addon_core::version_chain! {
                family = $name;
                /// The toy's chain for another family.
                pub enum History {
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
            type Beta = ToyBeta;
            type Component = ToyComponent;
            type History = $module::History;
            fn components() -> &'static ComponentTable<Self> {
                &ComponentTable::EMPTY
            }
            $($rule)*
        }
    };
}

// Particle and trade: a hand-written legacy slice (no null rule, unstrict), cereal slices
// after it; particle also clamps.
reading_toy!(LegacyFirst, legacy_first, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        clamps_beyond_double: true,
        ..Envelope::DEFAULT
    };
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::OWN,
            _ => Reading::DOCUMENT,
        }
    }
});

// Block: duplicates kept in `components`, the legacy slice its own, cereal slices strict.
reading_toy!(BlockLike, block_like, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        clamps_beyond_double: true,
        duplicates_kept_in: &["components"],
        ..Envelope::DEFAULT
    };
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(i) if i < 2 => Reading::OWN,
            _ => Reading::DOCUMENT,
        }
    }
});

// Entity: `JsonCpp`, strict from a slice on and for the beta struct.
reading_toy!(EntityLike, entity_like, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::JsonCpp,
        clamps_beyond_double: true,
        ..Envelope::DEFAULT
    };
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(i) if i < 1 => Reading::NULL_ABSENT,
            _ => Reading::NULL_ABSENT.with_unknown(UnknownMask::Error),
        }
    }
});

// Catalog and the camera preset: cereal throughout, the first slice unstrict.
reading_toy!(UnstrictFirst, unstrict_first, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::DOCUMENT.with_unknown(UnknownMask::Warn),
            _ => Reading::DOCUMENT,
        }
    }
});

// A converting family: its first slice keeps the unknown members of its structs.
reading_toy!(KeepsFirst, keeps_first, {
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::OWN.with_keeps_unknown(true),
            _ => Reading::OWN,
        }
    }
});

/// The JSON reading of `F`'s slice `index`.
fn json<F: Family>(index: StructIndex) -> Reading {
    F::reading(index, DocumentSource::Json)
}

#[test]
fn reading_names_each_toy_by_its_profile() {
    let warn = UnknownMask::Warn;
    let error = UnknownMask::Error;
    // `FirstRun`: the family's own reader.
    assert_eq!(json::<Toy>(StructIndex::Stable(0)), Reading::OWN);
    assert_eq!(
        json::<Masked>(StructIndex::Stable(2)),
        Reading::OWN.with_unknown(error)
    );
    // The document input, unstrict unknowns by default (feature rules, dimension, jigsaw).
    assert_eq!(
        json::<Document>(StructIndex::Beta),
        Reading::DOCUMENT.with_unknown(warn)
    );
    assert_eq!(
        json::<DocumentStrict>(StructIndex::Stable(1)),
        Reading::DOCUMENT
    );
    // `JsonCpp`.
    assert_eq!(
        json::<JsonCppToy>(StructIndex::Stable(0)),
        Reading::NULL_ABSENT
    );
    assert_eq!(
        json::<EntityLike>(StructIndex::Beta),
        Reading::NULL_ABSENT.with_unknown(error)
    );
    // The item's split.
    assert_eq!(json::<Mixed>(StructIndex::Stable(0)), Reading::NULL_ABSENT);
    assert_eq!(
        json::<Mixed>(StructIndex::Stable(2)),
        Reading::DOCUMENT.with_unknown(warn)
    );
    // Particle, trade, block: the legacy slice is the family's own.
    assert_eq!(json::<LegacyFirst>(StructIndex::Stable(0)), Reading::OWN);
    assert_eq!(
        json::<LegacyFirst>(StructIndex::Stable(1)),
        Reading::DOCUMENT
    );
    assert_eq!(json::<BlockLike>(StructIndex::Stable(2)), Reading::DOCUMENT);
    assert_eq!(
        json::<UnstrictFirst>(StructIndex::Stable(0)),
        Reading::DOCUMENT.with_unknown(warn)
    );
    // An MCB source has no null rule; its components stay as the slice reads them.
    assert_eq!(
        slice_reading::<DocumentStrict>(StructIndex::Stable(0), DocumentSource::Mcb),
        Reading::DOCUMENT.with_null(NullHandling::Own)
    );
}

#[test]
fn the_envelope_sizes_a_json_text_of_a_document_input_only() {
    assert_eq!(Toy::ENVELOPE.input.version_input(), VersionInput::Root);
    assert!(!Toy::ENVELOPE.sizes_json(DocumentSource::Json));
    assert_eq!(
        Document::ENVELOPE.input.version_input(),
        VersionInput::Document
    );
    assert!(Document::ENVELOPE.sizes_json(DocumentSource::Json));
    assert!(!Document::ENVELOPE.sizes_json(DocumentSource::Mcb));
    assert!(!EntityLike::ENVELOPE.sizes_json(DocumentSource::Json));
}

#[test]
fn a_profile_with_other_scalars_differs_from_own_in_them_alone() {
    let json_cpp = Reading::OWN.with_scalars(Scalars::JsonCpp);
    assert_eq!(json_cpp.scalars, Scalars::JsonCpp);
    assert_eq!(json_cpp.with_scalars(Scalars::ConvertAndTell), Reading::OWN);
    let error = Reading::OWN.with_beyond_width(BeyondWidth::Error);
    assert_eq!(error.beyond_width, BeyondWidth::Error);
    assert_eq!(
        error.with_beyond_width(BeyondWidth::ClampAndTell),
        Reading::OWN
    );
}

#[test]
fn below_floor_defaults_to_the_generic_floor_line() {
    use bedrock_addon_core::versioning::BelowFloor;
    // What `load` raises below the floor today, for every family that does not say otherwise.
    assert_eq!(Toy::ENVELOPE.below_floor, BelowFloor::UseFormatVersion);
    assert_eq!(Document::ENVELOPE.below_floor, BelowFloor::default());
}

// ---- the pipeline reads `reading` and the envelope's text reading ----

// The first run's input; the reading a document input's cereal slice.
reading_toy!(ReadByReading, read_by_reading, {
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        Reading::DOCUMENT
    }
});

// The input a document input's, every slice read as the first run's.
reading_toy!(DocumentByText, document_by_text, {
    const ENVELOPE: Envelope = Envelope {
        input: InputKind::Document,
        ..Envelope::DEFAULT
    };
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        Reading::OWN
    }
});

// The first run's input; the text clamps beyond a double.
reading_toy!(ClampsByText, clamps_by_text, {
    const ENVELOPE: Envelope = Envelope {
        clamps_beyond_double: true,
        ..Envelope::DEFAULT
    };
});

/// How many `InvalidValue` findings an outcome has.
fn invalid_values<F: Family>(outcome: &bedrock_addon_core::versioning::LoadOutcome<F>) -> usize {
    codes(outcome)
        .into_iter()
        .filter(|(code, _)| matches!(code, DiagnosticCode::InvalidValue { .. }))
        .count()
}

fn beta_cx() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    cx
}

#[test]
fn the_slice_s_null_rule_comes_from_reading() {
    let text = r#"{"format_version": "beta", "minecraft:toy": {"description": {"identifier": "demo:x"}, "beta_only": null}}"#;
    let rejected = load_with::<ReadByReading>(text, &beta_cx());
    assert!(rejected.value.is_none(), "{:?}", rejected.diagnostics);
    assert_eq!(
        codes(&rejected),
        [(DiagnosticCode::InvalidType { text: None }, Severity::Error)]
    );
    // The knobs alone read it as the first run does.
    let first_run = load_with::<crate::toy::Toy>(text, &beta_cx());
    assert!(
        matches!(first_run.value, Some(Loaded::Beta(ref b)) if b.beta_only.is_none()),
        "{:?}",
        first_run.diagnostics
    );
}

#[test]
fn the_slice_s_unknown_mask_comes_from_reading() {
    let text = r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}, "extra_member": true}}"#;
    let strict = load_with::<ReadByReading>(text, &LoadContext::detached());
    assert!(strict.value.is_none(), "{:?}", strict.diagnostics);
    assert_eq!(
        codes(&strict),
        [(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Error
        )]
    );
}

#[test]
fn the_version_input_comes_from_text() {
    // The document input reads a number as its text; the root member's reading refuses it.
    let text =
        r#"{"format_version": 1.2, "minecraft:toy": {"description": {"identifier": "demo:x"}}}"#;
    let outcome = load_with::<DocumentByText>(text, &LoadContext::detached());
    assert!(outcome.value.is_some(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.version.parsed_as, FormatVersion::new(1, 2, 0));
    let root = load_with::<crate::toy::Toy>(text, &LoadContext::detached());
    assert!(root.value.is_none(), "{:?}", root.diagnostics);
}

#[test]
fn the_clamp_beyond_a_double_comes_from_text() {
    let text = r#"{"format_version": "1.2.0", "minecraft:toy": {"description": {"identifier": "demo:x"}, "unread": [1e999]}}"#;
    let clamped = load_with::<ClampsByText>(text, &LoadContext::detached());
    let unclamped = load_with::<crate::toy::Toy>(text, &LoadContext::detached());
    assert_eq!(invalid_values(&clamped), 1, "{:?}", clamped.diagnostics);
    assert_eq!(invalid_values(&unclamped), 0, "{:?}", unclamped.diagnostics);
}

// ---- the reading table: what the pipeline reads each toy by ----

/// The reading tables of the toys, in `every_toy_reads_as_pinned`'s order.
const TOY_TABLES: &str = r#"
        Toy
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Stable
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Strict
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: rejected: NoValidVersion
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Quiet
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, silently
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Uncounted
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Uncounted
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Texted
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Warning: Some(NoFormatVersionField)
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        TextedInfo
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: Some(NoFormatVersionField)
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        StrictTexted
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: rejected: NoFormatVersionField
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        Mixed
        text: Document, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.1.0 (0..=1): json Absent/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        1.2.0..=1.2.0 (2..=2): json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
        Masked
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.1.0 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        1.2.0..=1.2.0 (2..=2): json Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse
        Document
        text: Document, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
        JsonCppToy
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Absent/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Absent/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        DocumentStrict
        text: Document, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        LegacyFirst
        text: Document, clamps beyond a double: true, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.0.0 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        1.1.0..=1.2.0 (1..=2): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        BlockLike
        text: Document, clamps beyond a double: true, duplicates kept in: ["components"]
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.1.0 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        1.2.0..=1.2.0 (2..=2): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        EntityLike
        text: Root, clamps beyond a double: true, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.0.0 (0..=0): json Absent/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        1.1.0..=1.2.0 (1..=2): json Absent/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse
        beta: json Absent/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/lenient/Refuse
        UnstrictFirst
        text: Document, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.0.0 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
        1.1.0..=1.2.0 (1..=2): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        ReadByReading
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
        DocumentByText
        text: Document, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        ClampsByText
        text: Root, clamps beyond a double: true, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.2.0 (0..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        KeepsFirst
        text: Root, clamps beyond a double: false, duplicates kept in: []
        envelope: Always
        missing version: at the floor, told at Info: None
        below floor: UseFormatVersion
        empty document: Error
        1.0.0..=1.0.0 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/keeps/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/keeps/lenient/Refuse
        1.1.0..=1.2.0 (1..=2): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
        beta: json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
"#;

/// The reading table of every toy family that states its reading: pinned, so that a change
/// to how a family declares its reading cannot change what the pipeline reads.
#[test]
fn every_toy_reads_as_pinned() {
    use crate::envelope::{Quiet, Strict, StrictTexted, Texted, TextedInfo, Uncounted};
    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};
    let tables = [
        ("Toy", reading_table::<Toy>()),
        ("Stable", reading_table::<Stable>()),
        ("Strict", reading_table::<Strict>()),
        ("Quiet", reading_table::<Quiet>()),
        ("Uncounted", reading_table::<Uncounted>()),
        ("Texted", reading_table::<Texted>()),
        ("TextedInfo", reading_table::<TextedInfo>()),
        ("StrictTexted", reading_table::<StrictTexted>()),
        ("Mixed", reading_table::<Mixed>()),
        ("Masked", reading_table::<Masked>()),
        ("Document", reading_table::<Document>()),
        ("JsonCppToy", reading_table::<JsonCppToy>()),
        ("DocumentStrict", reading_table::<DocumentStrict>()),
        ("LegacyFirst", reading_table::<LegacyFirst>()),
        ("BlockLike", reading_table::<BlockLike>()),
        ("EntityLike", reading_table::<EntityLike>()),
        ("UnstrictFirst", reading_table::<UnstrictFirst>()),
        ("ReadByReading", reading_table::<ReadByReading>()),
        ("DocumentByText", reading_table::<DocumentByText>()),
        ("ClampsByText", reading_table::<ClampsByText>()),
        ("KeepsFirst", reading_table::<KeepsFirst>()),
    ];
    let tables = tables.iter().fold(String::new(), |all, (name, table)| {
        all + name + "\n" + table
    });
    assert_reading_table("the toys", &tables, TOY_TABLES);
}

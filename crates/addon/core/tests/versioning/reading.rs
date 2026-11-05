//! `Family::reading` and `Family::TEXT`: the per-slice reading and the family's text reading,
//! each equal to what the old knobs (`INPUT`, `null_rule`, `error_mask`,
//! `CLAMPS_BEYOND_DOUBLE`, `DUPLICATES_KEPT_IN`) give, over toy families that cover every
//! combination of those knobs the families use. It covers what core derives from the knobs,
//! not the local overrides a family makes outside them (block's and item's forced strict
//! components, camera's coerced slice 0, dimension's version-filtered null rule): those are
//! listed on `Family::reading` and each family states them when it migrates.

use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, ErrorMask, Family, InputKind, MemberFailure, NullHandling, Reading, Scalars,
    StructIndex, UnknownMask, VersionChain, VersionInput,
};

use crate::envelope::{Document, DocumentStrict, JsonCppToy, Masked, Mixed, Stable};
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
            const DIRECTORIES: &'static [&'static str] = &["toys"];
            const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
            type Latest = Toy120;
            type Beta = ToyBeta;
            type Component = ToyComponent;
            type Any = $module::Any;
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
    const INPUT: InputKind = InputKind::Document;
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        match index {
            StructIndex::Stable(0) => None,
            _ => Some(NullRule::Rejected),
        }
    }
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }
});

// Block: duplicates kept in `components`, the legacy slice its own, cereal slices strict.
reading_toy!(BlockLike, block_like, {
    const INPUT: InputKind = InputKind::Document;
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const DUPLICATES_KEPT_IN: &'static [&'static str] = &["components"];
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        match index {
            StructIndex::Stable(i) if i < 2 => None,
            _ => Some(NullRule::Rejected),
        }
    }
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(i) if i < 2 => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }
});

// Entity: `JsonCpp`, strict from a slice on and for the beta struct.
reading_toy!(EntityLike, entity_like, {
    const INPUT: InputKind = InputKind::JsonCpp;
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(i) if i < 1 => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }
});

// Catalog and the camera preset: cereal throughout, the first slice unstrict.
reading_toy!(UnstrictFirst, unstrict_first, {
    const INPUT: InputKind = InputKind::Document;
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }
});

/// Every struct a reading is asked for: each stable index, and the beta struct if any.
fn indices<F: Family>() -> Vec<StructIndex> {
    let versions = <F::Any as VersionChain<F>>::VERSIONS.len();
    let mut out: Vec<_> = (0..versions).map(StructIndex::Stable).collect();
    if <F::Any as VersionChain<F>>::HAS_BETA {
        out.push(StructIndex::Beta);
    }
    out
}

const SOURCES: [DocumentSource; 2] = [DocumentSource::Json, DocumentSource::Mcb];

/// What `load` derives from the old knobs for a slice today: the null rule on the context
/// (none for an MCB source), `strict_components` (the slice rejects `null`, whatever the
/// source), and the error mask.
fn old_knobs<F: Family>(
    index: StructIndex,
    source: DocumentSource,
) -> (Option<NullRule>, bool, ErrorMask) {
    let rule = match source {
        DocumentSource::Mcb => None,
        _ => F::null_rule(index),
    };
    let strict_components = F::null_rule(index) == Some(NullRule::Rejected);
    (rule, strict_components, F::error_mask(index))
}

/// `F::reading` and `F::TEXT` reproduce the old knobs for every slice and source.
fn reproduces_the_knobs<F: Family>(family: &str) {
    for index in indices::<F>() {
        for source in SOURCES {
            let reading = F::reading(index, source);
            assert_eq!(
                (
                    reading.null_rule(),
                    reading.components_fatal,
                    reading.error_mask()
                ),
                old_knobs::<F>(index, source),
                "{family} {index:?} {source:?}: the reading differs from the old knobs",
            );
            assert_eq!(
                reading.scalars,
                Scalars::Strict,
                "{family}: nothing coerces yet"
            );
            assert_eq!(reading.on_member_failure, MemberFailure::Refuse, "{family}");
        }
    }
    let text = F::TEXT;
    assert_eq!(
        (
            text.version_input,
            text.clamps_beyond_double,
            text.duplicates_kept_in
        ),
        (
            F::INPUT.version_input(),
            F::CLAMPS_BEYOND_DOUBLE,
            F::DUPLICATES_KEPT_IN
        ),
        "{family}: the text reading differs from the old knobs",
    );
}

#[test]
fn reading_reproduces_the_old_knobs_of_every_toy_family() {
    reproduces_the_knobs::<Toy>("Toy");
    reproduces_the_knobs::<Stable>("Stable");
    reproduces_the_knobs::<Mixed>("Mixed");
    reproduces_the_knobs::<Masked>("Masked");
    reproduces_the_knobs::<Document>("Document");
    reproduces_the_knobs::<JsonCppToy>("JsonCppToy");
    reproduces_the_knobs::<DocumentStrict>("DocumentStrict");
    reproduces_the_knobs::<LegacyFirst>("LegacyFirst");
    reproduces_the_knobs::<BlockLike>("BlockLike");
    reproduces_the_knobs::<EntityLike>("EntityLike");
    reproduces_the_knobs::<UnstrictFirst>("UnstrictFirst");
}

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
    assert_eq!(
        json::<BlockLike>(StructIndex::Stable(2)),
        Reading::DOCUMENT
    );
    assert_eq!(
        json::<UnstrictFirst>(StructIndex::Stable(0)),
        Reading::DOCUMENT.with_unknown(warn)
    );
    // An MCB source has no null rule; its components stay as the slice reads them.
    assert_eq!(
        DocumentStrict::reading(StructIndex::Stable(0), DocumentSource::Mcb),
        Reading::DOCUMENT.with_null(NullHandling::Own)
    );
}

#[test]
fn text_reading_follows_input_clamp_and_duplicates() {
    assert_eq!(Toy::TEXT.version_input, VersionInput::Root);
    assert!(!Toy::TEXT.sizes_json(DocumentSource::Json));
    assert_eq!(Document::TEXT.version_input, VersionInput::Document);
    assert!(Document::TEXT.sizes_json(DocumentSource::Json));
    assert!(!Document::TEXT.sizes_json(DocumentSource::Mcb));
    assert_eq!(
        BlockLike::TEXT.clamps_beyond_double,
        BlockLike::CLAMPS_BEYOND_DOUBLE
    );
    assert_eq!(BlockLike::TEXT.duplicates_kept_in, &["components"]);
    assert!(!EntityLike::TEXT.sizes_json(DocumentSource::Json));
}

#[test]
fn coercing_profile_is_own_with_coercing_scalars() {
    assert_eq!(Reading::COERCING.scalars, Scalars::Coercing);
    assert_eq!(
        Reading::COERCING.with_scalars(Scalars::Strict),
        Reading::OWN
    );
}

//! Reading dimension documents. The game reads a document with no minimum version, so it never
//! checks the envelope: each slice parses the whole document as one struct that binds
//! `minecraft:dimension`. Unknown members warn, every other failure skips the file.

use facet::Facet;

use super::{DimensionComponents, DimensionDescription};
use crate::history::v1_18_0::Components as ComponentsV1;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    JsonSlice, NullRule, ScalarFailure, ScalarRead, Value, keep_unknown_members, parse_with_rules,
    reject_nulls,
};
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Reading};

/// The whole document of `[1.18.0, 1.26.50)` (vanilla `v1_21_60::DimensionDocument`).
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct RootV1 {
    #[facet(rename = "minecraft:dimension")]
    pub(crate) dimension: InnerV1,
}

/// `minecraft:dimension` of `[1.18.0, 1.26.50)`.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct InnerV1 {
    pub(crate) description: DimensionDescription,
    pub(crate) components: ComponentsV1,
}

/// The whole document from 1.26.50 (vanilla `v1_26_50::DimensionDocument`).
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct RootLatest {
    #[facet(rename = "minecraft:dimension")]
    pub(crate) dimension: InnerLatest,
}

/// `minecraft:dimension` from 1.26.50.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct InnerLatest {
    pub(crate) description: DimensionDescription,
    pub(crate) components: DimensionComponents,
}

/// The root member the envelope reads: the crate does not report it as an unknown member of the
/// root struct, although vanilla binds no `format_version` and so would.
const FORMAT_VERSION: &str = "format_version";

/// The first version the cereal slice `v1_21_60::DimensionDocument` reads; below it a
/// hand-written legacy parser reads the same struct.
const CEREAL_FLOOR: FormatVersion = FormatVersion::new(1, 21, 60);

/// The null rule the `v1_18_0` struct reads under: the family's (N2) from 1.21.60, where the
/// cereal slice reads it; below, the legacy parser's own reading, in which a `null` component
/// is absent.
pub(crate) fn slice_null_rule(cx: &ParseCx<'_>) -> Option<NullRule> {
    cx.load
        .null_rule
        .filter(|_| cx.original_version >= CEREAL_FLOOR)
}

/// Parses the whole document `text` as `R`, unknown members kept and warned (the root's
/// `format_version` excepted); a failure is one error at the failing member. Under rule N2
/// (`rule`) a `null` where `R` reads a value is an `InvalidType` error and the document is
/// rejected.
pub(crate) fn parse_root<R: for<'f> Facet<'f>>(
    text: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    rule: Option<NullRule>,
    d: &mut Diagnostics,
) -> Option<R> {
    // The scalars are read by the slice's rules (`Family::reading`: strict, a number beyond
    // its integer a type error).
    // A `null` read as a struct is the null rule's: `reject_nulls` below tells it from 1.21.60,
    // and the legacy parser below it reads it as its defaults untold.
    let rules = cx.load.scalar_rules.with_tells_null_struct(false);
    let ScalarRead { value, told, .. } = match parse_with_rules::<R>(text, rule, rules) {
        Ok(read) => read,
        Err(ScalarFailure::Read(error)) => {
            d.push(error.into_diagnostic(&cx.at, |code, _| code));
            return None;
        }
        Err(ScalarFailure::Refused(misread)) => {
            d.push(misread.finding(&cx.at));
            return None;
        }
    };
    // Under the strict rules the pre-pass refuses every conversion, so it tells nothing.
    for misread in told {
        d.push(misread.finding(&cx.at));
    }
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, text, &cx.at, d) {
        return None;
    }
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(text.text) {
        let mut kept = bedrock_addon_core::document::Unknown::new();
        let found = keep_unknown_members(&value, &json, &mut kept);
        // The root's `format_version` is the envelope's, written by the struct itself.
        kept.take(&JsonPath::new(), FORMAT_VERSION);
        d.unknown_mut().absorb(&cx.at, kept);
        for (path, _) in found {
            if path.is_key(FORMAT_VERSION) {
                continue;
            }
            let mut at = cx.at.clone();
            at.append(&path);
            // From 1.21.60 the cereal slice reads leniently and logs an unknown member at the
            // content log's warning level. Below it the legacy parser reads only what it knows,
            // in silence, so the crate reports an information.
            let code = DiagnosticCode::UnrecognizedField { text: None };
            let finding = if cx.original_version >= CEREAL_FLOOR {
                Diagnostic::warn(code)
            } else {
                Diagnostic::info(code)
            };
            d.push(finding.at(at));
        }
    }
    Some(value)
}

/// The bind constraints `value` states as its bounds and steps (`Schema::check`), at `at`:
/// each member outside its range, or in it and off its step, is one `InvalidValue` error with
/// cereal's text (the range first), which fails the file as vanilla's bind does. `false` when
/// one broke.
pub(crate) fn constraints<T: Schema>(value: &mut T, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let mut cx = SchemaCx::new(Reading::DOCUMENT, at.clone(), d).with_range_text(true);
    value.check(&mut cx);
    !cx.failed()
}

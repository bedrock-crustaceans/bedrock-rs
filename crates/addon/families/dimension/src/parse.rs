//! Reading dimension documents. The game reads a document with no minimum version, so it never
//! checks the envelope: each slice parses the whole document as one struct that binds
//! `minecraft:dimension`. Unknown members warn, every other failure skips the file.

use facet::Facet;

use super::{Clouds, DimensionBounds, DimensionComponents, DimensionDescription, DimensionHeight};
use crate::history::v1_18_0::Components as ComponentsV1;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    JsonSlice, NullRule, Value, keep_unknown_members, parse_at, reject_nulls,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ParseCx;

/// The whole document of `[1.18.0, 1.26.50)` (vanilla `v1_21_60::DimensionDocument`).
#[derive(Facet)]
pub(crate) struct RootV1 {
    #[facet(rename = "minecraft:dimension")]
    pub(crate) dimension: InnerV1,
    // A skipped field still takes the JSON member of its name in facet rc.7: a NUL-prefixed
    // name no document has keeps a root member `extra` an unknown member.
    #[facet(skip, rename = "\u{0}extra")]
    pub(crate) extra: Extra,
}

/// `minecraft:dimension` of `[1.18.0, 1.26.50)`.
#[derive(Facet)]
pub(crate) struct InnerV1 {
    pub(crate) description: DimensionDescription,
    pub(crate) components: ComponentsV1,
    #[facet(skip, rename = "\u{0}extra")]
    pub(crate) extra: Extra,
}

/// The whole document from 1.26.50 (vanilla `v1_26_50::DimensionDocument`).
#[derive(Facet)]
pub(crate) struct RootLatest {
    #[facet(rename = "minecraft:dimension")]
    pub(crate) dimension: InnerLatest,
    // A skipped field still takes the JSON member of its name in facet rc.7: a NUL-prefixed
    // name no document has keeps a root member `extra` an unknown member.
    #[facet(skip, rename = "\u{0}extra")]
    pub(crate) extra: Extra,
}

/// `minecraft:dimension` from 1.26.50.
#[derive(Facet)]
pub(crate) struct InnerLatest {
    pub(crate) description: DimensionDescription,
    pub(crate) components: DimensionComponents,
    #[facet(skip, rename = "\u{0}extra")]
    pub(crate) extra: Extra,
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
    root_extra: impl Fn(&mut R) -> &mut Extra,
    d: &mut Diagnostics,
) -> Option<R> {
    let mut value = match parse_at::<R>(text) {
        Ok(value) => value,
        Err(error) => {
            d.push(error.into_diagnostic(&cx.at, |code, _| code));
            return None;
        }
    };
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, text, &cx.at, d) {
        return None;
    }
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(text.text) {
        for (path, _) in keep_unknown_members(&mut value, &json) {
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
    root_extra(&mut value).remove(FORMAT_VERSION);
    Some(value)
}

/// `base` followed by `keys`.
pub(crate) fn child(base: &JsonPath, keys: &[&str]) -> JsonPath {
    let mut path = base.clone();
    for key in keys {
        path.push(key);
    }
    path
}

/// A bind constraint: `value` in `[min, max]`, and a multiple of `multiple` when given; a
/// violation fails the file in vanilla, and is an `InvalidValue` error here with cereal's
/// text. The range is checked first.
pub(crate) fn constraint(
    value: i32,
    (min, max): (i32, i32),
    multiple: Option<i32>,
    at: JsonPath,
    d: &mut Diagnostics,
) -> bool {
    // The shared texts: the range message names the bounds, not the value.
    let code = if !(min..=max).contains(&value) {
        bedrock_addon_core::diagnostic::cereal::NumericRange::inclusive(
            f64::from(min),
            f64::from(max),
        )
        .code()
    } else if let Some(multiple) = multiple.filter(|m| value % m != 0) {
        bedrock_addon_core::diagnostic::cereal::not_multiple_of(
            f64::from(value),
            f64::from(multiple),
        )
    } else {
        return true;
    };
    d.push(Diagnostic::error(code).at(at));
    false
}

/// The bind constraints of `minecraft:dimension_bounds` (`{-512, 512, multipleOf 16}` on both
/// members), the same in every format that reads the bounds.
pub(crate) fn bounds_constraints(
    bounds: &DimensionBounds,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let min = constraint(bounds.min.0, (-512, 512), Some(16), child(at, &["min"]), d);
    let max = constraint(bounds.max.0, (-512, 512), Some(16), child(at, &["max"]), d);
    min && max
}

/// The 1.26.50 schema constraints of `minecraft:dimension_height`: `min_y` in `[-512, 512]`,
/// `height_range` in `[16, 1024]` and a multiple of 16.
pub(crate) fn height_constraints(
    height: &DimensionHeight,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let min_y = constraint(
        i32::from(height.min_y.0),
        (-512, 512),
        None,
        child(at, &["min_y"]),
        d,
    );
    let range = constraint(
        i32::from(height.height_range.0),
        (16, 1024),
        Some(16),
        child(at, &["height_range"]),
        d,
    );
    min_y && range
}

/// The bind constraint of `minecraft:clouds`: `height` in `[-512, 512]`.
pub(crate) fn clouds_constraint(clouds: &Clouds, at: &JsonPath, d: &mut Diagnostics) -> bool {
    constraint(
        i32::from(clouds.height.0),
        (-512, 512),
        None,
        child(at, &["height"]),
        d,
    )
}

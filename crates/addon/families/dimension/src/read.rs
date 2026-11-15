//! Reading dimension documents. The game reads a document with no minimum version, so it never
//! checks the envelope: each slice parses the whole document as one struct that binds
//! `minecraft:dimension`. Unknown members warn, every other failure skips the file.

use facet::Facet;

use super::{DimensionComponents, DimensionDescription};
use crate::history::v1_18_0::Components as ComponentsV1;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{NullHandling, ParseCx, read_payload_with};

/// The whole document of `[1.18.0, 1.26.50)` (vanilla `v1_21_60::DimensionDocument`). The root's
/// `format_version` is the envelope's: bound here so it is no unknown member, though vanilla
/// binds none and would report it.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct RootV1 {
    #[facet(rename = "minecraft:dimension")]
    pub(crate) dimension: InnerV1,
    #[facet(rename = "format_version")]
    version: Option<Value>,
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
    #[facet(rename = "format_version")]
    version: Option<Value>,
}

/// `minecraft:dimension` from 1.26.50.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub(crate) struct InnerLatest {
    pub(crate) description: DimensionDescription,
    pub(crate) components: DimensionComponents,
}

/// The first version the cereal slice `v1_21_60::DimensionDocument` reads; below it a
/// hand-written legacy parser reads the same struct.
const CEREAL_FLOOR: FormatVersion = FormatVersion::new(1, 21, 60);

/// Reads the whole document `text` as `R` with core's reader (`versioning::read_payload_with`),
/// unknown members kept and warned; a failure is one error at the failing member. From 1.21.60
/// the cereal slice reads under the slice's reading: a `null` where `R` reads a value is an
/// `InvalidType` error that rejects the document, an unknown member a warning at the content
/// log's level. Below it the legacy parser reads only what it knows, in silence (the crate
/// reports an information) and a `null` component is absent.
pub(crate) fn parse_root<R: FromJson>(
    text: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<R> {
    read_payload_with(text, cx, d, |read| {
        if cx.original_version >= CEREAL_FLOOR {
            read
        } else {
            read.null(NullHandling::Own)
                .unknown_severity(Severity::Info)
        }
    })
}

/// The bind constraints `value` states as its bounds and steps (`Schema::check`), at `at`:
/// each member outside its range, or in it and off its step, is one `InvalidValue` error with
/// cereal's text (the range first), which fails the file as vanilla's bind does. `false` when
/// one broke.
pub(crate) fn constraints<T: Schema>(value: &mut T, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let mut cx = SchemaCx::new(crate::JSON_READING, at.clone(), d).with_range_text(true);
    value.check(&mut cx);
    !cx.failed()
}

//! The POI documents as written, and their chains.
//!
//! Both documents (`minecraft:poi_block`, `minecraft:poi_tag`) have one struct each, at 1.26.40,
//! read from 1.26.40 up to the engine version: no older shape exists and there is no upgrade
//! step.

use super::{PoiTag, PoiType};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::Single;

/// The `poi_block` family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PoiTypeFamily;

/// The `poi_tag` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PoiTagFamily;

/// A `minecraft:poi_block` document parsed by the struct its `format_version` selects: the one
/// type struct (format 1.26.40 and later).
pub type PoiTypeAny = Single<PoiTypeFamily>;

/// A `minecraft:poi_tag` document parsed by the struct its `format_version` selects: the one
/// tag struct (format 1.26.40 and later).
pub type PoiTagAny = Single<PoiTagFamily>;

bedrock_addon_core::impl_no_molang!(PoiType, PoiTag);

/// Runs the rest of the load on a document [`PoiType::parse_as_written`] gave: the same result
/// `Content::parse` gives for its file.
///
/// [`PoiType::parse_as_written`]: bedrock_addon_core::document::JsonDocument::parse_as_written
///
/// # Errors
///
/// Returns [`Rejected`] when a check refuses the document, or the `poi` experiment is off; it
/// carries every finding about the file.
pub fn upgrade_type(
    document: Document<PoiTypeAny>,
    cx: &LoadContext,
) -> Result<Document<PoiType>, Rejected> {
    super::upgrade::<PoiTypeFamily>(document, cx)
}

/// [`upgrade_type`] of a `minecraft:poi_tag` document.
///
/// # Errors
///
/// Returns [`Rejected`] when a check refuses the document, or the `poi` experiment is off; it
/// carries every finding about the file.
pub fn upgrade_tag(
    document: Document<PoiTagAny>,
    cx: &LoadContext,
) -> Result<Document<PoiTag>, Rejected> {
    super::upgrade::<PoiTagFamily>(document, cx)
}

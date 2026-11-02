//! Texture sets as written, and the chain that reads them.
//!
//! The chain is `v1_16_100 → v1_21_30`. The client has no upgrade path: each of its two schemas
//! reads only the documents whose `format_version` is exactly its version, and both fill the
//! same runtime value. The one step is therefore the identity; it exists so the newest
//! document is a single type. Texture sets have no beta struct.

pub mod v1_16_100;

use super::TextureSet;
use super::set_load::upgrade_document;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The texture-set family of the version-chain machinery (hidden: only the loader names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct TextureSetFamily;

bedrock_addon_core::version_chain! {
    family = TextureSetFamily;
    /// A texture set parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.16.100 exactly: four layers.
        V1_16_100(v1_16_100::TextureSet),
        /// Format 1.21.30 exactly: the newest document.
        V1_21_30(TextureSet),
    }
    molang = none;
}

/// Runs the upgrade step of a document [`TextureSet::parse_as_written`] gave: the same result
/// [`TextureSet::parse`](bedrock_addon_core::document::Content::parse) gives for its file (the step
/// runs over the document, which holds the origin).
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<TextureSet>, Rejected> {
    upgrade_document(document, cx)
}

//! The biome document as written in older format versions, and the steps that upgrade it to
//! [`super::Biome`].
//!
//! The chain is `v1_21_110 → v1_21_120 → v1_26_0 → v1_26_30`; each `vX_Y_Z::Biome` parses
//! `[X.Y.Z, next)` and its `Upgrade` impl is that version's step. A document from 1.26.30 up to
//! the engine version parses as [`super::Biome`]; `"format_version": "beta"` parses as
//! [`super::Beta`] and is never upgraded. Below 1.21.110 a document is not loaded: every
//! earlier format was experimental, and vanilla rejects it.

pub mod v1_21_110;
pub mod v1_21_120;
pub mod v1_26_0;

use super::{Beta, Biome};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The biome family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BiomeFamily;

bedrock_addon_core::version_chain! {
    family = BiomeFamily;
    /// A biome document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.21.110: read from the whole document.
        V1_21_110(v1_21_110::Biome),
        /// Format 1.21.120.
        V1_21_120(v1_21_120::Biome),
        /// Format 1.26.0.
        V1_26_0(v1_26_0::Biome),
        /// Format 1.26.30 and later: the newest document.
        V1_26_30(Biome),
    }
    beta = /// `"format_version": "beta"`.
    Beta(Beta);
}

/// Runs the remaining upgrade steps of a document [`Biome::parse_as_written`] gave: the same
/// result [`Biome::parse`](bedrock_addon_core::document::Content::parse) gives for its file
/// (`history::Any::upgrade`, realised over the document, which holds the origin of the file).
/// A beta document is rejected here (it is a [`Beta`]).
///
/// # Errors
///
/// Returns [`Rejected`] when the document is a beta one, or a remaining upgrade step leaves an error finding.
pub fn upgrade(document: Document<Any>, cx: &LoadContext) -> Result<Document<Biome>, Rejected> {
    super::load::upgrade_document(document, cx)
}

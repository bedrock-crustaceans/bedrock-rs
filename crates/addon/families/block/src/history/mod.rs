//! The block document as written in older format versions, and the steps that upgrade it to
//! [`super::Block`].
//!
//! The chain is `v1_16_0 → v1_16_100 → v1_18_0 → v1_19_10 → v1_19_20 → v1_19_30 → v1_19_40 →
//! v1_19_80 → v1_20_10 → v1_20_60 → v1_21_30 → v1_21_60 → v1_21_90 → v1_26_0 → v1_26_20 →
//! v1_26_60`; each `vX_Y_Z::Block` parses `[X.Y.Z, next)` and its step is that version's fixer
//! (below 1.26.20 the struct is `Legacy<BlockPayload, X, Y, Z>` and the step
//! `LegacyStep<X, Y, Z>` on [`BlockPayload`]). The 1.26.20 document (`v1_26_20`) is the first cereal one; a document
//! from 1.26.60 up to the engine version parses as [`super::Block`] and runs no step.
//! Below 1.16.0 a document is not loaded. Blocks have no struct for an unreleased format.

mod legacy;
mod ops;

pub mod v1_16_0;
pub mod v1_16_100;
pub mod v1_18_0;
pub mod v1_19_10;
pub mod v1_19_20;
pub mod v1_19_30;
pub mod v1_19_40;
pub mod v1_19_80;
pub mod v1_20_10;
pub mod v1_20_60;
pub mod v1_21_30;
pub mod v1_21_60;
pub mod v1_21_90;
pub mod v1_26_0;
pub mod v1_26_20;

pub use legacy::{BlockPayload, LegacyPermutation};

pub(crate) use legacy::to_object;

use super::Block;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The block family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct BlockFamily;

bedrock_addon_core::version_chain! {
    family = BlockFamily;
    /// A block document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum BlockHistory {
        /// Format 1.16.0.
        V1_16_0(v1_16_0::Block),
        /// Format 1.16.100.
        V1_16_100(v1_16_100::Block),
        /// Format 1.18.0.
        V1_18_0(v1_18_0::Block),
        /// Format 1.19.10.
        V1_19_10(v1_19_10::Block),
        /// Format 1.19.20.
        V1_19_20(v1_19_20::Block),
        /// Format 1.19.30.
        V1_19_30(v1_19_30::Block),
        /// Format 1.19.40.
        V1_19_40(v1_19_40::Block),
        /// Format 1.19.80.
        V1_19_80(v1_19_80::Block),
        /// Format 1.20.10.
        V1_20_10(v1_20_10::Block),
        /// Format 1.20.60.
        V1_20_60(v1_20_60::Block),
        /// Format 1.21.30.
        V1_21_30(v1_21_30::Block),
        /// Format 1.21.60.
        V1_21_60(v1_21_60::Block),
        /// Format 1.21.90.
        V1_21_90(v1_21_90::Block),
        /// Format 1.26.0.
        V1_26_0(v1_26_0::Block),
        /// Format 1.26.20: the first cereal document.
        V1_26_20(v1_26_20::Block),
        /// Format 1.26.60 and later: the newest document.
        V1_26_60(Block),
    }
}

/// Runs the remaining upgrade steps of a document [`Block::parse_as_written`](bedrock_addon_core::document::JsonDocument::parse_as_written) gave: the same
/// result [`Block::parse`](bedrock_addon_core::document::Content::parse) gives for its file (`history::BlockHistory::upgrade`, realised over the document, which holds the origin).
///
/// # Errors
///
/// Returns [`Rejected`] when a remaining upgrade step leaves an error finding.
pub fn upgrade(
    document: Document<BlockHistory>,
    cx: &LoadContext,
) -> Result<Document<Block>, Rejected> {
    super::read::upgrade_document(document, cx)
}

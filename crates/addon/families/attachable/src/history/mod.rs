//! The attachable document as written in older format versions, and the steps that upgrade
//! it to [`super::Attachable`].
//!
//! The chain is `v1_8_0 → v1_10_0 → v1_26_0`: [`v1_8_0::Attachable`] parses `[1.8.0, 1.10.0)`,
//! [`v1_10_0::Attachable`] parses `[1.10.0, 1.26.0)`, and a document from 1.26.0 up to the
//! engine version parses as [`super::Attachable`] and runs no step. Below 1.8.0 a document is
//! not loaded; one without `format_version` is read as 1.8.0. Attachables have no beta
//! struct.

pub mod v1_10_0;
pub mod v1_8_0;

use super::Attachable;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The attachable family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AttachableFamily;

bedrock_addon_core::version_chain! {
    family = AttachableFamily;
    /// An attachable document parsed by the struct its `format_version` selects, not
    /// upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum AttachableHistory {
        /// Format 1.8.0 up to 1.10.0.
        V1_8_0(v1_8_0::Attachable),
        /// Format 1.10.0 up to 1.26.0.
        V1_10_0(v1_10_0::Attachable),
        /// Format 1.26.0 and later: the newest document.
        V1_26_0(Attachable),
    }
}

/// Runs the remaining upgrade steps of a document [`Attachable::parse_as_written`](bedrock_addon_core::document::JsonDocument::parse_as_written) gave: the
/// same result [`Attachable::parse`](bedrock_addon_core::document::Content::parse) gives for its file
/// (`history::AttachableHistory::upgrade`, realised over the document, which holds the
/// origin).
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<AttachableHistory>,
    cx: &LoadContext,
) -> Result<Document<Attachable>, Rejected> {
    crate::upgrade_document(document, cx)
}

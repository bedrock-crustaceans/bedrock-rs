//! The client entity document as written in older format versions, and the steps that upgrade
//! it to [`super::ClientEntity`].
//!
//! The chain is `v1_8_0 → v1_10_0 → v1_26_0`: [`v1_8_0::ClientEntity`] parses `[1.8.0, 1.10.0)`,
//! [`v1_10_0::ClientEntity`] parses `[1.10.0, 1.26.0)`, and a document from 1.26.0 up to the
//! engine version parses as [`super::ClientEntity`] and runs no step. Below 1.8.0 a document is
//! not loaded; one without `format_version` is read as 1.8.0. Client entities have no beta
//! struct.

pub mod v1_10_0;
pub mod v1_8_0;

use super::ClientEntity;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The client entity family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ClientEntityFamily;

bedrock_addon_core::version_chain! {
    family = ClientEntityFamily;
    /// A client entity document parsed by the struct its `format_version` selects, not
    /// upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.8.0 up to 1.10.0.
        V1_8_0(v1_8_0::ClientEntity),
        /// Format 1.10.0 up to 1.26.0.
        V1_10_0(v1_10_0::ClientEntity),
        /// Format 1.26.0 and later: the newest document.
        V1_26_0(ClientEntity),
    }
}

/// Runs the remaining upgrade steps of a document [`ClientEntity::parse_as_written`] gave: the
/// same result [`ClientEntity::parse`](bedrock_addon_core::document::Content::parse) gives for its file.
///
/// The split lets a caller inspect the document as written before it is upgraded.
///
/// # Errors
///
/// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
/// checks that follow it, find an error that stops the document from loading.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<ClientEntity>, Rejected> {
    super::load::upgrade_document(document, cx)
}

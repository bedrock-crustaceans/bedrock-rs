//! Trade tables as written before format 1.21.30, and the step that upgrades them to
//! [`TradeTable`].
//!
//! The chain is `v0 → v1_21_30`: a document without `format_version` (or with one that is not
//! a string), or with a version below 1.21.30, is read by vanilla's legacy parser
//! ([`v0::TradeTable`]); from 1.21.30 up to the engine version it is the newest document and
//! runs no step. Vanilla's `"1.26.10"` stamp is such a version: no newer trade struct exists.
//! Trade tables have no beta struct.

mod convert;
mod read;
pub mod v0;

use super::TradeTable;
use super::upgrade_document;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The trade-table family of the version-chain machinery (hidden: only the loader names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct TradeFamily;

bedrock_addon_core::version_chain! {
    family = TradeFamily;
    /// A trade table parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// No `format_version`, or one below 1.21.30: vanilla's legacy parser.
        V0(v0::TradeTable),
        /// Format 1.21.30 and later: the newest document.
        V1_21_30(TradeTable),
    }
    molang = none;
}

/// Runs the remaining upgrade step of a document
/// [`TradeTable::parse_as_written`](bedrock_addon_core::document::JsonDocument::parse_as_written)
/// gave: the same result [`TradeTable::parse`](bedrock_addon_core::document::Content::parse)
/// gives for its file.
/// It works over the document because the document holds the origin.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<TradeTable>, Rejected> {
    upgrade_document(document, cx)
}

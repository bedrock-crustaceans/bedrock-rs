//! Feature documents as written in older format versions, and the steps that upgrade them to
//! [`Feature`].
//!
//! Three feature types changed their schema: `scatter_feature` at 1.21.10 (the scatter
//! parameters moved into `distribution`), `single_block_feature` at 1.21.40 (weighted
//! `places_block`, `randomize_rotation`, `may_not_attach_to`, `diagonal`) and
//! `snap_to_surface_feature` at 1.26.50 (`vertical_search_range` renamed `search_range`).
//! [`v1_13_0`] has their older structs, each with its `Upgrade` impl. Every other type has one
//! schema from 1.13.0 (`multi_block_feature` from 1.26.40), so its documents parse as the
//! newest struct and run no step. Below its floor a document is not loaded. Features have no
//! beta struct.
//!
//! Each type's chain is its history (`OreHistory`, `ScatterHistory`, `TreeHistory` holding
//! the tree boxed), and [`FeatureHistory`] has one variant per type holding it.

pub mod v1_13_0;

use super::Feature;
use super::upgrade_document;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

// One history per feature type (`OreHistory`, `ScatterHistory`, …, declared with the type's
// row) and the enum of them, `FeatureHistory`.
pub use crate::family::histories::*;

/// Runs the remaining upgrade steps of a document [`Feature::parse_as_written`] gave: the same
/// result [`Feature::parse`](bedrock_addon_core::document::Content::parse) gives for its file.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<FeatureHistory>,
    cx: &LoadContext,
) -> Result<Document<Feature>, Rejected> {
    upgrade_document(document, cx)
}

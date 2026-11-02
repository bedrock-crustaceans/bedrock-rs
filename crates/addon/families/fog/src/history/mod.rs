//! Fog settings as written in older format versions, and the steps that upgrade them to
//! [`FogSettings`].
//!
//! The chain is `v1_16_100 → v1_21_0 → v1_21_90 → v1_26_20`. A file from 1.16.100 up to 1.26.20
//! parses as the struct of its range, each a root of the game's strict older schema (an
//! unknown member fails the file); the first two steps are plain copies and the third converts
//! to the newest shape. Exactly 1.26.20 parses as [`FogSettings`] itself; nothing above it and
//! nothing below 1.16.100 is read. A pre-release such as `1.26.20-beta.1` sorts below its
//! release and takes the older struct.

pub mod v1_16_100;
pub mod v1_21_0;
pub mod v1_21_90;

use super::FogSettings;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The `minecraft:fog_settings` family of the version-chain machinery (hidden: only the
/// loader names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct FogFamily;

bedrock_addon_core::version_chain! {
    family = FogFamily;
    /// Fog settings parsed by the struct their `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.16.100 up to 1.21.0.
        V1_16_100(v1_16_100::FogDocument),
        /// Format 1.21.0 up to 1.21.90.
        V1_21_0(v1_21_0::FogDocument),
        /// Format 1.21.90 up to 1.26.20.
        V1_21_90(v1_21_90::FogDocument),
        /// Format 1.26.20: the newest settings.
        V1_26_20(FogSettings),
    }
    molang = none;
}

/// Runs the remaining upgrade steps and the checks of a document
/// [`FogSettings::parse_as_written`] gave: the same result
/// [`FogSettings::parse`](bedrock_addon_core::document::Content::parse) gives for its file (that is
/// `history::Any::upgrade`, over the document, which holds the origin).
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<FogSettings>, Rejected> {
    crate::upgrade_document(document, cx)
}

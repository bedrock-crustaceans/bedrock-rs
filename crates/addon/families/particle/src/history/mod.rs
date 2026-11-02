//! The particle document as written below format 1.20.80, and the step that upgrades it to
//! [`super::ParticleEffect`].
//!
//! The chain is `v1_10_0 → v1_20_80`: [`v1_10_0::ParticleEffect`] parses `[1.10.0, 1.20.80)`
//! by the rules of the game's legacy parser, and its `Upgrade` impl is the conversion the
//! game runs on it. A document from 1.20.80 up to the engine version parses as
//! [`super::ParticleEffect`] and runs no step. Below 1.10.0 a document is not loaded.
//! Particles have no struct for the game's experimental builds.

pub mod v1_10_0;

use super::ParticleEffect;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The particle family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ParticleFamily;

bedrock_addon_core::version_chain! {
    family = ParticleFamily;
    /// A particle document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.10.0 up to, not including, 1.20.80.
        V1_10_0(v1_10_0::ParticleEffect),
        /// Format 1.20.80 and later: the newest document.
        V1_20_80(ParticleEffect),
    }
}

/// Runs the upgrade step of a document [`ParticleEffect::parse_as_written`] gave: the same
/// result [`ParticleEffect::parse`](bedrock_addon_core::document::Content::parse) gives for its file.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<ParticleEffect>, Rejected> {
    super::load::upgrade_document(document, cx)
}

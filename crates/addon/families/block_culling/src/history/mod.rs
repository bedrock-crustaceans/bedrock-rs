//! The block-culling rules as written in older format versions.
//!
//! Culling rules have a chain, `v1_20_60 → v1_21_80`: [`v1_20_60::CullingRules`] parses
//! `[1.20.60, 1.21.80)` and [`super::CullingRules`] exactly 1.21.80 (the newest struct closes
//! the last range at its own version, below the engine's). The step copies every member; the
//! 1.21.80 members start as the client's defaults. There is no beta struct.

pub mod v1_20_60;

/// The block-culling family of the version-chain machinery (hidden: only the loader names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CullingFamily;

bedrock_addon_core::version_chain! {
    family = CullingFamily;
    /// A culling-rules document parsed by the struct its `format_version` selects, not
    /// upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum CullingHistory {
        /// Format 1.20.60 up to 1.21.80.
        V1_20_60(v1_20_60::CullingRules),
        /// Format 1.21.80: the newest document.
        V1_21_80(super::CullingRules),
    }
    molang = none;
}

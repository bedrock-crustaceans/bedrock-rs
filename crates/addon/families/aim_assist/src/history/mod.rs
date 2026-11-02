//! The aim-assist documents as written in older format versions, and their chains.
//!
//! Presets have two structs: [`v1_21_50::AimAssistPreset`] parses `[1.21.50, 1.21.120)` and its
//! step turns `exclusion_list` into `exclusion_settings`; [`AimAssistPreset`] parses from
//! 1.21.120 up to the engine version. Category files have one struct, at 1.21.50. Nothing below
//! 1.21.50 loads, and neither family has a beta struct.

pub mod v1_21_50;

use super::{AimAssistCategories, AimAssistPreset};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The `minecraft:aim_assist_preset` family of the version-chain machinery (hidden: only the
/// loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PresetFamily;

/// The `minecraft:aim_assist_categories` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CategoriesFamily;

bedrock_addon_core::version_chain! {
    family = PresetFamily;
    /// An aim-assist preset parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.21.50 up to 1.21.120.
        V1_21_50(v1_21_50::AimAssistPreset),
        /// Format 1.21.120 and later: the newest preset.
        V1_21_120(AimAssistPreset),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = CategoriesFamily;
    /// An aim-assist categories file parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum CategoriesAny {
        /// Format 1.21.50 and later: the only categories struct.
        V1_21_50(AimAssistCategories),
    }
    molang = none;
}

/// Runs the remaining upgrade step of a preset [`AimAssistPreset::parse_as_written`] gave: the same
/// result [`AimAssistPreset::parse`](bedrock_addon_core::document::Content::parse) gives for its
/// file. The step runs over the document, which holds the origin the step needs.
///
/// # Errors
///
/// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
/// checks that follow it, find an error that stops the document from loading.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<AimAssistPreset>, Rejected> {
    bedrock_addon_support::chain_file::upgrade::<PresetFamily>(
        document,
        bedrock_addon_core::diagnostic::JsonPath::new(),
        cx,
    )
}

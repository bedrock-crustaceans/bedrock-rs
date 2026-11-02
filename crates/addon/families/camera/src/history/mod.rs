//! The camera documents as written in older format versions, and their chains.
//!
//! Presets have two structs, one per slice of vanilla's loader: [`v1_19_50::CameraPreset`]
//! parses `[1.19.50, 1.26.50)` (vanilla's legacy slice, read by the non-strict reader), and its
//! step copies every member into [`CameraPreset`] (1.26.50, the strict slice) and sets
//! [`CameraPreset::apply_inherited_starting_rotation`]. Lists (1.19.50), spline files (1.26.0)
//! and camera entity documents (any version) have one struct each and no step. No camera
//! family has a beta struct.

pub mod v1_19_50;

use super::{CameraEntity, CameraList, CameraPreset, CameraSpline};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;

/// The `minecraft:camera_preset` family of the version-chain machinery (hidden: only the
/// loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PresetFamily;

/// The `minecraft:camera_list` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ListFamily;

/// The `minecraft:camera_custom_splines` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct SplineFamily;

/// The `minecraft:camera_entity` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct EntityFamily;

bedrock_addon_core::version_chain! {
    family = PresetFamily;
    /// A camera preset parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.19.50 up to 1.26.50 (vanilla's legacy slice).
        V1_19_50(v1_19_50::CameraPreset),
        /// Format 1.26.50 and later: the newest preset.
        V1_26_50(CameraPreset),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = ListFamily;
    /// A camera list parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum ListAny {
        /// Format 1.19.50 and later: the only list struct.
        V1_19_50(CameraList),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = SplineFamily;
    /// A camera spline file parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum SplineAny {
        /// Format 1.26.0 and later: the only spline struct.
        V1_26_0(CameraSpline),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = EntityFamily;
    /// A camera entity document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum EntityAny {
        /// Every format version up to the engine's: the only camera entity struct.
        V0_0_0(CameraEntity),
    }
    molang = none;
}

/// Runs the remaining upgrade step of a preset [`CameraPreset::parse_as_written`] gave: the
/// same result [`CameraPreset::parse`](bedrock_addon_core::document::Content::parse) gives for
/// its file. It works on the document, which holds the origin, so the split lets a caller
/// inspect the file as written before it is upgraded.
///
/// # Errors
///
/// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
/// checks that follow it, find an error that stops the document from loading.
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<CameraPreset>, Rejected> {
    bedrock_addon_support::chain_file::upgrade::<PresetFamily>(
        document,
        bedrock_addon_core::diagnostic::JsonPath::new(),
        cx,
    )
}

pub use bedrock_addon_support::chain_file::SourceVersion;

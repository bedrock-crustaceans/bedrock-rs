//! `minecraft:camera_list`: a named list of camera presets.

use bedrock_addon_core::id::{CameraListId, CameraPresetId};

/// A camera list (`minecraft:camera_list`, in `cameras/lists/`): the presets a player may
/// switch between.
///
/// JSON: `{"format_version": "1.19.50", "minecraft:camera_list": {"identifier": "ns:name",
/// "cameras": ["ns:preset", …]}}`, every member required. The list must name at least one
/// preset, each once, and every preset must be loaded: vanilla reads the lists after the presets
/// of every pack, so a preset the stack discarded is unknown to a list.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:camera_list")]
#[doc(alias = "camera_list")]
pub struct CameraList {
    /// The list's name, `namespace:name` (JSON `identifier`).
    pub identifier: CameraListId,
    /// The presets of the list, in order (JSON `cameras`).
    pub cameras: Vec<CameraPresetId>,
}

impl CameraList {
    /// A list named `identifier` of `cameras`.
    #[must_use]
    pub fn new(identifier: CameraListId, cameras: Vec<CameraPresetId>) -> Self {
        Self {
            identifier,
            cameras,
        }
    }
}

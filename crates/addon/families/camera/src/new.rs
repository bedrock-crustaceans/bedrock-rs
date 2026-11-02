//! Constructors of the camera documents.

use super::{CameraList, CameraPreset};
use bedrock_addon_core::id::{CameraListId, CameraPresetId};

impl CameraPreset {
    /// A preset named `identifier` that sets nothing.
    #[must_use]
    pub fn new(identifier: CameraPresetId) -> Self {
        Self {
            identifier,
            inherit_from: None,
            pos_x: None,
            pos_y: None,
            pos_z: None,
            rot_x: None,
            rot_y: None,
            rotation_speed: None,
            snap_to_target: None,
            horizontal_rotation_limit: None,
            vertical_rotation_limit: None,
            continue_targeting: None,
            tracking_radius: None,
            view_offset: None,
            entity_offset: None,
            radius: None,
            yaw_limits_min: None,
            yaw_limits_max: None,
            listener: None,
            player_effects: None,
            aim_assist: None,
            control_scheme: None,
            starting_rot: None,
            apply_inherited_starting_rotation: false,
        }
    }
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

//! The camera preset at `[1.19.50, 1.26.50)`, which vanilla's legacy slice parses, and its step
//! to 1.26.50.

use super::super::{AudioListener, ControlScheme, PresetAimAssist};
use super::SourceVersion;
use bedrock_addon_core::id::CameraPresetId;
use bedrock_addon_core::json::{StrictBool, StrictFloat};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};

/// A camera preset as written below format 1.26.50: [`super::super::CameraPreset`] without
/// `starting_rot`.
///
/// No member is gated by the format version inside the range: a 1.19.50 document may use
/// `control_scheme` or `aim_assist`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct CameraPreset {
    /// JSON `identifier`.
    pub identifier: CameraPresetId,
    /// JSON `inherit_from`.
    pub inherit_from: Option<CameraPresetId>,
    /// JSON `pos_x`.
    pub pos_x: Option<StrictFloat<f32>>,
    /// JSON `pos_y`.
    pub pos_y: Option<StrictFloat<f32>>,
    /// JSON `pos_z`.
    pub pos_z: Option<StrictFloat<f32>>,
    /// JSON `rot_x`.
    pub rot_x: Option<StrictFloat<f32>>,
    /// JSON `rot_y`.
    pub rot_y: Option<StrictFloat<f32>>,
    /// JSON `rotation_speed`.
    pub rotation_speed: Option<StrictFloat<f32>>,
    /// JSON `snap_to_target`.
    pub snap_to_target: Option<StrictBool>,
    /// JSON `horizontal_rotation_limit`.
    pub horizontal_rotation_limit: Option<[StrictFloat<f32>; 2]>,
    /// JSON `vertical_rotation_limit`.
    pub vertical_rotation_limit: Option<[StrictFloat<f32>; 2]>,
    /// JSON `continue_targeting`.
    pub continue_targeting: Option<StrictBool>,
    /// JSON `tracking_radius`.
    pub tracking_radius: Option<StrictFloat<f32>>,
    /// JSON `view_offset`.
    pub view_offset: Option<[StrictFloat<f32>; 2]>,
    /// JSON `entity_offset`.
    pub entity_offset: Option<[StrictFloat<f32>; 3]>,
    /// JSON `radius`.
    pub radius: Option<StrictFloat<f32>>,
    /// JSON `yaw_limits_min`.
    pub yaw_limits_min: Option<StrictFloat<f32>>,
    /// JSON `yaw_limits_max`.
    pub yaw_limits_max: Option<StrictFloat<f32>>,
    /// JSON `listener`.
    pub listener: Option<AudioListener>,
    /// JSON `player_effects`.
    pub player_effects: Option<StrictBool>,
    /// JSON `aim_assist`.
    pub aim_assist: Option<PresetAimAssist>,
    /// JSON `control_scheme`.
    pub control_scheme: Option<ControlScheme>,
    /// The `format_version` the document was read at, 1.19.50 or later; the struct is
    /// written back at it (not a JSON member of the payload).
    // A skipped field still takes the JSON member of its name in facet rc.7, so the field is
    // given a NUL-prefixed name that no document has.
    #[facet(skip, rename = "\u{0}source_format_version")]
    pub source_format_version: SourceVersion,
}

impl Versioned for CameraPreset {
    const VERSION: FormatVersion = FormatVersion::new(1, 19, 50);
}

/// The step to 1.26.50: every member copied by name, `starting_rot` left unset, and
/// the flag [`apply_inherited_starting_rotation`](super::super::CameraPreset::apply_inherited_starting_rotation)
/// set, so the client keeps applying the inherited camera's starting rotation as it did before
/// 1.26.50. No finding.
impl Upgrade for CameraPreset {
    type Next = super::super::CameraPreset;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        super::super::CameraPreset {
            identifier: self.identifier,
            inherit_from: self.inherit_from,
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            pos_z: self.pos_z,
            rot_x: self.rot_x,
            rot_y: self.rot_y,
            rotation_speed: self.rotation_speed,
            snap_to_target: self.snap_to_target,
            horizontal_rotation_limit: self.horizontal_rotation_limit,
            vertical_rotation_limit: self.vertical_rotation_limit,
            continue_targeting: self.continue_targeting,
            tracking_radius: self.tracking_radius,
            view_offset: self.view_offset,
            entity_offset: self.entity_offset,
            radius: self.radius,
            yaw_limits_min: self.yaw_limits_min,
            yaw_limits_max: self.yaw_limits_max,
            listener: self.listener,
            player_effects: self.player_effects,
            aim_assist: self.aim_assist,
            control_scheme: self.control_scheme,
            starting_rot: None,
            apply_inherited_starting_rotation: true,
        }
    }
}

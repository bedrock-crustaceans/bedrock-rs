//! `minecraft:camera_preset`: a camera preset.

use bedrock_addon_core::id::{AimAssistPresetId, CameraPresetId};
use bedrock_addon_core::json::{StrictBool, StrictFloat};

/// The vanilla presets another preset may inherit from (`ALLOWED_MC_INHERIT`); they count as
/// known parents whether or not a pack of the stack defines them.
pub const ALLOWED_MC_INHERIT: &[&str] = &[
    "minecraft:free",
    "minecraft:follow_orbit",
    "minecraft:fixed_boom",
    "minecraft:third_person",
    "minecraft:third_person_front",
];

/// The vanilla cameras whose `radius` a preset may not set (`VANILLA_CAMERAS`); a preset named
/// like one of them is also checked for `starting_rot`.
pub const VANILLA_CAMERAS: &[&str] = &[
    "minecraft:first_person",
    "minecraft:third_person",
    "minecraft:third_person_front",
    "minecraft:death_camera",
];

/// A camera preset (`minecraft:camera_preset`, in `cameras/presets/`), at the newest format
/// version (1.26.50): a named set of camera settings the `/camera` command applies.
///
/// JSON: `{"format_version": "1.26.50", "minecraft:camera_preset": {"identifier": "ns:name",
/// "inherit_from": "minecraft:free", …}}`. Only `identifier` is required. The struct holds no
/// defaults: a member the document leaves out is taken from the `inherit_from` chain when the
/// client sets the camera up, and is not written back. A member vanilla does not know rejects
/// the preset at every format version.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:camera_preset")]
#[doc(alias = "camera_preset")]
pub struct CameraPreset {
    /// The preset's name, `namespace:name` (JSON `identifier`).
    pub identifier: CameraPresetId,
    /// The preset this one takes its unset members from (JSON `inherit_from`): a loaded preset
    /// or one of [`ALLOWED_MC_INHERIT`].
    pub inherit_from: Option<CameraPresetId>,
    /// The camera's x position (JSON `pos_x`).
    pub pos_x: Option<StrictFloat<f32>>,
    /// The camera's y position (JSON `pos_y`).
    pub pos_y: Option<StrictFloat<f32>>,
    /// The camera's z position (JSON `pos_z`).
    pub pos_z: Option<StrictFloat<f32>>,
    /// The pitch in degrees, -90 to 90 (JSON `rot_x`).
    pub rot_x: Option<StrictFloat<f32>>,
    /// The yaw in degrees (JSON `rot_y`).
    pub rot_y: Option<StrictFloat<f32>>,
    /// How fast the camera turns towards its target, in degrees per second, 0 to 360; 0 tracks
    /// perfectly (JSON `rotation_speed`).
    pub rotation_speed: Option<StrictFloat<f32>>,
    /// Whether the camera snaps to its target (JSON `snap_to_target`).
    pub snap_to_target: Option<StrictBool>,
    /// The horizontal rotation limits `[x, y]` in degrees, each 0 to 360 and together at most
    /// 360 (JSON `horizontal_rotation_limit`).
    pub horizontal_rotation_limit: Option<[StrictFloat<f32>; 2]>,
    /// The vertical rotation limits `[x, y]` in degrees, each 0 to 180, `x` below `y` (JSON
    /// `vertical_rotation_limit`).
    pub vertical_rotation_limit: Option<[StrictFloat<f32>; 2]>,
    /// Whether the camera keeps targeting an entity that left the tracking radius (JSON
    /// `continue_targeting`).
    pub continue_targeting: Option<StrictBool>,
    /// The radius in blocks within which the camera tracks its target; the client uses 50 when
    /// no preset of the chain sets it (JSON `tracking_radius`).
    pub tracking_radius: Option<StrictFloat<f32>>,
    /// The over-the-shoulder offset `[x, y]`, each -64 to 64 (JSON `view_offset`).
    pub view_offset: Option<[StrictFloat<f32>; 2]>,
    /// The offset `[x, y, z]` of the camera's pivot from the entity, each -64 to 64 (JSON
    /// `entity_offset`).
    pub entity_offset: Option<[StrictFloat<f32>; 3]>,
    /// The distance of the camera from the entity, 0.1 to 64 (JSON `radius`). A preset named
    /// like one of [`VANILLA_CAMERAS`] may not set it.
    pub radius: Option<StrictFloat<f32>>,
    /// The lowest yaw, -180 to 179.9 (JSON `yaw_limits_min`).
    pub yaw_limits_min: Option<StrictFloat<f32>>,
    /// The highest yaw, -179.9 to 180 (JSON `yaw_limits_max`).
    pub yaw_limits_max: Option<StrictFloat<f32>>,
    /// Where sound is heard from; the client uses the camera when no preset of the chain says
    /// (JSON `listener`).
    pub listener: Option<AudioListener>,
    /// Whether player effects (night vision, blindness, …) show through this camera; `false`
    /// when no preset of the chain says (JSON `player_effects`).
    pub player_effects: Option<StrictBool>,
    /// The aim assist the camera turns on (JSON `aim_assist`).
    pub aim_assist: Option<PresetAimAssist>,
    /// How movement input maps to the player under this camera (JSON `control_scheme`).
    pub control_scheme: Option<ControlScheme>,
    /// The starting rotation `[pitch, yaw]` (JSON `starting_rot`, format 1.26.50). Vanilla's
    /// help: "Optional. Overrides the starting camera rotation as [pitch, yaw] for a preset that
    /// inherits from a third person boom camera (such as `minecraft:follow_orbit`). When omitted,
    /// the inherited starting rotation is not applied and the player's current look direction
    /// is kept." No constraint.
    pub starting_rot: Option<[StrictFloat<f32>; 2]>,
    /// Whether the client applies the starting rotation of the inherited camera when the
    /// preset has no [`starting_rot`](Self::starting_rot). It is a member of the camera presets
    /// packet and of no JSON document (the key `apply_inherited_starting_rotation` is an unknown
    /// member). `false` for a preset read at 1.26.50 or later.
    // A skipped field still takes the JSON member of its name in facet rc.7, so the field is
    // given a NUL-prefixed name that no document has.
    #[facet(skip, rename = "\u{0}apply_inherited_starting_rotation")]
    pub apply_inherited_starting_rotation: bool,
}

impl CameraPreset {
    /// [`apply_inherited_starting_rotation`](Self::apply_inherited_starting_rotation).
    #[must_use]
    pub fn applies_inherited_starting_rotation(&self) -> bool {
        self.apply_inherited_starting_rotation
    }
}

/// Where sound is heard from under a camera (`listener`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum AudioListener {
    /// At the camera (JSON `camera`), what the client uses when nothing says.
    #[default]
    Camera,
    /// At the player (JSON `player`).
    Player,
}

/// How movement input maps to the player (`control_scheme`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ControlScheme {
    /// JSON `locked_player_relative_strafe`.
    LockedPlayerRelativeStrafe,
    /// JSON `camera_relative`.
    CameraRelative,
    /// JSON `camera_relative_strafe`.
    CameraRelativeStrafe,
    /// JSON `player_relative`.
    PlayerRelative,
    /// JSON `player_relative_strafe`.
    PlayerRelativeStrafe,
}

/// The aim assist a preset turns on (`aim_assist`); every member is optional.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct PresetAimAssist {
    /// The aim-assist preset to use (JSON `preset`).
    pub preset: Option<AimAssistPresetId>,
    /// How targets are picked (JSON `target_mode`).
    pub target_mode: Option<TargetMode>,
    /// The view angle `[x, y]` targets are picked in (JSON `angle`).
    pub angle: Option<[StrictFloat<f32>; 2]>,
    /// The distance targets are picked in (JSON `distance`).
    pub distance: Option<StrictFloat<f32>>,
}

/// How aim assist picks its target (`target_mode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum TargetMode {
    /// By angle from the view direction (JSON `angle`).
    Angle,
    /// By distance (JSON `distance`).
    Distance,
}

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

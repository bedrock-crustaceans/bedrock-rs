//! `minecraft:camera_entity`: a camera entity document, the client's definition of one of its
//! seven cameras. Vanilla reads these only from base-game and slice resource packs (`cameras/<name>.json`); an add-on pack cannot add or replace one,
//! so no stack accessor loads them. The types read vanilla's own files.

use bedrock_addon_core::schema::{self, Schema};

/// A camera entity document (`minecraft:camera_entity`, vanilla resource pack
/// `cameras/<name>.json`).
///
/// JSON: `{"format_version": "1.26.10", "minecraft:camera_entity": {"description":
/// {"identifier": "minecraft:free"}, "components": {"minecraft:camera": {"field_of_view": 66},
/// …}}}`. One struct reads every format version up to the engine's.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "minecraft:camera_entity")]
#[doc(alias = "CameraDocument")]
pub struct CameraEntity {
    /// The camera's identifier (JSON `description`).
    pub description: CameraEntityDescription,
    /// The camera's behaviours (JSON `components`).
    pub components: CameraComponents,
}

/// `minecraft:camera_entity/description`.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
pub struct CameraEntityDescription {
    /// The camera's name, `namespace:name` (JSON `identifier`).
    pub identifier: String,
}

/// A component without members: present or absent (JSON `{}`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
pub struct Present {}

/// The components of a camera entity (31 components); each is
/// optional.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[doc(alias = "CameraDefinitions")]
pub struct CameraComponents {
    /// The inside-cube renderer is used when the camera is inside a block (JSON
    /// `minecraft:allow_inside_block`).
    #[facet(rename = "minecraft:allow_inside_block")]
    pub allow_inside_block: Option<Present>,
    /// The locator the camera is attached to (JSON `minecraft:camera_attach_to_player`).
    #[facet(rename = "minecraft:camera_attach_to_player")]
    pub camera_attach_to_player: Option<AttachToPlayer>,
    /// How the camera avoids obstructions (JSON `minecraft:camera_avoidance`).
    #[facet(rename = "minecraft:camera_avoidance")]
    pub camera_avoidance: Option<Avoidance>,
    /// View bobbing (JSON `minecraft:camera_player_bob`).
    #[facet(rename = "minecraft:camera_player_bob")]
    pub camera_player_bob: Option<Present>,
    /// VR comfort movement (JSON `minecraft:vr_comfort_move`).
    #[facet(rename = "minecraft:vr_comfort_move")]
    pub vr_comfort_move: Option<Present>,
    /// The projection (JSON `minecraft:camera`).
    #[facet(rename = "minecraft:camera")]
    pub camera: Option<Camera>,
    /// The pitch limits of a directly controlled camera (JSON `minecraft:camera_direct_look`).
    #[facet(rename = "minecraft:camera_direct_look")]
    pub camera_direct_look: Option<DirectLook>,
    /// The first-person camera (JSON `minecraft:camera_first_person`).
    #[facet(rename = "minecraft:camera_first_person")]
    pub camera_first_person: Option<Present>,
    /// Free flight (JSON `minecraft:camera_fly_move`).
    #[facet(rename = "minecraft:camera_fly_move")]
    pub camera_fly_move: Option<FlyMove>,
    /// The offset while in liquid (JSON `minecraft:camera_liquid_offset`).
    #[facet(rename = "minecraft:camera_liquid_offset")]
    pub camera_liquid_offset: Option<Present>,
    /// The camera looks at the player (JSON `minecraft:camera_look_at_player`).
    #[facet(rename = "minecraft:camera_look_at_player")]
    pub camera_look_at_player: Option<Present>,
    /// The view and pivot offsets (JSON `minecraft:camera_offset`).
    #[facet(rename = "minecraft:camera_offset")]
    pub camera_offset: Option<Offset>,
    /// The camera orbits its entity (JSON `minecraft:camera_orbit`).
    #[facet(rename = "minecraft:camera_orbit")]
    pub camera_orbit: Option<Orbit>,
    /// The perspective option the camera stands for (JSON
    /// `minecraft:camera_perspective_option`).
    #[facet(rename = "minecraft:camera_perspective_option")]
    pub camera_perspective_option: Option<PerspectiveOption>,
    /// Portal distortion (JSON `minecraft:camera_portal_distortion`).
    #[facet(rename = "minecraft:camera_portal_distortion")]
    pub camera_portal_distortion: Option<Present>,
    /// First-person objects are drawn (JSON `minecraft:camera_render_first_person_objects`).
    #[facet(rename = "minecraft:camera_render_first_person_objects")]
    pub camera_render_first_person_objects: Option<Present>,
    /// The player model is drawn (JSON `minecraft:camera_render_player_model`).
    #[facet(rename = "minecraft:camera_render_player_model")]
    pub camera_render_player_model: Option<Present>,
    /// Camera shake (JSON `minecraft:camera_shake`).
    #[facet(rename = "minecraft:camera_shake")]
    pub camera_shake: Option<Shake>,
    /// The sleep vignette (JSON `minecraft:camera_player_sleep_vignette`).
    #[facet(rename = "minecraft:camera_player_sleep_vignette")]
    pub camera_player_sleep_vignette: Option<Present>,
    /// How the camera follows a target (JSON `minecraft:target`).
    #[facet(rename = "minecraft:target")]
    pub target: Option<Target>,
    /// The third-person boom (JSON `minecraft:camera_third_person_boom`).
    #[facet(rename = "minecraft:camera_third_person_boom")]
    pub camera_third_person_boom: Option<ThirdPersonBoom>,
    /// The third-person camera (JSON `minecraft:camera_third_person`).
    #[facet(rename = "minecraft:camera_third_person")]
    pub camera_third_person: Option<Present>,
    /// The fixed boom (JSON `minecraft:camera_fixed_boom`).
    #[facet(rename = "minecraft:camera_fixed_boom")]
    pub camera_fixed_boom: Option<Present>,
    /// The death camera (JSON `minecraft:death_camera_component`).
    #[facet(rename = "minecraft:death_camera_component")]
    pub death_camera_component: Option<Present>,
    /// The default input camera (JSON `minecraft:default_input_camera`).
    #[facet(rename = "minecraft:default_input_camera")]
    pub default_input_camera: Option<Present>,
    /// The player is drawn further out (JSON `minecraft:extend_player_rendering`).
    #[facet(rename = "minecraft:extend_player_rendering")]
    pub extend_player_rendering: Option<Present>,
    /// Gameplay changes the field of view (JSON `minecraft:gameplay_affects_fov`).
    #[facet(rename = "minecraft:gameplay_affects_fov")]
    pub gameplay_affects_fov: Option<Present>,
    /// The player's state changes rendering (JSON `minecraft:player_state_affects_rendering`).
    #[facet(rename = "minecraft:player_state_affects_rendering")]
    pub player_state_affects_rendering: Option<Present>,
    /// Camera input is redirected (JSON `minecraft:redirect_camera_input`).
    #[facet(rename = "minecraft:redirect_camera_input")]
    pub redirect_camera_input: Option<Present>,
    /// A camera fixed in the world (JSON `minecraft:stationary_camera`).
    #[facet(rename = "minecraft:stationary_camera")]
    pub stationary_camera: Option<Stationary>,
    /// The player turns with the camera (JSON `minecraft:update_player_from_camera`).
    #[facet(rename = "minecraft:update_player_from_camera")]
    pub update_player_from_camera: Option<UpdatePlayerFromCamera>,
}

/// `minecraft:camera_attach_to_player`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct AttachToPlayer {
    /// The locator the camera is attached to (JSON `locator`, default `""`).
    pub locator: Option<String>,
}

/// `minecraft:camera_avoidance`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Avoidance {
    /// The stiffness of the spring that answers obstructions, 0 or more; 0 adjusts at once
    /// (JSON `relax_distance_smoothing_spring`, default 4).
    #[facet(schema::fallback = 4.0)]
    pub relax_distance_smoothing_spring: Option<f32>,
    /// The least radial offset kept from the attachment point, 0 or more (JSON
    /// `distance_constraint_min`, default 0).
    #[facet(schema::fallback = 0.0)]
    pub distance_constraint_min: Option<f32>,
    /// The horizontal rays that detect objects (JSON `avoidance_rays_horizontal`).
    pub avoidance_rays_horizontal: Option<Vec<AvoidanceRay>>,
    /// The vertical rays that detect objects (JSON `avoidance_rays_vertical`).
    pub avoidance_rays_vertical: Option<Vec<AvoidanceRay>>,
}

/// One avoidance ray.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct AvoidanceRay {
    /// The ray's angle in degrees (JSON `angle`, default 0).
    #[facet(schema::fallback = 0.0)]
    pub angle: Option<f32>,
    /// The ray's weight (JSON `weight`, default 0).
    #[facet(schema::fallback = 0.0)]
    pub weight: Option<f32>,
}

/// `minecraft:camera`: the projection.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Camera {
    /// The field of view in degrees; required in a document (JSON `field_of_view`).
    pub field_of_view: Option<f32>,
    /// The distance to the near clipping plane (JSON `near_clipping_plane`, default 0.025).
    #[facet(schema::fallback = 0.025)]
    pub near_clipping_plane: Option<f32>,
    /// The distance to the far clipping plane (JSON `far_clipping_plane`, default 2500).
    #[facet(schema::fallback = 2500.0)]
    pub far_clipping_plane: Option<f32>,
}

/// `minecraft:camera_direct_look`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct DirectLook {
    /// The lowest pitch in degrees from horizontal (JSON `pitch_min`, default -90).
    #[facet(schema::fallback = -90.0)]
    pub pitch_min: Option<f32>,
    /// The highest pitch in degrees from horizontal (JSON `pitch_max`, default 90).
    #[facet(schema::fallback = 90.0)]
    pub pitch_max: Option<f32>,
}

/// `minecraft:camera_fly_move`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct FlyMove {
    /// The move speed in metres per second (JSON `speed`, default 1).
    #[facet(schema::fallback = 1.0)]
    pub speed: Option<f32>,
    /// The move speed while sprinting; unset by default (JSON `sprint_speed`).
    pub sprint_speed: Option<f32>,
}

/// `minecraft:camera_offset`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Offset {
    /// The over-the-shoulder offset `[x, y]`, each -100 to 100 (JSON `view`, default
    /// `[0, 0]`).
    pub view: Option<[f32; 2]>,
    /// The offset `[x, y, z]` of the pivot, each -100 to 100 (JSON `entity`, default
    /// `[0, 0, 0]`).
    pub entity: Option<[f32; 3]>,
}

/// `minecraft:camera_orbit`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Orbit {
    /// The azimuth spring, 0 or more (JSON `azimuth_smoothing_spring`, default 4).
    #[facet(schema::fallback = 4.0)]
    pub azimuth_smoothing_spring: Option<f32>,
    /// The polar-angle spring, 0 or more (JSON `polar_angle_smoothing_spring`, default 4).
    #[facet(schema::fallback = 4.0)]
    pub polar_angle_smoothing_spring: Option<f32>,
    /// The distance spring, 0 or more (JSON `distance_smoothing_spring`, default 4).
    #[facet(schema::fallback = 4.0)]
    pub distance_smoothing_spring: Option<f32>,
    /// The radius of the orbit, 0.1 to 64 (JSON `radius`, default 1).
    #[facet(schema::fallback = 1.0)]
    pub radius: Option<f32>,
    /// The lowest polar angle in degrees, 0 to 180 (JSON `polar_angle_min`, default 5).
    #[facet(schema::fallback = 5.0)]
    pub polar_angle_min: Option<f32>,
    /// The highest polar angle in degrees, 0 to 180 (JSON `polar_angle_max`, default 65).
    #[facet(schema::fallback = 65.0)]
    pub polar_angle_max: Option<f32>,
    /// The lowest yaw in degrees, -180 to 180 (JSON `yaw_angle_min`, default -180).
    #[facet(schema::fallback = -180.0)]
    pub yaw_angle_min: Option<f32>,
    /// The highest yaw in degrees, -180 to 180 (JSON `yaw_angle_max`, default 180).
    #[facet(schema::fallback = 180.0)]
    pub yaw_angle_max: Option<f32>,
    /// Horizontal input is inverted (JSON `invert_x_input`, default `false`).
    #[facet(schema::fallback = false)]
    pub invert_x_input: Option<bool>,
    /// Vertical input is inverted (JSON `invert_y_input`, default `false`).
    #[facet(schema::fallback = false)]
    pub invert_y_input: Option<bool>,
}

/// `minecraft:camera_perspective_option`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
pub struct PerspectiveOption {
    /// The perspective the camera stands for; required (JSON `view_mode`).
    pub view_mode: PlayerViewMode,
}

/// A perspective option (`view_mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum PlayerViewMode {
    /// JSON `first_person`.
    #[default]
    FirstPerson,
    /// JSON `third_person`.
    ThirdPerson,
    /// JSON `third_person_front`.
    ThirdPersonFront,
}

/// `minecraft:camera_shake`.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Shake {
    /// JSON `frequency`, default 0.
    #[facet(schema::fallback = 0.0)]
    pub frequency: Option<f32>,
    /// JSON `amplitude`, default 0.
    #[facet(schema::fallback = 0.0)]
    pub amplitude: Option<f32>,
    /// JSON `noise_multiplier`, default 0.
    #[facet(schema::fallback = 0.0)]
    pub noise_multiplier: Option<f32>,
}

/// `minecraft:target`: how the camera follows a
/// target.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct Target {
    /// The rotation speed in degrees per second, 0 to 360 (JSON `rotation_speed`, default
    /// 0).
    #[facet(schema::fallback = 0.0)]
    pub rotation_speed: Option<f32>,
    /// The camera snaps to its target (JSON `snap_to_target`, default `false`).
    #[facet(schema::fallback = false)]
    pub snap_to_target: Option<bool>,
    /// The horizontal limits `[x, y]`, each 0 to 360 and together at most 360; unset by
    /// default (JSON `horizontal_rotation_limit`).
    pub horizontal_rotation_limit: Option<[f32; 2]>,
    /// The vertical limits `[x, y]`, each 0 to 180; unset by default (JSON
    /// `vertical_rotation_limit`).
    pub vertical_rotation_limit: Option<[f32; 2]>,
    /// The camera keeps targeting outside the tracking radius (JSON `continue_targeting`,
    /// default `false`).
    #[facet(schema::fallback = false)]
    pub continue_targeting: Option<bool>,
    /// The tracking radius in blocks (JSON `tracking_radius`, default 50).
    #[facet(schema::fallback = 50.0)]
    pub tracking_radius: Option<f32>,
}

/// `minecraft:camera_third_person_boom`.
#[derive(Clone, Copy, Debug, PartialEq, facet::Facet, Schema)]
pub struct ThirdPersonBoom {
    /// The starting distance to the target (JSON `starting_radius`, default 0).
    pub starting_radius: Option<f32>,
    /// The starting orientation around the target (JSON `starting_rot`, default `[0, 0]`).
    pub starting_rot: Option<[f32; 2]>,
    /// The space `starting_rot` is in (JSON `rotation_space`, default `local`).
    pub rotation_space: Option<RotationSpace>,
}

/// The space of a boom's starting orientation (`rotation_space`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum RotationSpace {
    /// JSON `local`, the default.
    #[default]
    Local,
    /// JSON `world`.
    World,
}

/// `minecraft:stationary_camera`.
#[derive(Clone, Copy, Debug, PartialEq, facet::Facet, Schema)]
pub struct Stationary {
    /// The camera's world position (JSON `pos`, default `[0, 0, 0]`).
    pub pos: Option<[f32; 3]>,
    /// The camera's rotation in degrees (JSON `rot`, default `[0, 0]`).
    pub rot: Option<[f32; 2]>,
}

/// `minecraft:update_player_from_camera`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
pub struct UpdatePlayerFromCamera {
    /// Where the player looks (JSON `look_mode`, default `along_camera`).
    pub look_mode: Option<LookMode>,
}

/// Where a player turned by the camera looks (`look_mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LookMode {
    /// JSON `along_camera`, the default.
    #[default]
    AlongCamera,
    /// JSON `at_camera`.
    AtCamera,
}

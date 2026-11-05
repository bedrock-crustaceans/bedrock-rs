//! Reading and writing camera entity documents.

use super::read::{VEC2, VEC3, each_ranged, ranged};
use super::{CameraComponents, CameraEntity, texts};
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, write_payload};
use bedrock_addon_support::chain_file as file;
use bedrock_addon_support::shape::{
    self, Member, Shape, child, error, namespace_name, opt, req, violation,
};

/// A component without members.
const EMPTY: Shape = Shape::Object(&[]);

/// One avoidance ray.
const RAY: &[Member] = &[opt("angle", Shape::Float), opt("weight", Shape::Float)];
const RAYS: Shape = Shape::List(&Shape::Object(RAY));

const ATTACH: &[Member] = &[opt("locator", Shape::Str)];
const AVOIDANCE: &[Member] = &[
    opt("relax_distance_smoothing_spring", Shape::Float),
    opt("distance_constraint_min", Shape::Float),
    opt("avoidance_rays_horizontal", RAYS),
    opt("avoidance_rays_vertical", RAYS),
];
const CAMERA: &[Member] = &[
    req("field_of_view", Shape::Float),
    opt("near_clipping_plane", Shape::Float),
    opt("far_clipping_plane", Shape::Float),
];
const DIRECT_LOOK: &[Member] = &[
    opt("pitch_min", Shape::Float),
    opt("pitch_max", Shape::Float),
];
const FLY_MOVE: &[Member] = &[
    opt("speed", Shape::Float),
    opt("sprint_speed", Shape::Float),
];
const OFFSET: &[Member] = &[opt("view", VEC2), opt("entity", VEC3)];
const ORBIT: &[Member] = &[
    opt("azimuth_smoothing_spring", Shape::Float),
    opt("polar_angle_smoothing_spring", Shape::Float),
    opt("distance_smoothing_spring", Shape::Float),
    opt("radius", Shape::Float),
    opt("polar_angle_min", Shape::Float),
    opt("polar_angle_max", Shape::Float),
    opt("yaw_angle_min", Shape::Float),
    opt("yaw_angle_max", Shape::Float),
    opt("invert_x_input", Shape::Bool),
    opt("invert_y_input", Shape::Bool),
];
const PERSPECTIVE: &[Member] = &[req("view_mode", Shape::Str)];
const SHAKE: &[Member] = &[
    opt("frequency", Shape::Float),
    opt("amplitude", Shape::Float),
    opt("noise_multiplier", Shape::Float),
];
const TARGET: &[Member] = &[
    opt("rotation_speed", Shape::Float),
    opt("snap_to_target", Shape::Bool),
    opt("horizontal_rotation_limit", VEC2),
    opt("vertical_rotation_limit", VEC2),
    opt("continue_targeting", Shape::Bool),
    opt("tracking_radius", Shape::Float),
];
const BOOM: &[Member] = &[
    opt("starting_radius", Shape::Float),
    opt("starting_rot", VEC2),
    opt("rotation_space", Shape::Str),
];
const STATIONARY: &[Member] = &[opt("pos", VEC3), opt("rot", VEC2)];
const UPDATE_PLAYER: &[Member] = &[opt("look_mode", Shape::Str)];

/// The 31 components a camera entity document may hold.
const COMPONENTS: &[Member] = &[
    opt("minecraft:allow_inside_block", EMPTY),
    opt("minecraft:camera_attach_to_player", Shape::Object(ATTACH)),
    opt("minecraft:camera_avoidance", Shape::Object(AVOIDANCE)),
    opt("minecraft:camera_player_bob", EMPTY),
    opt("minecraft:vr_comfort_move", EMPTY),
    opt("minecraft:camera", Shape::Object(CAMERA)),
    opt("minecraft:camera_direct_look", Shape::Object(DIRECT_LOOK)),
    opt("minecraft:camera_first_person", EMPTY),
    opt("minecraft:camera_fly_move", Shape::Object(FLY_MOVE)),
    opt("minecraft:camera_liquid_offset", EMPTY),
    opt("minecraft:camera_look_at_player", EMPTY),
    opt("minecraft:camera_offset", Shape::Object(OFFSET)),
    opt("minecraft:camera_orbit", Shape::Object(ORBIT)),
    opt(
        "minecraft:camera_perspective_option",
        Shape::Object(PERSPECTIVE),
    ),
    opt("minecraft:camera_portal_distortion", EMPTY),
    opt("minecraft:camera_render_first_person_objects", EMPTY),
    opt("minecraft:camera_render_player_model", EMPTY),
    opt("minecraft:camera_shake", Shape::Object(SHAKE)),
    opt("minecraft:camera_player_sleep_vignette", EMPTY),
    opt("minecraft:target", Shape::Object(TARGET)),
    opt("minecraft:camera_third_person_boom", Shape::Object(BOOM)),
    opt("minecraft:camera_third_person", EMPTY),
    opt("minecraft:camera_fixed_boom", EMPTY),
    opt("minecraft:death_camera_component", EMPTY),
    opt("minecraft:default_input_camera", EMPTY),
    opt("minecraft:extend_player_rendering", EMPTY),
    opt("minecraft:gameplay_affects_fov", EMPTY),
    opt("minecraft:player_state_affects_rendering", EMPTY),
    opt("minecraft:redirect_camera_input", EMPTY),
    opt("minecraft:stationary_camera", Shape::Object(STATIONARY)),
    opt(
        "minecraft:update_player_from_camera",
        Shape::Object(UPDATE_PLAYER),
    ),
];

const DESCRIPTION: &[Member] = &[req("identifier", Shape::Str)];

/// The document: both members required.
const DOCUMENT: &[Member] = &[
    req("description", Shape::Object(DESCRIPTION)),
    req("components", Shape::Components(COMPONENTS)),
];

/// A member with a lower bound only: 0 or more. The game's text for a one-sided range is not
/// known, so the finding carries none.
fn non_negative(value: Option<f32>, at: JsonPath, d: &mut Diagnostics) -> bool {
    if value.is_some_and(|value| value < 0.0) {
        d.push(error(DiagnosticCode::InvalidValue { text: None }, at));
        return false;
    }
    true
}

impl CameraComponents {
    /// The bind constraints of the components. `false` when one failed.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) -> bool {
        let mut passed = true;
        if let Some(avoidance) = &self.camera_avoidance {
            let at = child(at, "minecraft:camera_avoidance");
            for (key, value) in [
                (
                    "relax_distance_smoothing_spring",
                    avoidance.relax_distance_smoothing_spring,
                ),
                ("distance_constraint_min", avoidance.distance_constraint_min),
            ] {
                passed &= non_negative(value, child(&at, key), d);
            }
        }
        if let Some(offset) = &self.camera_offset {
            // Each element is in [-100, 100].
            let at = child(at, "minecraft:camera_offset");
            if let Some(view) = &offset.view {
                passed &= each_ranged(view, (-100.0, 100.0), &child(&at, "view"), d);
            }
            if let Some(entity) = &offset.entity {
                passed &= each_ranged(entity, (-100.0, 100.0), &child(&at, "entity"), d);
            }
        }
        if let Some(orbit) = &self.camera_orbit {
            let at = child(at, "minecraft:camera_orbit");
            for (key, value) in [
                ("azimuth_smoothing_spring", orbit.azimuth_smoothing_spring),
                (
                    "polar_angle_smoothing_spring",
                    orbit.polar_angle_smoothing_spring,
                ),
                ("distance_smoothing_spring", orbit.distance_smoothing_spring),
            ] {
                passed &= non_negative(value, child(&at, key), d);
            }
            for (key, value, range) in [
                ("radius", orbit.radius, (f64::from(0.1_f32), 64.0)),
                ("polar_angle_min", orbit.polar_angle_min, (0.0, 180.0)),
                ("polar_angle_max", orbit.polar_angle_max, (0.0, 180.0)),
                ("yaw_angle_min", orbit.yaw_angle_min, (-180.0, 180.0)),
                ("yaw_angle_max", orbit.yaw_angle_max, (-180.0, 180.0)),
            ] {
                passed &= ranged(value, range, child(&at, key), d);
            }
        }
        if let Some(target) = &self.target {
            let at = child(at, "minecraft:target");
            passed &= ranged(
                target.rotation_speed,
                (0.0, 360.0),
                child(&at, "rotation_speed"),
                d,
            );
            if let Some(limit) = &target.horizontal_rotation_limit {
                // Each component is in [0, 360] and the sum is at most 360.
                let at = child(&at, "horizontal_rotation_limit");
                let [x, y] = *limit;
                if !(0.0..=360.0).contains(&x) || !(0.0..=360.0).contains(&y) {
                    violation(texts::TARGET_HORIZONTAL_RANGE, at, d);
                    passed = false;
                } else if x + y > 360.0 {
                    violation(texts::TARGET_HORIZONTAL_SUM, at, d);
                    passed = false;
                }
            }
            if let Some(limit) = &target.vertical_rotation_limit {
                // Elements [0, 180]; no order check here.
                let at = child(&at, "vertical_rotation_limit");
                passed &= each_ranged(limit, (0.0, 180.0), &at, d);
            }
        }
        passed
    }
}

impl VersionStruct for CameraEntity {
    /// Reads the payload (the envelope is always checked).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let entity: Self = file::typed(
            payload,
            &Shape::Object(DOCUMENT),
            None,
            shape::NOT_IN_SCHEMA,
            cx,
            d,
        )?;
        let mut passed = true;
        // The identifier must match `^\w+:\w+$`.
        if !namespace_name(&entity.description.identifier) {
            let at = child(&child(&cx.at, "description"), "identifier");
            violation(texts::ENTITY_IDENTIFIER, at, d);
            passed = false;
        }
        passed &= entity.components.check(&child(&cx.at, "components"), d);
        passed.then_some(entity)
    }

    /// The payload.
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

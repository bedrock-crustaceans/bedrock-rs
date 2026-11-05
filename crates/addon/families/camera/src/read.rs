//! Reading and writing camera presets: each struct typed from the whole document after the
//! shape check, then the bind constraints vanilla's cereal parse checks.

use super::history::v1_19_50;
use super::{CameraPreset, texts};
use super::{V1_19_50, V1_26_50};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_support::chain_file::{self as file, document};
use bedrock_addon_support::shape::{
    Member, Shape, child, element, in_range, namespace_name, opt, req, violation,
};

/// A pair of numbers.
pub(crate) const VEC2: Shape = Shape::Array(&Shape::Float, 2);
/// Three numbers.
pub(crate) const VEC3: Shape = Shape::Array(&Shape::Float, 3);

/// The members of `aim_assist`.
const PRESET_AIM_ASSIST: &[Member] = &[
    opt("preset", Shape::Str),
    opt("target_mode", Shape::Str),
    opt("angle", VEC2),
    opt("distance", Shape::Float),
];

/// The 22 members both of vanilla's slices bind, then the one of 1.26.50.
const PRESET: &[Member] = &[
    req("identifier", Shape::Str),
    opt("inherit_from", Shape::Str),
    opt("pos_x", Shape::Float),
    opt("pos_y", Shape::Float),
    opt("pos_z", Shape::Float),
    opt("rot_x", Shape::Float),
    opt("rot_y", Shape::Float),
    opt("rotation_speed", Shape::Float),
    opt("snap_to_target", Shape::Bool),
    opt("horizontal_rotation_limit", VEC2),
    opt("vertical_rotation_limit", VEC2),
    opt("continue_targeting", Shape::Bool),
    opt("tracking_radius", Shape::Float),
    opt("view_offset", VEC2),
    opt("entity_offset", VEC3),
    opt("radius", Shape::Float),
    opt("yaw_limits_min", Shape::Float),
    opt("yaw_limits_max", Shape::Float),
    opt("listener", Shape::Str),
    opt("player_effects", Shape::Bool),
    opt("aim_assist", Shape::Object(PRESET_AIM_ASSIST)),
    opt("control_scheme", Shape::Str),
    opt("starting_rot", VEC2),
];

/// How many of [`PRESET`]'s members the 1.19.50 struct has.
const PRESET_V1_19_50: usize = 22;

/// The whole preset file: both members required, the payload's members
/// those of the struct.
fn preset_file(members: &'static [Member]) -> [Member; 2] {
    [
        req("format_version", Shape::Str),
        req("minecraft:camera_preset", Shape::Object(members)),
    ]
}

/// The members of [`PRESET`] the 1.19.50 struct binds.
fn preset_v1_19_50() -> &'static [Member] {
    PRESET.get(..PRESET_V1_19_50).unwrap_or(PRESET)
}

/// The members of a preset that carry a bind constraint; the same in both structs.
struct Constrained<'a> {
    identifier: &'a str,
    inherit_from: Option<&'a str>,
    rot_x: Option<f32>,
    rotation_speed: Option<f32>,
    horizontal_rotation_limit: Option<[f32; 2]>,
    vertical_rotation_limit: Option<[f32; 2]>,
    view_offset: Option<[f32; 2]>,
    entity_offset: Option<[f32; 3]>,
    radius: Option<f32>,
    yaw_limits_min: Option<f32>,
    yaw_limits_max: Option<f32>,
}

/// The constrained members of a preset struct.
macro_rules! constrained {
    ($preset:expr) => {
        Constrained {
            identifier: $preset.identifier.as_str(),
            inherit_from: $preset.inherit_from.as_ref().map(|id| id.as_str()),
            rot_x: $preset.rot_x,
            rotation_speed: $preset.rotation_speed,
            horizontal_rotation_limit: $preset.horizontal_rotation_limit,
            vertical_rotation_limit: $preset.vertical_rotation_limit,
            view_offset: $preset.view_offset,
            entity_offset: $preset.entity_offset,
            radius: $preset.radius,
            yaw_limits_min: $preset.yaw_limits_min,
            yaw_limits_max: $preset.yaw_limits_max,
        }
    };
}

/// `value` in `[min, max]`, when it is set.
pub(crate) fn ranged(
    value: Option<f32>,
    (min, max): (f64, f64),
    at: JsonPath,
    d: &mut Diagnostics,
) -> bool {
    value.is_none_or(|value| in_range(f64::from(value), min, max, at, d))
}

/// Each element of `values` in `[min, max]`: the first one outside is reported.
pub(crate) fn each_ranged(
    values: &[f32],
    (min, max): (f64, f64),
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, value)| in_range(f64::from(*value), min, max, element(at, i), d))
}

impl Constrained<'_> {
    /// The bind constraints of a preset, in bind order; a violation is a constraint violation
    /// of vanilla's parser, fatal on both slices. `false` when one failed.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) -> bool {
        let mut passed = true;
        // `^(\w+:\w+)?$`: the empty string passes.
        for (key, id) in [
            ("identifier", Some(self.identifier)),
            ("inherit_from", self.inherit_from),
        ] {
            if id.is_some_and(|id| !id.is_empty() && !namespace_name(id)) {
                violation(texts::IDENTIFIER, child(at, key), d);
                passed = false;
            }
        }
        passed &= ranged(self.rot_x, (-90.0, 90.0), child(at, "rot_x"), d);
        passed &= ranged(
            self.rotation_speed,
            (0.0, 360.0),
            child(at, "rotation_speed"),
            d,
        );
        if let Some(limit) = &self.horizontal_rotation_limit {
            passed &= horizontal_limit(*limit, &child(at, "horizontal_rotation_limit"), d);
        }
        if let Some(limit) = &self.vertical_rotation_limit {
            passed &= vertical_limit(*limit, &child(at, "vertical_rotation_limit"), d);
        }
        if let Some(offset) = &self.view_offset {
            passed &= each_ranged(offset, (-64.0, 64.0), &child(at, "view_offset"), d);
        }
        if let Some(offset) = &self.entity_offset {
            passed &= each_ranged(offset, (-64.0, 64.0), &child(at, "entity_offset"), d);
        }
        // The lower bound is the `f32` 0.1 widened to a double.
        passed &= ranged(
            self.radius,
            (f64::from(0.1_f32), 64.0),
            child(at, "radius"),
            d,
        );
        passed &= ranged(
            self.yaw_limits_min,
            (-180.0, 179.9),
            child(at, "yaw_limits_min"),
            d,
        );
        passed &= ranged(
            self.yaw_limits_max,
            (-179.9, 180.0),
            child(at, "yaw_limits_max"),
            d,
        );
        passed
    }
}

/// Each component in `[0, 360]` and the sum at most 360. The constraints are checked in bind
/// order and the first one that fails is the one reported; each rejects the preset either way.
fn horizontal_limit(limit: [f32; 2], at: &JsonPath, d: &mut Diagnostics) -> bool {
    if !each_ranged(&limit, (0.0, 360.0), at, d) {
        return false;
    }
    if limit[0] + limit[1] > 360.0 {
        violation(texts::VECTOR_TOTAL, at.clone(), d);
        return false;
    }
    true
}

/// The components differ, `x` is below `y`, and each is in `[0, 180]`; there is no sum
/// constraint. As for the horizontal limit, the first failing constraint is the one reported.
fn vertical_limit(limit: [f32; 2], at: &JsonPath, d: &mut Diagnostics) -> bool {
    let [x, y] = limit;
    #[allow(
        clippy::float_cmp,
        reason = "vanilla compares the two components for equality"
    )]
    if x == y {
        violation(texts::VECTOR_EQUAL, at.clone(), d);
        return false;
    }
    if x >= y {
        violation(texts::VECTOR_ORDER, at.clone(), d);
        return false;
    }
    each_ranged(&limit, (0.0, 180.0), at, d)
}

impl VersionStruct for v1_19_50::CameraPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        static FILE: std::sync::OnceLock<[Member; 2]> = std::sync::OnceLock::new();
        let members = FILE.get_or_init(|| preset_file(preset_v1_19_50()));
        // The legacy slice's reader is the non-strict one.
        let mut preset: Self = file::typed_unstrict(
            text,
            &Shape::Object(members),
            Some("minecraft:camera_preset"),
            texts::UNRECOGNIZED_FIELD,
            cx,
            d,
        )?;
        preset.source_format_version = cx.original_version.into();
        constrained!(preset)
            .check(&child(&cx.at, "minecraft:camera_preset"), d)
            .then_some(preset)
    }

    /// The whole document, at the version it was read at: the struct covers a range of
    /// versions, so writing a fixed one would change the `format_version` of a document that
    /// is read and written without an upgrade.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            &self
                .source_format_version
                .version()
                .max(Self::VERSION)
                .to_string(),
            "minecraft:camera_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for CameraPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        static FILE: std::sync::OnceLock<[Member; 2]> = std::sync::OnceLock::new();
        let members = FILE.get_or_init(|| preset_file(PRESET));
        let preset: Self = file::typed(
            text,
            &Shape::Object(members),
            Some("minecraft:camera_preset"),
            texts::UNRECOGNIZED_FIELD,
            cx,
            d,
        )?;
        constrained!(preset)
            .check(&child(&cx.at, "minecraft:camera_preset"), d)
            .then_some(preset)
    }

    /// The whole document, at format 1.26.50; at 1.19.50 when the preset applies the inherited
    /// starting rotation and has no `starting_rot`, the one value 1.26.50 cannot spell: the
    /// legacy slice reads it back and the step sets the flag again. With a `starting_rot` the
    /// flag decides nothing at the client, which tests the two together, and the 1.26.50 form
    /// is written.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        let version = if self.apply_inherited_starting_rotation && self.starting_rot.is_none() {
            V1_19_50
        } else {
            V1_26_50
        };
        Ok(document(
            &version.to_string(),
            "minecraft:camera_preset",
            write_payload(self)?,
        ))
    }
}

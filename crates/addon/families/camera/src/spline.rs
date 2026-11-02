//! `minecraft:camera_custom_splines`: camera splines.

use bedrock_addon_core::json::{StrictFloat, StrictString};

/// A camera spline file (`minecraft:camera_custom_splines`, in `cameras/splines/`), format
/// 1.26.0: the paths the `/camera … play_spline` command moves a camera along.
///
/// JSON: `{"format_version": "1.26.0", "minecraft:camera_custom_splines": {"splines":
/// [spline, …]}}`. Any failure rejects the whole file. Vanilla keeps every spline of every
/// file in load order and does not look names up while loading, so two splines of one name
/// both stay.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:camera_custom_splines")]
#[doc(alias = "camera_spline")]
pub struct CameraSpline {
    /// The splines of the file, in order (JSON `splines`).
    pub splines: Vec<Spline>,
}

/// One spline.
///
/// JSON: `{"name": "ns:name", "total_time": 4.0, "spline_type"?: "catmullrom",
/// "control_points": [{"position": [x, y, z]}, …], "progress_key_frames": [{"progress": 0.5,
/// "time": 2.0, "easing"?: "linear"}, …], "rotation_key_frames": [{"rotation": [x, y, z],
/// "time": 2.0, "easing"?: "linear"}, …]}`.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[doc(alias = "CameraSplineDefinition")]
pub struct Spline {
    /// The spline's name, `namespace:name` (JSON `name`).
    pub name: StrictString,
    /// How long the spline plays, in seconds, 0 or more (JSON `total_time`).
    pub total_time: StrictFloat<f32>,
    /// How the control points are joined (JSON `spline_type`, default `catmullrom`: see
    /// [`Spline::spline_type`]).
    pub spline_type: Option<SplineType>,
    /// The points the path runs through, in world space (JSON `control_points`).
    pub control_points: Vec<ControlPoint>,
    /// How far along the path the camera is at given times (JSON `progress_key_frames`).
    pub progress_key_frames: Vec<ProgressKeyFrame>,
    /// The camera's rotation at given times (JSON `rotation_key_frames`).
    pub rotation_key_frames: Vec<RotationKeyFrame>,
}

impl Spline {
    /// [`spline_type`](Self::spline_type), `catmullrom` when the document leaves it out.
    #[must_use]
    pub fn spline_type(&self) -> SplineType {
        self.spline_type.unwrap_or_default()
    }
}

/// How a spline joins its control points (`spline_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum SplineType {
    /// A Catmull-Rom curve through the points (JSON `catmullrom`), the default.
    #[default]
    Catmullrom,
    /// Straight segments (JSON `linear`).
    Linear,
}

/// One control point of a spline (`control_points[]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet)]
pub struct ControlPoint {
    /// The point `[x, y, z]` (JSON `position`).
    pub position: [StrictFloat<f32>; 3],
}

/// One progress key frame of a spline (`progress_key_frames[]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet)]
pub struct ProgressKeyFrame {
    /// The fraction of the path covered at `time`, 0 to 1 (JSON `progress`).
    pub progress: StrictFloat<f32>,
    /// The time in seconds, 0 or more (JSON `time`).
    pub time: StrictFloat<f32>,
    /// How the progress is interpolated towards this frame; linear at playback when left out
    /// (JSON `easing`).
    pub easing: Option<Easing>,
}

/// One rotation key frame of a spline (`rotation_key_frames[]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet)]
pub struct RotationKeyFrame {
    /// The rotation `[x, y, z]` at `time` (JSON `rotation`).
    pub rotation: [StrictFloat<f32>; 3],
    /// The time in seconds, 0 or more (JSON `time`).
    pub time: StrictFloat<f32>,
    /// How the rotation is interpolated towards this frame; linear at playback when left out
    /// (JSON `easing`).
    pub easing: Option<Easing>,
}

/// An easing function (the `easing_function` type of the game's camera documents).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "each variant is the easing function of its name"
)]
pub enum Easing {
    #[default]
    Linear,
    Spring,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InSine,
    OutSine,
    InOutSine,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBounce,
    OutBounce,
    InOutBounce,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
}

impl Easing {
    /// Every easing function's JSON name, in vanilla's order.
    pub const NAMES: &'static [&'static str] = &[
        "linear",
        "spring",
        "in_quad",
        "out_quad",
        "in_out_quad",
        "in_cubic",
        "out_cubic",
        "in_out_cubic",
        "in_quart",
        "out_quart",
        "in_out_quart",
        "in_quint",
        "out_quint",
        "in_out_quint",
        "in_sine",
        "out_sine",
        "in_out_sine",
        "in_expo",
        "out_expo",
        "in_out_expo",
        "in_circ",
        "out_circ",
        "in_out_circ",
        "in_bounce",
        "out_bounce",
        "in_out_bounce",
        "in_back",
        "out_back",
        "in_out_back",
        "in_elastic",
        "out_elastic",
        "in_out_elastic",
    ];

    /// This easing function's JSON name, the entry of [`NAMES`](Self::NAMES) at its position.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

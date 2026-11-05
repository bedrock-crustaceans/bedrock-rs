//! `minecraft:camera_custom_splines`: camera splines.

use bedrock_addon_core::schema::{self, Schema};

/// A camera spline file (`minecraft:camera_custom_splines`, in `cameras/splines/`), format
/// 1.26.0: the paths the `/camera … play_spline` command moves a camera along.
///
/// JSON: `{"format_version": "1.26.0", "minecraft:camera_custom_splines": {"splines":
/// [spline, …]}}`. Any failure rejects the whole file. Vanilla keeps every spline of every
/// file in load order and does not look names up while loading, so two splines of one name
/// both stay.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
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
#[derive(Clone, Debug, Default, PartialEq, facet::Facet, Schema)]
#[doc(alias = "CameraSplineDefinition")]
pub struct Spline {
    /// The spline's name, `namespace:name` (JSON `name`).
    pub name: String,
    /// How long the spline plays, in seconds, 0 or more (JSON `total_time`).
    pub total_time: f32,
    /// How the control points are joined (JSON `spline_type`, default `catmullrom`: see
    /// [`Spline::spline_type`]).
    #[facet(schema::fallback = SplineType::Catmullrom)]
    pub spline_type: Option<SplineType>,
    /// The points the path runs through, in world space (JSON `control_points`).
    pub control_points: Vec<ControlPoint>,
    /// How far along the path the camera is at given times (JSON `progress_key_frames`).
    pub progress_key_frames: Vec<ProgressKeyFrame>,
    /// The camera's rotation at given times (JSON `rotation_key_frames`).
    pub rotation_key_frames: Vec<RotationKeyFrame>,
}

/// How a spline joins its control points (`spline_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet, Schema)]
pub struct ControlPoint {
    /// The point `[x, y, z]` (JSON `position`).
    pub position: [f32; 3],
}

/// One progress key frame of a spline (`progress_key_frames[]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet, Schema)]
pub struct ProgressKeyFrame {
    /// The fraction of the path covered at `time`, 0 to 1 (JSON `progress`).
    pub progress: f32,
    /// The time in seconds, 0 or more (JSON `time`).
    pub time: f32,
    /// How the progress is interpolated towards this frame; linear at playback when left out
    /// (JSON `easing`).
    pub easing: Option<Easing>,
}

/// One rotation key frame of a spline (`rotation_key_frames[]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet, Schema)]
pub struct RotationKeyFrame {
    /// The rotation `[x, y, z]` at `time` (JSON `rotation`).
    pub rotation: [f32; 3],
    /// The time in seconds, 0 or more (JSON `time`).
    pub time: f32,
    /// How the rotation is interpolated towards this frame; linear at playback when left out
    /// (JSON `easing`).
    pub easing: Option<Easing>,
}

/// An easing function (the `easing_function` type of the game's camera documents).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
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

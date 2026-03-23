use crate::version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraSplineInstruction<V: ProtoVersion> {
    #[endianness(le)]
    pub total_time: f32,
    pub spline_type: V::CameraSplineType,
    #[endianness(le)]
    pub curve: Vec<(f32, f32, f32)>,
    pub progress_key_frames: Vec<ProgressKeyFrame<V>>,
    pub rotation_option: Vec<RotationOption<V>>,
    pub spline_identifier: Option<String>,
    pub load_from_json: Option<bool>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ProgressKeyFrame<V: ProtoVersion> {
    #[endianness(le)]
    pub value: f32,
    #[endianness(le)]
    pub time: f32,
    pub ease_type: V::CameraSplineEaseType,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct RotationOption<V: ProtoVersion> {
    #[endianness(le)]
    pub key_frame_values: (f32, f32, f32),
    #[endianness(le)]
    pub key_frame_times: f32,
    pub ease_type: V::CameraSplineEaseType,
}

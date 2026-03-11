use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use vek::Vec3;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraSplineInstruction<V: ProtoVersion> {
    #[endianness(le)]
    pub total_time: f32,
    pub spline_type: V::CameraSplineType,
    #[endianness(le)]
    pub curve: Vec<Vec3<f32>>,
    pub progress_key_frames: Vec<ProgressKeyFrame<V>>,
    pub rotation_option: Vec<RotationOption>,
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
pub struct RotationOption {
    #[endianness(le)]
    pub key_frame_values: Vec3<f32>,
    #[endianness(le)]
    pub key_frame_times: f32,
}

use crate::version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use vek::{Vec2, Vec3};

#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugShape<V: ProtoVersion> {
    #[endianness(var)]
    pub id: u64,
    pub debug_shape_type: Option<DebugShapeType>,
    #[endianness(le)]
    pub position: Option<Vec3<f32>>,
    #[endianness(le)]
    pub scale: Option<f32>,
    #[endianness(le)]
    pub rotation: Option<Vec2<f32>>,
    #[endianness(le)]
    pub remaining_duration: Option<f32>,
    pub color: Option<V::Color>,
    pub text: Option<String>,
    #[endianness(le)]
    pub box_bounds: Option<Vec3<f32>>,
    #[endianness(le)]
    pub line_end_position: Option<Vec3<f32>>,
    #[endianness(le)]
    pub arrow_head_length: Option<f32>,
    #[endianness(le)]
    pub arrow_head_radius: Option<f32>,
    pub segments: Option<i8>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum DebugShapeType {
    Line = 0,
    Box = 1,
    Sphere = 2,
    Circle = 3,
    Text = 4,
    Arrow = 5,
}

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
    pub shape_data: DebugShapeData,
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

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum DebugShapeData {
    Last = 0,
    Arrow {
        #[endianness(le)]
        arrow_end_position: Option<Vec3<f32>>,
        #[endianness(le)]
        arrow_head_length: Option<f32>,
        #[endianness(le)]
        arrow_head_radius: Option<f32>,
        arrow_head_segments: Option<i8>,
    } = 1,
    Text {
        text: Option<String>,
    } = 2,
    Box {
        #[endianness(le)]
        box_bounds: Option<Vec3<f32>>,
    } = 3,
    Line {
        #[endianness(le)]
        line_end_position: Option<Vec3<f32>>,
    } = 4,
    Sphere {
        segments: Option<i8>,
    } = 5,
}

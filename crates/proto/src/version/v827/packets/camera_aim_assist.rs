use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use vek::Vec2;

#[packet(id = 316)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPacket<V: ProtoVersion> {
    pub preset_id: String,
    #[endianness(le)]
    pub view_angle: Vec2<f32>,
    #[endianness(le)]
    pub distance: f32,
    pub target_mode: TargetMode,
    pub action: V::AimAssistAction,
    pub show_debug_render: bool,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum TargetMode {
    Angle = 0,
    Distance = 1,
}

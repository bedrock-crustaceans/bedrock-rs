use vek::Vec2;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use super::super::enums::AimAssistAction;

#[gamepacket(id = 316)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPacket {
    pub preset_id: String,
    #[endianness(le)]
    pub view_angle: Vec2<f32>,
    #[endianness(le)]
    pub distance: f32,
    pub target_mode: TargetMode,
    pub action: AimAssistAction,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum TargetMode {
    Angle = 0,
    Distance = 1,
}
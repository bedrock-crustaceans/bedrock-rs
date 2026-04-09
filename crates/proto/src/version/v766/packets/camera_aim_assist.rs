use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 316)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPacket<V: ProtoVersion> {
    pub preset_id: String,
    #[endianness(le)]
    pub view_angle: (f32, f32),
    #[endianness(le)]
    pub distance: f32,
    pub target_mode: TargetMode,
    pub action: V::AimAssistAction,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum TargetMode {
    Angle = 0,
    Distance = 1,
}

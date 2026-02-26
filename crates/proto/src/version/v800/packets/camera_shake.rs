use super::super::enums::{CameraShakeAction, CameraShakeType};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 159)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraShakePacket {
    #[endianness(le)]
    pub intensity: f32,
    #[endianness(le)]
    pub seconds: f32,
    pub shake_type: CameraShakeType,
    pub shake_action: CameraShakeAction,
}
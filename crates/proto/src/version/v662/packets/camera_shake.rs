use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 159)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraShakePacket<V: ProtoVersion> {
    #[endianness(le)]
    pub intensity: f32,
    #[endianness(le)]
    pub seconds: f32,
    pub shake_type: V::CameraShakeType,
    pub shake_action: V::CameraShakeAction,
}

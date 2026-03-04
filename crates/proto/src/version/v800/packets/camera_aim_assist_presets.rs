use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 320)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub category_definitions: Vec<V::CameraAimAssistCategory>,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
    pub operation: V::CameraAimAssistOperation,
}

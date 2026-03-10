use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 320)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub categories: Vec<V::CameraAimAssistCategories>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
}

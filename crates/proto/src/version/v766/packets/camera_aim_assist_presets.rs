use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 320)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub categories: Vec<V::CameraAimAssistCategories>,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
}

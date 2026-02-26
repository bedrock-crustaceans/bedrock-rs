use super::super::types::CameraPreset;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresets {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub presets: Vec<CameraPreset>,
}
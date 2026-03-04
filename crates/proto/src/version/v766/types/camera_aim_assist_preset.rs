use bedrockrs_macros::ProtoCodec;
use vek::Vec2;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPreset {
    pub identifier: Option<String>,
    #[endianness(le)]
    pub target_mode: Option<i32>,
    #[endianness(le)]
    pub angle: Option<Vec2<f32>>,
    #[endianness(le)]
    pub distance: Option<f32>,
}

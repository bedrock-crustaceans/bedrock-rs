use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistItemSettings {
    pub item_id: String,
    pub category: String,
}
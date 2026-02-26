use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPriority {
    pub name: String,
    #[endianness(le)]
    pub priority: i32,
}
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 339)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistActorPriorityPacket {
    pub priority_data: Vec<CameraAimAssistActorPriorityData>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistActorPriorityData {
    #[endianness(le)]
    pub preset_index: i32,
    #[endianness(le)]
    pub category_index: i32,
    #[endianness(le)]
    pub actor_index: i32,
    #[endianness(le)]
    pub priority_value: i32,
}

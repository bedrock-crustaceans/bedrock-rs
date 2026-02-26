use super::super::types::MoveActorAbsoluteData;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 18)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsolutePacket {
    pub move_data: MoveActorAbsoluteData,
}
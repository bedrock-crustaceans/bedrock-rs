use super::super::enums::PlayerActionType;
use super::super::types::{ActorRuntimeID, NetworkBlockPosition};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 36)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerActionPacket {
    pub player_runtime_id: ActorRuntimeID,
    pub action: PlayerActionType,
    pub block_position: NetworkBlockPosition,
    pub result_pos: NetworkBlockPosition,
    #[endianness(var)]
    pub face: i32,
}

// TODO: PlayerActionType is has enum variants, but this packet doesn't serialize them. Might require moving the variants into their specific type
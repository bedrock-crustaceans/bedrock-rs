use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 36)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerActionPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    pub action: V::PlayerActionType,
    pub block_position: V::NetworkBlockPosition,
    pub result_pos: V::NetworkBlockPosition,
    #[endianness(var)]
    pub face: i32,
}

// TODO: V::PlayerActionType is has enum variants, but this packet doesn't serialize them. Might require moving the variants into their specific type
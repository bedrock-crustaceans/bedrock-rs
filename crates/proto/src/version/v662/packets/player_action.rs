use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use crate::v662::enums::PlayerActionType;

#[packet(id = 36)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerActionPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    pub action: PlayerActionType,
    pub block_position: V::NetworkBlockPosition,
    pub result_pos: V::NetworkBlockPosition,
    #[endianness(var)]
    pub face: i32,
}

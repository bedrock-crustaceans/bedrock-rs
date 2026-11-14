use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};
use crate::enums::PlayerActionType;

#[packet(id = 36, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerActionPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    pub action: PlayerActionType,
    pub block_position: V::NetworkBlockPosition,
    pub result_pos: V::NetworkBlockPosition,
    #[endianness(var)]
    pub face: i32,
}

use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 62)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
}
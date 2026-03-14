use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 62)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
}

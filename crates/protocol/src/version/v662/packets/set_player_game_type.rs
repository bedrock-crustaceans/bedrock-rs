use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 62, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
}

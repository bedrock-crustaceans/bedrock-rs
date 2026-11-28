use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 62, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
}

use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 151, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdatePlayerGameTypePacket<V: ProtoVersion> {
    pub player_game_type: V::GameType,
    pub target_player: V::ActorUniqueID,
}

use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 105, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDefaultGameTypePacket<V: ProtoVersion> {
    pub default_game_type: V::GameType,
}

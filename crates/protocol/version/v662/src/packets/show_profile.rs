use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 104, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowProfilePacket {
    pub player_xuid: String,
}

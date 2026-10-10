use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 160, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerFogPacket {
    pub fog_stack: Vec<String>,
}

use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 333, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIShowScreenPacket {
    pub screen_id: String,
}

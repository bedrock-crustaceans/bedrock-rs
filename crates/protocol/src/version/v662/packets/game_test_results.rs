use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 195, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameTestResultsPacket {
    pub succeeded: bool,
    pub error: String,
    pub test_name: String,
}

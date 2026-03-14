use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 195)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameTestResultsPacket {
    pub succeeded: bool,
    pub error: String,
    pub test_name: String,
}

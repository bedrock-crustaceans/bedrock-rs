use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 195)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameTestResultsPacket {
    pub succeeded: bool,
    pub error: String,
    pub test_name: String,
}
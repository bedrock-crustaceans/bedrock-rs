use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 60, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDifficultyPacket {
    #[endianness(var)]
    pub difficulty: u32,
}

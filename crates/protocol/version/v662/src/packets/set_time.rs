use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 10, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetTimePacket {
    #[endianness(var)]
    pub time: i32,
}

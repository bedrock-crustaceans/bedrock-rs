use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 84)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackChunkRequestPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk: u32,
}

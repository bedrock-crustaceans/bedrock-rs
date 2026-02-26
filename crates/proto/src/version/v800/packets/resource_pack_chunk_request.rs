use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 84)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackChunkRequestPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk: u32,
}
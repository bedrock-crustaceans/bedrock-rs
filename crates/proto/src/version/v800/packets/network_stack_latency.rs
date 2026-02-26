use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 115)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkStackLatencyPacket {
    #[endianness(le)]
    pub creation_time: u64,
    pub is_from_server: bool,
}
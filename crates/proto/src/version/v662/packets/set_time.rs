use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 10)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetTimePacket {
    #[endianness(var)]
    pub time: i32,
}

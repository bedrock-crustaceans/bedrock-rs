use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 42)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHealthPacket {
    #[endianness(var)]
    pub health: i32,
}

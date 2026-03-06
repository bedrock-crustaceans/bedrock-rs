use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 42)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHealthPacket {
    #[endianness(var)]
    pub health: i32,
}

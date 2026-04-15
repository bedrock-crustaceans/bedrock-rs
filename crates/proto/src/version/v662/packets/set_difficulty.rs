use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 60)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDifficultyPacket {
    #[endianness(var)]
    pub difficulty: u32,
}

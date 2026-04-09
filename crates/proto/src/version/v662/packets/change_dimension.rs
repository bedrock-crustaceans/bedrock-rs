use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 61)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ChangeDimensionPacket {
    #[endianness(var)]
    pub dimension_id: i32,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    pub respawn: bool,
}

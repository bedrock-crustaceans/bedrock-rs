use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 16)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPlayerPostMovePositionPacket {
    #[endianness(le)]
    pub pos: (f32, f32, f32),
}

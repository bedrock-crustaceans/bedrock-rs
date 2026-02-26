use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 16)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPlayerPostMovePositionPacket {
    #[endianness(le)]
    pub pos: Vec3<f32>,
}
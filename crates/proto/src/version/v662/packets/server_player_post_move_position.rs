use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 16)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerPlayerPostMovePositionPacket {
    #[endianness(le)]
    pub pos: Vec3<f32>,
}

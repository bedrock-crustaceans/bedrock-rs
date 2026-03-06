use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec2;

#[packet(id = 57)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerInputPacket {
    #[endianness(le)]
    pub move_vector: Vec2<f32>,
    pub jumping: bool,
    pub sneaking: bool,
}

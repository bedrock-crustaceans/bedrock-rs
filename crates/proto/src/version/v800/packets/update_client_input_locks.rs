use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 196)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateClientInputLocksPacket {
    #[endianness(var)]
    pub input_lock_component_data: i32,
    #[endianness(le)]
    pub server_pos: Vec3<f32>,
}
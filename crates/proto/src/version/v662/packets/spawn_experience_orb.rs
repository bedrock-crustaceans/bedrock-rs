use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 66)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnExperienceOrbPacket {
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub xp_value: i32,
}

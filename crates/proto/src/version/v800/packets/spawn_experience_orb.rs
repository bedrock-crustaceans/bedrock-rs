use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 66)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnExperienceOrbPacket {
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub xp_value: i32,
}
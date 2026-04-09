use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 66)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnExperienceOrbPacket {
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub xp_value: i32,
}

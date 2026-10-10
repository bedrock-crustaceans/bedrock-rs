use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 66, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnExperienceOrbPacket {
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub xp_value: i32,
}

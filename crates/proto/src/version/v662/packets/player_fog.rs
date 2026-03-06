use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 160)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerFogPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub fog_stack: Vec<String>,
}

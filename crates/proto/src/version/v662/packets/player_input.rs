use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 57)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerInputPacket {
    #[endianness(le)]
    pub move_vector: (f32, f32),
    pub jumping: bool,
    pub sneaking: bool,
}

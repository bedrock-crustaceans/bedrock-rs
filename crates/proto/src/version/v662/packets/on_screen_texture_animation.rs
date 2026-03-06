use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 130)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OnScreenTextureAnimationPacket {
    #[endianness(le)]
    pub effect_id: u32,
}

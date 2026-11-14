use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 130, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OnScreenTextureAnimationPacket {
    #[endianness(le)]
    pub effect_id: u32,
}

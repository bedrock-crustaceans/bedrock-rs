use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 160)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerFogPacket {
    pub fog_stack: Vec<String>,
}

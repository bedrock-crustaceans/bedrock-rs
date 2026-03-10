use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 160)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerFogPacket {
    pub fog_stack: Vec<String>,
}

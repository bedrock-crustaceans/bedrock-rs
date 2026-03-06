use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 104)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ShowProfilePacket {
    pub player_xuid: String,
}

use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 163)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct FilterTextPacket {
    pub text: String,
    pub from_server: bool,
}

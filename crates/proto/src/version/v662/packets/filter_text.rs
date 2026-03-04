use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 163)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct FilterTextPacket {
    pub text: String,
    pub from_server: bool,
}

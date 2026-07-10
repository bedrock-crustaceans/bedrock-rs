use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 350)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PartyDestinationCookieResponsePacket {
    pub cookie: String,
    pub accepted: bool,
}

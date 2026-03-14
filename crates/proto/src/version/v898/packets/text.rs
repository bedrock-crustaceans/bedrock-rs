use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 9)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TextPacket<V: ProtoVersion> {
    pub localize: bool,
    pub message_type: V::TextPacketType,
    pub sender_xuid: String,
    pub platform_id: String,
    pub filtered_message: Option<String>,
}

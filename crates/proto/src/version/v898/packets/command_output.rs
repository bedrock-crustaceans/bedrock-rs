use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 79)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandOutputPacket<V: ProtoVersion> {
    pub origin_data: V::CommandOriginData,
    pub output_type: V::CommandOutputType,
    #[endianness(le)]
    pub success_count: u32,
    pub output_messages: Vec<OutputMessagesEntry>,
    pub data: Option<String>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct OutputMessagesEntry {
    pub message_id: String,
    pub internal: bool,
    pub parameters: Vec<String>,
}

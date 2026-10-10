use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 77, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket<V: ProtoVersion> {
    pub command: String,
    pub command_origin: V::CommandOriginData,
    pub internal: bool,
    pub version: String,
}

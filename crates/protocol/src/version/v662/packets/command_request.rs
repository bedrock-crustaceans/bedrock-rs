use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 77, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket<V: ProtoVersion> {
    pub command: String,
    pub command_origin: V::CommandOriginData,
    pub internal: bool,
    #[endianness(var)]
    pub version: i32,
}

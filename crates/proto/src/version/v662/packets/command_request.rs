use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 77)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket<V: ProtoVersion> {
    pub command: String,
    pub command_origin: V::CommandOriginData,
    pub internal: bool,
    #[endianness(var)]
    pub version: i32,
}

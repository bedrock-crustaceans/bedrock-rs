use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 77)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket<V: ProtoVersion> {
    pub command: String,
    pub command_origin: V::CommandOriginData,
    pub internal: bool,
    pub version: String,
}

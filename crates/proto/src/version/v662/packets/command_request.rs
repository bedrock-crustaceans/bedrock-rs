use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 77)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket<V: ProtoVersion> {
    pub command: String,
    pub command_origin: V::CommandOriginData,
    pub internal: bool,
    #[endianness(var)]
    pub version: i32,
}

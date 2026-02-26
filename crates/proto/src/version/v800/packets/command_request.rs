use super::super::types::CommandOriginData;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 77)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandRequestPacket {
    pub command: String,
    pub command_origin: CommandOriginData,
    pub is_internal_source: bool,
    #[endianness(var)]
    pub version: i32,
}
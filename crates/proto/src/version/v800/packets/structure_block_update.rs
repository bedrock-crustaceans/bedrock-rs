use super::super::types::{NetworkBlockPosition, StructureEditorData};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 90)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureBlockUpdatePacket {
    pub block_position: NetworkBlockPosition,
    pub structure_data: StructureEditorData,
    pub trigger: bool,
    pub is_waterlogged: bool,
}
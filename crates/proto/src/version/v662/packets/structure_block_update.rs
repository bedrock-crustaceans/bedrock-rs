use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 90)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureBlockUpdatePacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    pub structure_data: V::StructureEditorData,
    pub trigger: bool,
    pub is_waterlogged: bool,
}
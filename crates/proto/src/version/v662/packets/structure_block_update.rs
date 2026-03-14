use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 90)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureBlockUpdatePacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    pub structure_data: V::StructureEditorData,
    pub trigger: bool,
    pub is_waterlogged: bool,
}

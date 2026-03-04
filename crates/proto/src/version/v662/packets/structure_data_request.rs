use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 132)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureDataRequestPacket<V: ProtoVersion> {
    pub structure_name: String,
    pub structure_position: V::NetworkBlockPosition,
    pub structure_settings: V::StructureSettings,
    pub requested_operation: V::StructureTemplateRequestOperation,
}
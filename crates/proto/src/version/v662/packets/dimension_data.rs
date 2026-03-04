use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 180)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDataPacket<V: ProtoVersion> {
    pub dimension_definition_group: V::DimensionDefinitionGroup,
}
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 180)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDataPacket<V: ProtoVersion> {
    pub dimension_definition_group: V::DimensionDefinitionGroup,
}

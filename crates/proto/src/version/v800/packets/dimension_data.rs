use super::super::types::DimensionDefinitionGroup;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 180)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDataPacket {
    pub dimension_definition_group: DimensionDefinitionGroup,
}
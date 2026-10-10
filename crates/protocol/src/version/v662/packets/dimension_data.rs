use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 180, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDataPacket<V: ProtoVersion> {
    pub dimension_definition_group: V::DimensionDefinitionGroup,
}

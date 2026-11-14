use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDefinitionGroup {
    pub definitions: Vec<DimensionDefinitionGroupType>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDefinitionGroupType {
    pub name: String,
    pub dimension_definition: DimensionDefinition,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDefinition {
    #[endianness(var)]
    pub minimum_y: i32,
    #[endianness(var)]
    pub height_range: i32,
    #[endianness(var)]
    pub generator_type: i32,
    #[endianness(var)]
    pub dimension_type: i32,
    pub pack_id: Uuid,
    pub default_biome: String,
}

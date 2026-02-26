use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DimensionDefinitionGroup {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
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
    pub height_max: i32,
    #[endianness(var)]
    pub height_min: i32,
    #[endianness(var)]
    pub generator_type: i32,
}

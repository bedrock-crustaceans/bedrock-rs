use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerMixDataEntry {
    #[endianness(var)]
    pub input_item_id: i32,
    #[endianness(var)]
    pub reagent_item_id: i32,
    #[endianness(var)]
    pub output_item_id: i32,
}
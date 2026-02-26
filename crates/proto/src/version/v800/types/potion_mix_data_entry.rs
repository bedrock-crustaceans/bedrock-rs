use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PotionMixDataEntry {
    #[endianness(var)]
    pub input_potion_id: i32,
    #[endianness(var)]
    pub input_item_aux: i32,
    #[endianness(var)]
    pub reagent_item_id: i32,
    #[endianness(var)]
    pub reagent_item_aux: i32,
    #[endianness(var)]
    pub output_potion_id: i32,
    #[endianness(var)]
    pub output_item_aux: i32,
}
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MaterialReducerDataEntry {
    #[endianness(var)]
    pub input: i32,

    pub ids_and_counts: Vec<MaterialReducerDataEntryIdAndCount>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MaterialReducerDataEntryIdAndCount {
    #[endianness(var)]
    pub id: i32,
    #[endianness(var)]
    pub count: i32,
}

use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MaterialReducerDataEntry {
    #[endianness(var)]
    pub input: i32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub ids_and_counts: Vec<MaterialReducerDataEntryIdAndCount>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MaterialReducerDataEntryIdAndCount {
    #[endianness(var)]
    pub id: i32,
    #[endianness(var)]
    pub count: i32,
}

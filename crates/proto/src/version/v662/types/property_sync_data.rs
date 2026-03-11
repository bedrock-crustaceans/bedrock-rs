use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PropertySyncData {
    pub int_entries_list: Vec<IntEntry>,

    pub float_entries_list: Vec<FloatEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct IntEntry {
    #[endianness(var)]
    pub property_index: u32,
    #[endianness(le)]
    pub data: f32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct FloatEntry {
    #[endianness(var)]
    pub property_index: u32,
    #[endianness(var)]
    pub data: i32,
}

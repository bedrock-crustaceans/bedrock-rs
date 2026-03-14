use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 332)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerBoundDataStorePacket {
    pub data_store_name: String,
    pub property: String,
    pub path: String,
    pub data: ServerBoundDataStoreValue,
    #[endianness(le)]
    pub update_count: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ServerBoundDataStoreValue {
    Double(#[endianness(le)] f64) = 0,
    Bool(bool) = 1,
    String(String) = 2,
}

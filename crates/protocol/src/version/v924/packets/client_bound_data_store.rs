use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 330)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataStorePacket {
    pub updates: Vec<ClientBoundDataStoreUpdate>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ClientBoundDataStoreUpdate {
    Update {
        data_store_name: String,
        property: String,
        path: String,
        data: ClientBoundDataStoreValue,
        #[endianness(le)]
        update_count: u32,
        #[endianness(le)]
        path_update_count: u32,
    } = 0,
    Change {
        data_store_name: String,
        property: String,
        #[endianness(le)]
        update_count: u32,
        new_value: ClientBoundDataStorePropertyValue,
    } = 1,
    Remove {
        data_store_name: String,
    } = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum ClientBoundDataStoreValue {
    Double(#[endianness(le)] f64) = 0,
    Bool(bool) = 1,
    String(String) = 2,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum ClientBoundDataStorePropertyValue {
    None = 0,
    Bool(bool) = 1,
    Int64(#[endianness(le)] i64) = 2,
    String(String) = 4,
    Map(Vec<DataStoreMapEntry>) = 6,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataStoreMapEntry {
    pub key: String,
    pub value: ClientBoundDataStorePropertyValue,
}

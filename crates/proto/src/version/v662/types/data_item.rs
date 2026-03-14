use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataItem<V: ProtoVersion> {
    #[endianness(var)]
    pub data_item_id: u32,
    pub data_item_type: V::DataItemType,
}

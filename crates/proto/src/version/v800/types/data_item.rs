use super::super::enums::DataItemType;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct DataItem {
    #[endianness(var)]
    pub data_item_id: u32,
    pub data_item_type: DataItemType,
}
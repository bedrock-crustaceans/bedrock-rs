use bedrockrs_macros::ProtoCodec;
use super::super::types::FullContainerName;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackRequestSlotInfo {
    pub container_name: FullContainerName,
    pub slot: i8,
    #[endianness(var)]
    pub raw_id: i32,
}
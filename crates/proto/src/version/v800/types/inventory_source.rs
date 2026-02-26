use super::super::enums::InventorySourceType;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySource {
    pub source_type: InventorySourceType,
}
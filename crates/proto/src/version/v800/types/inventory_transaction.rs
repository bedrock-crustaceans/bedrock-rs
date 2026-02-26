use super::super::types::InventoryAction;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransaction {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub action: Vec<InventoryAction>,
}
use super::super::enums::ComplexInventoryTransactionType;
use super::super::types::InventoryTransaction;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 30)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransactionPacket {
    #[endianness(var)]
    pub raw_id: i32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub legacy_set_item_slots: Vec<LegacySetItemSlotsEntry>,
    pub transaction_type: ComplexInventoryTransactionType,
    pub transaction: InventoryTransaction,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LegacySetItemSlotsEntry {
    pub container_enum: i8, // TODO: find container enum?
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub slot_vector: Vec<i8>, // TODO: find slot enum? (i8 is Slot)
}

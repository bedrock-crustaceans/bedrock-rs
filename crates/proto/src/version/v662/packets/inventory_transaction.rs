use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 30)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransactionPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub raw_id: i32,
    pub legacy_set_item_slots: Vec<LegacySetItemSlotsEntry>,
    pub transaction_type: V::ComplexInventoryTransactionType,
    pub transaction: V::InventoryTransaction,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LegacySetItemSlotsEntry {
    pub container_enum: i8,   // TODO: find container enum?
    pub slot_vector: Vec<i8>, // TODO: find slot enum? (i8 is Slot)
}

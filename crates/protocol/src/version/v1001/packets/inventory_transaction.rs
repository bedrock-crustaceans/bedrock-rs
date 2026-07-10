use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 30)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransactionPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub raw_id: i32,
    pub legacy_set_item_slots: Option<Vec<LegacySetItemSlotsEntry>>,
    pub transaction_type: Option<V::ComplexInventoryTransactionType>,
    pub transaction: Option<V::InventoryTransaction>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct LegacySetItemSlotsEntry {
    pub container_enum: i8,
    pub slot_vector: Vec<i8>,
}

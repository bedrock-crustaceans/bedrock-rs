use crate::ProtoVersion;
use crate::version::v662::enums::ComplexInventoryTransactionType;
use crate::version::v662::packets::LegacySetItemSlotsEntry;
use crate::version::v662::types::InventoryTransactionData;
use bedrock_macros::packet;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 30)]
#[derive(Clone, Debug)]
pub struct InventoryTransactionPacket<V: ProtoVersion> {
    pub legacy_request_id: i32,
    pub legacy_set_item_slots: Option<Vec<LegacySetItemSlotsEntry>>,
    pub actions: Vec<V::InventoryAction>,
    pub data: InventoryTransactionData<V>,
}

impl<V: ProtoVersion> ProtoCodec for InventoryTransactionPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.legacy_request_id, stream)?;
        self.legacy_set_item_slots.serialize(stream)?;
        self.data.transaction_type().serialize(stream)?;
        self.actions.serialize(stream)?;
        self.data.serialize_payload(stream)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let legacy_request_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let legacy_set_item_slots = ProtoCodec::deserialize(stream)?;
        let transaction_type = ComplexInventoryTransactionType::deserialize(stream)?;
        let actions = ProtoCodec::deserialize(stream)?;
        let data = InventoryTransactionData::deserialize_payload(&transaction_type, stream)?;
        Ok(Self { legacy_request_id, legacy_set_item_slots, actions, data })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.legacy_request_id)
            + self.legacy_set_item_slots.size_hint()
            + self.data.transaction_type().size_hint()
            + self.actions.size_hint()
            + self.data.size_hint()
    }
}

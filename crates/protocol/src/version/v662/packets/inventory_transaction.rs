use crate::ProtoVersion;
use crate::version::v662::enums::ComplexInventoryTransactionType;
use crate::version::v662::types::InventoryTransactionData;
use bedrock_macros::{ProtoCodec, packet};
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

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
pub struct LegacySetItemSlotsEntry {
    pub container_id: i8,
    pub slots: Vec<i8>,
}

pub fn has_legacy_set_item_slots(legacy_request_id: i32) -> bool {
    legacy_request_id < -1 && legacy_request_id & 1 == 0
}

impl<V: ProtoVersion> ProtoCodec for InventoryTransactionPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.legacy_request_id, stream)?;
        if has_legacy_set_item_slots(self.legacy_request_id) {
            let slots = self.legacy_set_item_slots.as_ref().ok_or(ProtoCodecError::ExpectedSome("legacy_set_item_slots"))?;
            slots.serialize(stream)?;
        }
        self.data.transaction_type().serialize(stream)?;
        self.actions.serialize(stream)?;
        self.data.serialize_payload(stream)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let legacy_request_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let legacy_set_item_slots = match has_legacy_set_item_slots(legacy_request_id) {
            true => Some(ProtoCodec::deserialize(stream)?),
            false => None,
        };
        let transaction_type = ComplexInventoryTransactionType::deserialize(stream)?;
        let actions = ProtoCodec::deserialize(stream)?;
        let data = InventoryTransactionData::deserialize_payload(&transaction_type, stream)?;
        Ok(Self { legacy_request_id, legacy_set_item_slots, actions, data })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.legacy_request_id)
            + self.legacy_set_item_slots.as_ref().map_or(0, |slots| slots.size_hint())
            + self.data.transaction_type().size_hint()
            + self.actions.size_hint()
            + self.data.size_hint()
    }
}

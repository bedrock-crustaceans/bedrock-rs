use crate::ProtoVersion;
use crate::version::v662::packets::{LegacySetItemSlotsEntry, has_legacy_set_item_slots};
use crate::version::v662::types::ItemUseTransactionData;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::fmt::Debug;
use std::io::{Read, Write};

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, false, false>;

#[derive(Clone, Debug)]
pub struct ItemUseTransactionWithActions<V: ProtoVersion, U, const SLOTS_FLAGGED: bool, const ACTIONS_FLAGGED: bool> {
    pub legacy_request_id: i32,
    pub legacy_set_item_slots: Option<Vec<LegacySetItemSlotsEntry>>,
    pub actions: V::InventoryTransaction,
    pub item_use: U,
}

fn expect_present<R: Read>(stream: &mut R, field: &'static str) -> Result<(), ProtoCodecError> {
    bool::deserialize(stream)?.then_some(()).ok_or(ProtoCodecError::ExpectedSome(field))
}

impl<V, U, const SLOTS_FLAGGED: bool, const ACTIONS_FLAGGED: bool> ProtoCodec for ItemUseTransactionWithActions<V, U, SLOTS_FLAGGED, ACTIONS_FLAGGED>
where
    V: ProtoVersion,
    U: ProtoCodec + Clone + Debug,
{
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.legacy_request_id, stream)?;
        if SLOTS_FLAGGED {
            self.legacy_set_item_slots.serialize(stream)?;
        } else if has_legacy_set_item_slots(self.legacy_request_id) {
            let slots = self.legacy_set_item_slots.as_ref().ok_or(ProtoCodecError::ExpectedSome("legacy_set_item_slots"))?;
            slots.serialize(stream)?;
        }
        if ACTIONS_FLAGGED {
            true.serialize(stream)?;
            true.serialize(stream)?;
        }
        self.actions.serialize(stream)?;
        self.item_use.serialize(stream)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let legacy_request_id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let legacy_set_item_slots = match SLOTS_FLAGGED {
            true => ProtoCodec::deserialize(stream)?,
            false if has_legacy_set_item_slots(legacy_request_id) => Some(ProtoCodec::deserialize(stream)?),
            false => None,
        };
        if ACTIONS_FLAGGED {
            expect_present(stream, "actions")?;
            expect_present(stream, "actions")?;
        }
        let actions = ProtoCodec::deserialize(stream)?;
        let item_use = U::deserialize(stream)?;
        Ok(Self { legacy_request_id, legacy_set_item_slots, actions, item_use })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.legacy_request_id)
            + self.legacy_set_item_slots.size_hint()
            + if ACTIONS_FLAGGED { 2 } else { 0 }
            + self.actions.size_hint()
            + self.item_use.size_hint()
    }
}

use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct PackedItemUseLegacyInventoryTransaction<V: ProtoVersion> {
    pub id: i32,
    pub container_slots: Option<Vec<ContainerSlotEntry>>,
    pub action: V::InventoryTransaction,
    pub action_type: V::ItemUseInventoryTransactionType,
    pub position: V::NetworkBlockPosition,
    pub face: i32,
    pub slot: i32,
    pub item: V::NetworkItemStackDescriptor,
    pub from_position: (f32, f32, f32),
    pub click_position: (f32, f32, f32),
    pub target_block_id: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerSlotEntry {
    pub container_enum_name: String,

    pub slots: Vec<i8>,
}

impl<V: ProtoVersion> ProtoCodec for PackedItemUseLegacyInventoryTransaction<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.id, stream)?;

        match &self.id {
            0 => {}
            _ => {
                let vec = self.container_slots.as_ref().unwrap();
                let len: u32 = vec.len().try_into()?;
                ProtoCodecVAR::serialize(&len, stream)?;
                for i in vec {
                    i.serialize(stream)?
                }
            }
        }

        self.action.serialize(stream)?;
        self.action_type.serialize(stream)?;
        self.position.serialize(stream)?;
        ProtoCodecVAR::serialize(&self.face, stream)?;
        ProtoCodecVAR::serialize(&self.slot, stream)?;
        self.item.serialize(stream)?;
        ProtoCodecLE::serialize(&self.from_position, stream)?;
        ProtoCodecLE::serialize(&self.click_position, stream)?;
        ProtoCodecVAR::serialize(&self.target_block_id, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let id = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let container_slots = match id {
            0 => None,
            _ => {
                let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
                let mut vec = Vec::with_capacity(len.try_into()?);
                for _ in 0..len {
                    vec.push(ContainerSlotEntry::deserialize(stream)?);
                }
                Some(vec)
            }
        };
        let action = V::InventoryTransaction::deserialize(stream)?;
        let action_type = V::ItemUseInventoryTransactionType::deserialize(stream)?;
        let position = V::NetworkBlockPosition::deserialize(stream)?;
        let face = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let slot = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let item = V::NetworkItemStackDescriptor::deserialize(stream)?;
        let from_position = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let click_position = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let target_block_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;

        Ok(Self {
            id,
            container_slots,
            action,
            action_type,
            position,
            face,
            slot,
            item,
            from_position,
            click_position,
            target_block_id,
        })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.id)
            + match &self.id {
                0 => 0,
                _ => {
                    let vec = self.container_slots.as_ref().unwrap();
                    vec.len() + vec.iter().map(|i| i.size_hint()).sum::<usize>()
                }
            }
            + self.action.size_hint()
            + self.action_type.size_hint()
            + self.position.size_hint()
            + ProtoCodecVAR::size_hint(&self.face)
            + ProtoCodecVAR::size_hint(&self.slot)
            + self.item.size_hint()
            + ProtoCodecLE::size_hint(&self.from_position)
            + ProtoCodecLE::size_hint(&self.click_position)
            + ProtoCodecVAR::size_hint(&self.target_block_id)
    }
}

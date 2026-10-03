use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemReleaseTransactionData<V: ProtoVersion> {
    pub action_type: V::ItemReleaseInventoryTransactionType,
    #[endianness(var)]
    pub hotbar_slot: i32,
    pub held_item: V::NetworkItemStackDescriptor,
    #[endianness(le)]
    pub head_position: (f32, f32, f32),
}

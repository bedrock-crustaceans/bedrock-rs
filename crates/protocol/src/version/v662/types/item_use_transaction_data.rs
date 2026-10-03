use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemUseTransactionData<V: ProtoVersion> {
    pub action_type: V::ItemUseInventoryTransactionType,
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub block_face: i32,
    #[endianness(var)]
    pub hotbar_slot: i32,
    pub held_item: V::NetworkItemStackDescriptor,
    #[endianness(le)]
    pub player_position: (f32, f32, f32),
    #[endianness(le)]
    pub click_position: (f32, f32, f32),
    #[endianness(var)]
    pub block_runtime_id: u32,
}

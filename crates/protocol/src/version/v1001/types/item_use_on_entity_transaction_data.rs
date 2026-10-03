use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemUseOnEntityTransactionData<V: ProtoVersion> {
    #[endianness(var)]
    pub actor_runtime_id: u64,
    #[endianness(var)]
    pub action_type: i32,
    #[endianness(var)]
    pub hotbar_slot: i32,
    pub held_item: V::NetworkItemStackDescriptorV2,
    #[endianness(le)]
    pub player_position: (f32, f32, f32),
    #[endianness(le)]
    pub click_position: (f32, f32, f32),
}

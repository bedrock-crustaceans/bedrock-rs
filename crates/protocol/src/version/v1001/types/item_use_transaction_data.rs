use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemUseTransactionData<V: ProtoVersion> {
    #[endianness(var)]
    pub action_type: i32,
    pub trigger_type: u8,
    pub block_position: V::NetworkBlockPosition,
    pub block_face: u8,
    #[endianness(var)]
    pub hotbar_slot: i32,
    pub held_item: V::NetworkItemStackDescriptorV2,
    #[endianness(le)]
    pub player_position: (f32, f32, f32),
    #[endianness(le)]
    pub click_position: (f32, f32, f32),
    #[endianness(var)]
    pub block_runtime_id: u32,
    pub client_prediction: u8,
    pub client_cooldown_state: u8,
}

use crate::ProtoVersion;
use crate::version::v662::types::ItemUseTransactionWithActions;
use bedrock_macros::ProtoCodec;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, AuthInputItemUseData<V>, false, false>;

#[derive(ProtoCodec, Clone, Debug)]
pub struct AuthInputItemUseData<V: ProtoVersion> {
    pub action_type: V::ItemUseInventoryTransactionType,
    #[endianness(var)]
    pub trigger_type: u32,
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub block_face: i32,
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

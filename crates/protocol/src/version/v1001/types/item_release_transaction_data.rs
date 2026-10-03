use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemReleaseTransactionData<V: ProtoVersion> {
    #[endianness(var)]
    pub action_type: i32,
    #[endianness(var)]
    pub hotbar_slot: i32,
    pub held_item: V::NetworkItemStackDescriptorV2,
    #[endianness(le)]
    pub head_position: (f32, f32, f32),
}

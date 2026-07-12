use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseSlotInfo<V: ProtoVersion> {
    pub requested_slot: i8,
    pub slot: i8,
    pub amount: i8,
    #[endianness(var)]
    pub item_stack_net_id: Option<Option<i32>>,
    pub custom_name: V::RedactableString,
    #[endianness(var)]
    pub durability_correction: i32,
}

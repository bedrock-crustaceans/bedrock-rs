use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemStackResponseSlotInfo {
    pub requested_slot: i8,
    pub slot: i8,
    pub amount: i8,
    #[endianness(var)]
    pub item_stack_net_id: i32,
    pub custom_name: String,
    pub filtered_custom_name: String,
    #[endianness(var)]
    pub durability_correction: i32,
}
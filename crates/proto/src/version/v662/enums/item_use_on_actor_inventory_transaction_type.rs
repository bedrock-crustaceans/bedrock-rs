use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
/// UNUSED
pub enum ItemUseOnActorInventoryTransactionType {
    Interact = 0,
    Attack = 1,
    ItemInteract = 2,
}

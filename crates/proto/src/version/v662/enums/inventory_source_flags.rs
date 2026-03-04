use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
/// UNUSED
pub enum InventorySourceFlags {
    NoFlag = 0,
    WorldInteractionRandom = 1,
}

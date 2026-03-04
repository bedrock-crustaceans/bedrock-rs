use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_endianness(var)]
#[enum_repr(u32)]
#[repr(u32)]
/// UNUSED
pub enum ItemReleaseInventoryTransactionType {
    Release = 0,
    Use = 1,
}

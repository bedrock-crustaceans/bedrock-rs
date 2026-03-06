use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum InventorySourceType<V: ProtoVersion> {
    InvalidInventory = u32::MAX,
    ContainerInventory(V::ContainerID) = 0,
    GlobalInventory = 1,
    WorldInteraction(#[endianness(var)] u32) = 2,
    CreativeInventory = 3,
    NonImplementedFeatureTODO = 99999,
}

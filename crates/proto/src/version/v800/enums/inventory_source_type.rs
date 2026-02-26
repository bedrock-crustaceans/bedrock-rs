use super::super::enums::ContainerID;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum InventorySourceType {
    InvalidInventory = u32::MAX,
    ContainerInventory(ContainerID) = 0,
    GlobalInventory = 1,
    WorldInteraction(#[endianness(var)] u32) = 2,
    CreativeInventory = 3,
    NonImplementedFeatureTODO = 99999,
}
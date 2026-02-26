use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum InventoryRightTabIndex {
    None = 0,
    FullScreen = 1,
    Crafting = 2,
    Armor = 3,
    Count = 4,
}
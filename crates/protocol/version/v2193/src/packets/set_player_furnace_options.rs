use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 351)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerFurnaceOptionsPacket {
    pub furnace_type: FurnaceType,
    pub furnace_options: FurnaceOptions,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct FurnaceOptions {
    pub left_furnace_tab: FurnaceLeftTabIndex,
    pub filtering: bool,
    pub furnace_layout: FurnaceLayout,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum FurnaceType {
    None = 0,
    Furnace = 1,
    BlastFurnace = 2,
    Smoker = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum FurnaceLeftTabIndex {
    None = 0,
    RecipeFood = 1,
    RecipeItems = 2,
    RecipeBlocks = 3,
    RecipeSearch = 4,
    Inventory = 5,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum FurnaceLayout {
    None = 0,
    InventoryOnly = 1,
    Default = 2,
}

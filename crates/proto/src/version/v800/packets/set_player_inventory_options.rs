use super::super::enums::{InventoryLayout, InventoryLeftTabIndex, InventoryRightTabIndex};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 307)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerInventoryOptionsPacket {
    pub left_inventory_tab: InventoryLeftTabIndex,
    pub right_inventory_tab: InventoryRightTabIndex,
    pub filtering: bool,
    pub layout_inv: InventoryLayout,
    pub layout_craft: InventoryLayout,
}
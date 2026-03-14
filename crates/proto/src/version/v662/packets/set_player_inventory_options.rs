use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 307)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerInventoryOptionsPacket<V: ProtoVersion> {
    pub left_inventory_tab: V::InventoryLeftTabIndex,
    pub right_inventory_tab: V::InventoryRightTabIndex,
    pub filtering: bool,
    pub layout_inv: V::InventoryLayout,
    pub layout_craft: V::InventoryLayout,
}

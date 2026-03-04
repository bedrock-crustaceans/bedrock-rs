use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 307)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPlayerInventoryOptionsPacket<V: ProtoVersion> {
    pub left_inventory_tab: V::InventoryLeftTabIndex,
    pub right_inventory_tab: V::InventoryRightTabIndex,
    pub filtering: bool,
    pub layout_inv: V::InventoryLayout,
    pub layout_craft: V::InventoryLayout,
}
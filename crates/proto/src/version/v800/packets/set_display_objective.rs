use super::super::enums::ObjectiveSortOrder;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 107)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDisplayObjectivePacket {
    pub display_slot_name: String,
    pub objective_name: String,
    pub objective_display_name: String,
    pub criteria_name: String,
    pub sort_order: ObjectiveSortOrder,
}
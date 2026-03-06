use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 107)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetDisplayObjectivePacket<V: ProtoVersion> {
    pub display_slot_name: String,
    pub objective_name: String,
    pub objective_display_name: String,
    pub criteria_name: String,
    pub sort_order: V::ObjectiveSortOrder,
}

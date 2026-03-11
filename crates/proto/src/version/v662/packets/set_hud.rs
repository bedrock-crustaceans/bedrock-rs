use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 308)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHudPacket<V: ProtoVersion> {
    pub hud_elements_list: Vec<V::HudElement>,
    pub hud_visibility: V::HudVisibility,
}

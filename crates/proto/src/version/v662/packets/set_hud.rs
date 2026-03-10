use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 308)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHudPacket<V: ProtoVersion> {
    pub hud_elements_list: Vec<V::HudElement>,
    pub hud_visibility: V::HudVisibility,
}

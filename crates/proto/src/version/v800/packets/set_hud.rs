use super::super::enums::{HudElement, HudVisibility};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 308)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHudPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub hud_elements_list: Vec<HudElement>,
    pub hud_visibility: HudVisibility,
}
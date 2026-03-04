use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 308)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHudPacket<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub hud_elements_list: Vec<V::HudElement>,
    pub hud_visibility: V::HudVisibility,
}
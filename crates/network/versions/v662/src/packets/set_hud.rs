use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 308, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetHudPacket<V: ProtoVersion> {
    pub hud_elements_list: Vec<V::HudElement>,
    pub hud_visibility: V::HudVisibility,
}

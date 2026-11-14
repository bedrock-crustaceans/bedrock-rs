use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 48)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerHotbarPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub selected_slot: u32,
    pub container_id: V::ContainerID,
    pub should_select_slot: bool,
}

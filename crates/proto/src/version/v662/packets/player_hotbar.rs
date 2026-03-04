use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 48)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerHotbarPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub selected_slot: u32,
    pub container_id: V::ContainerID,
    pub should_select_slot: bool,
}
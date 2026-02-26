use super::super::enums::ContainerID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 48)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerHotbarPacket {
    #[endianness(var)]
    pub selected_slot: u32,
    pub container_id: ContainerID,
    pub should_select_slot: bool,
}
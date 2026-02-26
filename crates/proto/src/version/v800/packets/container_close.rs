use super::super::enums::{ContainerID, ContainerType};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 47)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerClosePacket {
    pub container_id: ContainerID,
    pub container_type: ContainerType,
    pub server_initiated_close: bool,
}
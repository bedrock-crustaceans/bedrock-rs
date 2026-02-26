use super::super::enums::{ContainerID, ContainerType};
use super::super::types::{ActorUniqueID, NetworkBlockPosition};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 46)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerOpenPacket {
    pub container_id: ContainerID,
    pub container_type: ContainerType,
    pub position: NetworkBlockPosition,
    pub target_actor_id: ActorUniqueID,
}
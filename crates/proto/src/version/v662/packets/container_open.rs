use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 46)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerOpenPacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    pub container_type: V::ContainerType,
    pub position: V::NetworkBlockPosition,
    pub target_actor_id: V::ActorUniqueID,
}

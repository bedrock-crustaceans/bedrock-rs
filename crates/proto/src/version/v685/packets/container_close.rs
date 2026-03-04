use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 47)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerClosePacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    pub container_type: V::ContainerType,
    pub server_initiated_close: bool,
}

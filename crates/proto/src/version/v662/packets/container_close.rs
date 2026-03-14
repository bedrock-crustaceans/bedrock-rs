use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 47)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerClosePacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    pub server_initiated_close: bool,
}

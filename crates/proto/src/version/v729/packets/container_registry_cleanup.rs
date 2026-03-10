use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 317)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerRegistryCleanupPacket<V: ProtoVersion> {
    containers: Vec<V::FullContainerName>,
}

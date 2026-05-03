use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 317)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerRegistryCleanupPacket<V: ProtoVersion> {
    containers: Vec<V::FullContainerName>,
}

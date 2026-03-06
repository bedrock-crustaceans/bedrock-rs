use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 317)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerRegistryCleanupPacket<V: ProtoVersion> {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    containers: Vec<V::FullContainerName>,
}

use bedrockrs_macros::{gamepacket, ProtoCodec};
use super::super::types::FullContainerName;

#[gamepacket(id = 317)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerRegistryCleanupPacket {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    containers: Vec<FullContainerName>
}
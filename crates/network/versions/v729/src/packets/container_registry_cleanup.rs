use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 317, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ContainerRegistryCleanupPacket<V: ProtoVersion> {
    containers: Vec<V::FullContainerName>,
}

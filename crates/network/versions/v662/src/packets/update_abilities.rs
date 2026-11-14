use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 187, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateAbilitiesPacket<V: ProtoVersion> {
    pub data: V::SerializedAbilitiesData,
}

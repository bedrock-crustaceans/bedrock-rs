use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 170, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EduUriResourcePacket<V: ProtoVersion> {
    pub edu_shared_uri_resource: V::EduSharedUriResource,
}

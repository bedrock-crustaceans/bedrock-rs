use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 41, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorLinkPacket<V: ProtoVersion> {
    pub link: V::ActorLink,
}

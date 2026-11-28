use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 96, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLastHurtByPacket<V: ProtoVersion> {
    pub last_hurt_by: V::ActorType,
}

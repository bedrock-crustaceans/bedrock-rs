use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 74, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BossEventPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub event_type: V::BossEventUpdateType,
}

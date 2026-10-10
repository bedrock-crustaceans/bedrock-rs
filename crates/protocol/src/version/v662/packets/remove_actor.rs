use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 14, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
}

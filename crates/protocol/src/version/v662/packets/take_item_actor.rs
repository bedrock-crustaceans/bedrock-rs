use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 17, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TakeItemActorPacket<V: ProtoVersion> {
    pub item_runtime_id: V::ActorRuntimeID,
    pub actor_runtime_id: V::ActorRuntimeID,
}

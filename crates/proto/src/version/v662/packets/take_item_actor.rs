use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 17)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TakeItemActorPacket<V: ProtoVersion> {
    pub item_runtime_id: V::ActorRuntimeID,
    pub actor_runtime_id: V::ActorRuntimeID,
}

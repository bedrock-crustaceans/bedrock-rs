use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 17)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TakeItemActorPacket<V: ProtoVersion> {
    pub item_runtime_id: V::ActorRuntimeID,
    pub actor_runtime_id: V::ActorRuntimeID,
}

use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 14)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
}
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 14)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveActorPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
}

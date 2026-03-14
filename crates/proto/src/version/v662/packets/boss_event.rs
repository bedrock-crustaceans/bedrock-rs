use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 74)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BossEventPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub event_type: V::BossEventUpdateType,
}

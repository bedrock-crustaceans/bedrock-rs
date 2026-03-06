use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 96)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLastHurtByPacket<V: ProtoVersion> {
    pub last_hurt_by: V::ActorType,
}

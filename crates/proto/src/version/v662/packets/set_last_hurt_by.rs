use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 96)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLastHurtByPacket<V: ProtoVersion> {
    pub last_hurt_by: V::ActorType,
}
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 155)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugInfoPacket<V: ProtoVersion> {
    pub actor_id: V::ActorUniqueID,
    pub data: String,
}

use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 155)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugInfoPacket<V: ProtoVersion> {
    pub actor_id: V::ActorUniqueID,
    pub data: String,
}
use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 113)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLocalPlayerAsInitializedPacket<V: ProtoVersion> {
    pub player_id: V::ActorRuntimeID,
}
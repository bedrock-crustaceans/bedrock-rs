use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 113)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLocalPlayerAsInitializedPacket<V: ProtoVersion> {
    pub player_id: V::ActorRuntimeID,
}

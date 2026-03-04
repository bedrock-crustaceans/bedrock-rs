use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 131)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapCreateLockedCopyPacket<V: ProtoVersion> {
    pub original_map_id: V::ActorUniqueID,
    pub new_map_id: V::ActorUniqueID,
}
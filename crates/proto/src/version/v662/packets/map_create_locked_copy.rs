use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 131)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapCreateLockedCopyPacket<V: ProtoVersion> {
    pub original_map_id: V::ActorUniqueID,
    pub new_map_id: V::ActorUniqueID,
}

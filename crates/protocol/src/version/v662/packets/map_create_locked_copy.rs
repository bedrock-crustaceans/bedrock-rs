use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 131, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapCreateLockedCopyPacket<V: ProtoVersion> {
    pub original_map_id: V::ActorUniqueID,
    pub new_map_id: V::ActorUniqueID,
}

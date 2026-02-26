use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 131)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MapCreateLockedCopyPacket {
    pub original_map_id: ActorUniqueID,
    pub new_map_id: ActorUniqueID,
}
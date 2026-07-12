use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 67)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundMapItemDataPacket<V: ProtoVersion> {
    pub map_id: V::ActorUniqueID,
    pub dimension: i8,
    pub is_locked: bool,
    pub map_origin: V::NetworkBlockPosition,
    pub creation_map_ids: Option<Vec<V::ActorUniqueID>>,
    pub scale: Option<i8>,
    pub tracked_actor_ids: Option<Vec<V::MapItemTrackedActorUniqueID>>,
    pub decorations: Option<Vec<V::MapDecoration>>,
    #[endianness(var)]
    pub width: Option<i32>,
    #[endianness(var)]
    pub height: Option<i32>,
    #[endianness(var)]
    pub start_x: Option<i32>,
    #[endianness(var)]
    pub start_y: Option<i32>,
    #[endianness(le)]
    pub pixels: Option<Vec<i32>>,
}

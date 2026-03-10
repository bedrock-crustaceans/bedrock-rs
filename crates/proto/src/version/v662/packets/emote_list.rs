use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use uuid::Uuid;

#[gamepacket(id = 152)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EmoteListPacket<V: ProtoVersion> {
    pub runtime_id: V::ActorRuntimeID,

    pub emote_piece_ids: Vec<Uuid>,
}

use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use uuid::Uuid;

#[gamepacket(id = 152)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EmoteListPacket {
    pub runtime_id: ActorRuntimeID,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub emote_piece_ids: Vec<Uuid>,
}
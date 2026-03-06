use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use uuid::Uuid;

#[packet(id = 152)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct EmoteListPacket<V: ProtoVersion> {
    pub runtime_id: V::ActorRuntimeID,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub emote_piece_ids: Vec<Uuid>,
}

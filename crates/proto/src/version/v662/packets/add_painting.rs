use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 22)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AddPaintingPacket<V: ProtoVersion> {
    pub target_actor_id: V::ActorUniqueID,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub direction: i32,
    pub motif: String,
}

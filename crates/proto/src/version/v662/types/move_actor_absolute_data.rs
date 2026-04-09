use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsoluteData<V: ProtoVersion> {
    pub actor_runtime_id: V::ActorRuntimeID,
    pub header: i8,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    pub rotation_x: i8,
    pub rotation_y: i8,
    pub rotation_y_head: i8,
}

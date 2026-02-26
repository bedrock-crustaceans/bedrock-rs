use super::super::types::ActorRuntimeID;
use bedrockrs_macros::ProtoCodec;
use vek::Vec3;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorAbsoluteData {
    pub actor_runtime_id: ActorRuntimeID,
    pub header: i8,
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub rotation_x: i8,
    pub rotation_y: i8,
    pub rotation_y_head: i8,
}
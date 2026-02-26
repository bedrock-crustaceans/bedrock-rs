use super::super::types::ActorRuntimeID;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorDeltaData {
    pub actor_runtime_id: ActorRuntimeID,
    #[endianness(le)]
    pub header: u16,
    #[endianness(le)]
    pub position_x: f32,
    #[endianness(le)]
    pub position_y: f32,
    #[endianness(le)]
    pub position_z: f32,
    pub rotation_x: i8,
    pub rotation_y: i8,
    pub rotation_y_head: i8,
}
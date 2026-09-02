use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MoveActorDeltaData<V: ProtoVersion> {
    pub actor_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub position_x: Option<f32>,
    #[endianness(le)]
    pub position_y: Option<f32>,
    #[endianness(le)]
    pub position_z: Option<f32>,
    pub rotation_x: Option<i8>,
    pub rotation_y: Option<i8>,
    pub rotation_y_head: Option<i8>,
    pub on_ground: bool,
    pub force_move: bool,
    pub force_move_local_entity: bool,
    pub force_completion: bool,
    #[endianness(var)]
    pub ticks: u64,
}

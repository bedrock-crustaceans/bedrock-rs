use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 19)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MovePlayerPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(le)]
    pub rotation: (f32, f32),
    #[endianness(le)]
    pub y_head_rotation: f32,
    pub position_mode: V::PlayerPositionMode,
    pub on_ground: bool,
    pub riding_runtime_id: V::ActorRuntimeID,
    pub teleport_data: Option<V::MovePlayerTeleportData>,
    #[endianness(var)]
    pub tick: u64,
}

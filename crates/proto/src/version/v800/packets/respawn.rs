use super::super::enums::PlayerRespawnState;
use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 45)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RespawnPacket {
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub state: PlayerRespawnState,
    pub player_runtime_id: ActorRuntimeID,
}
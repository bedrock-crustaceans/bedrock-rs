use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 45)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RespawnPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub state: V::PlayerRespawnState,
    pub player_runtime_id: V::ActorRuntimeID,
}

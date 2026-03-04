use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 45)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RespawnPacket<V: ProtoVersion> {
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub state: V::PlayerRespawnState,
    pub player_runtime_id: V::ActorRuntimeID,
}
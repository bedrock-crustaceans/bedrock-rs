use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 322)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MovementPredictionSyncPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub flags: u128,
    #[endianness(le)]
    pub bounding_box: Vec3<f32>,
    #[endianness(le)]
    pub speed: f32,
    #[endianness(le)]
    pub underwater_speed: f32,
    #[endianness(le)]
    pub lava_speed: f32,
    #[endianness(le)]
    pub jump_strength: f32,
    #[endianness(le)]
    pub health: f32,
    #[endianness(le)]
    pub hunger: f32,
    pub runtime_entity_id: V::ActorRuntimeID,
    pub is_flying: bool,
}

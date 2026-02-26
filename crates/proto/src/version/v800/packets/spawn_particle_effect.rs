use super::super::types::{ActorUniqueID, MolangVariableMap};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 118)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnParticleEffectPacket {
    pub dimension_id: i8,
    pub actor_id: ActorUniqueID,
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub effect_name: String,
    pub molang_variables: Option<MolangVariableMap>,
}
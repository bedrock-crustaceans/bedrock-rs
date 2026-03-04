use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 118)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnParticleEffectPacket<V: ProtoVersion> {
    pub dimension_id: i8,
    pub actor_id: V::ActorUniqueID,
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub effect_name: String,
    pub molang_variables: Option<V::MolangVariableMap>,
}
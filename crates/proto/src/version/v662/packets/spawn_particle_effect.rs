use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 118)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnParticleEffectPacket<V: ProtoVersion> {
    pub dimension_id: i8,
    pub actor_id: V::ActorUniqueID,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    pub effect_name: String,
    pub molang_variables: Option<V::MolangVariableMap>,
}

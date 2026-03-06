use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnSettings<V: ProtoVersion> {
    pub spawn_type: V::SpawnBiomeType,
    pub user_defined_biome_name: String,
    #[endianness(var)]
    pub dimension: i32,
}

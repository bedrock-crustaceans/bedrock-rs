use super::super::enums::SpawnBiomeType;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnSettings {
    pub spawn_type: SpawnBiomeType,
    pub user_defined_biome_name: String,
    #[endianness(var)]
    pub dimension: i32,
}

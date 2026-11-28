use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SpawnSettings<V: ProtoVersion> {
    pub spawn_type: V::SpawnBiomeType,
    pub user_defined_biome_name: String,
    #[endianness(var)]
    pub dimension: i32,
}

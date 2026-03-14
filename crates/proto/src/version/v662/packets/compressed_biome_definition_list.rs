use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 301)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompressedBiomeDefinitionListPacket {
    pub compressed_biome_data: String,
}

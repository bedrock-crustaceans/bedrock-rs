use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 301)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompressedBiomeDefinitionListPacket {
    pub compressed_biome_data: String,
}
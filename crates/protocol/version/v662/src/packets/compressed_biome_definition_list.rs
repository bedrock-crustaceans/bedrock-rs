use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 301, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CompressedBiomeDefinitionListPacket {
    pub compressed_biome_data: String,
}

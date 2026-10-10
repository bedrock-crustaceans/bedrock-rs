use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 314, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CurrentStructureFeaturePacket {
    pub current_structure_feature: String,
}

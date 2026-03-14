use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 314)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CurrentStructureFeaturePacket {
    pub current_structure_feature: String,
}

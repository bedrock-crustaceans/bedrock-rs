use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 191)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct FeatureRegistryPacket {
    pub features_data_list: Vec<FeaturesDataListEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct FeaturesDataListEntry {
    pub feature_name: String,
    pub binary_json_output: String,
}

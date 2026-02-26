use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 191)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct FeatureRegistryPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub features_data_list: Vec<FeaturesDataListEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct FeaturesDataListEntry {
    pub feature_name: String,
    pub binary_json_output: String,
}

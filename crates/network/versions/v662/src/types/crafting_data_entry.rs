use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataEntry<V: ProtoVersion> {
    pub crafting_type: V::CraftingDataEntryType,
}

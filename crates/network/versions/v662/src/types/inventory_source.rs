use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySource<V: ProtoVersion> {
    pub source_type: V::InventorySourceType,
}

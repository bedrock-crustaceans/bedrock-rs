use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransaction<V: ProtoVersion> {
    pub action: Vec<V::InventoryAction>,
}

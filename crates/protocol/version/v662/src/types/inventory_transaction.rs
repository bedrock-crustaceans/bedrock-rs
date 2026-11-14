use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransaction<V: ProtoVersion> {
    pub action: Vec<V::InventoryAction>,
}

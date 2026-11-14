use bedrock_protocol_version::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySource<V: ProtoVersion> {
    pub source_type: V::InventorySourceType,
}

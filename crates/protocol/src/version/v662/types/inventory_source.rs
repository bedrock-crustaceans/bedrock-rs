use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySource<V: ProtoVersion> {
    pub source_type: V::InventorySourceType,
}

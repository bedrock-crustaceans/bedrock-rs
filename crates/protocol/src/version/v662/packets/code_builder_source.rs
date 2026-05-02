use crate::version::versions::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 178)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CodeBuilderSourcePacket<V: ProtoVersion> {
    pub operation: V::CodeBuilderStorageOperation,
    pub category: V::CodeBuilderStorageCategory,
    pub value: String,
}

use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 178)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CodeBuilderSourcePacket<V: ProtoVersion> {
    pub operation: V::CodeBuilderStorageOperation,
    pub category: V::CodeBuilderStorageCategory,
    pub code_staus: V::CodeBuilderCodeStatus,
}

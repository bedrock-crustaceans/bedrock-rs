use super::super::enums::{CodeBuilderStorageCategory, CodeBuilderStorageOperation, CodeBuilderCodeStatus};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 178)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CodeBuilderSourcePacket {
    pub operation: CodeBuilderStorageOperation,
    pub category: CodeBuilderStorageCategory,
    pub code_status: CodeBuilderCodeStatus,
}
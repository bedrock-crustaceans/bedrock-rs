use bedrock_network_version_v1001::types::InventorySourceKind;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
pub struct InventorySource {
    pub source_type: InventorySourceKind,
    pub container_id: Option<i8>,
    #[endianness(var)]
    pub flags: Option<u32>,
}

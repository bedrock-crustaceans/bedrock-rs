use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventorySource<V: ProtoVersion> {
    pub source_type: V::InventorySourceType,
}

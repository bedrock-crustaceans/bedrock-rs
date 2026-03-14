use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransaction<V: ProtoVersion> {
    pub action: Vec<V::InventoryAction>,
}

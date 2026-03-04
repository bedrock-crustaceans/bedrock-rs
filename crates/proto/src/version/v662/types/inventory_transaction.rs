use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct InventoryTransaction<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub action: Vec<V::InventoryAction>,
}

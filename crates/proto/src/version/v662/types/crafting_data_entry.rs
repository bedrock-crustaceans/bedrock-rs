use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataEntry<V: ProtoVersion> {
    pub crafting_type: V::CraftingDataEntryType,
}

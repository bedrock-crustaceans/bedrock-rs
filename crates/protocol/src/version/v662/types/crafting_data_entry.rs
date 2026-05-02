use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CraftingDataEntry<V: ProtoVersion> {
    pub crafting_type: V::CraftingDataEntryType,
}

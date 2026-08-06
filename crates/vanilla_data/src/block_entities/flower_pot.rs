use bedrock_level::subchunk::BlockDef;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct FlowerPot {
    pub plant_block: Option<BlockDef>,
}

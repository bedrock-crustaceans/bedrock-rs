use super::super::enums::StructureTemplateResponseType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 133)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureDataResponsePacket {
    pub structure_name: String,
    #[nbt]
    pub structure_nbt: Option<nbtx::Value>, // TODO: NBT Structure
    pub response_type: StructureTemplateResponseType,
}

// VERIFY: If this actually works
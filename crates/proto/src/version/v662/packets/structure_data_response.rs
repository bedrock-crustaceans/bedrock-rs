use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 133)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureDataResponsePacket<V: ProtoVersion> {
    pub structure_name: String,
    #[nbt]
    pub structure_nbt: Option<nbtx::Value>, // TODO: NBT Structure
    pub response_type: V::StructureTemplateResponseType,
}

// VERIFY: If this actually works

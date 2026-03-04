use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 133)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureDataResponsePacket<V: ProtoVersion> {
    pub structure_name: String,
    #[nbt]
    pub structure_nbt: Option<nbtx::Value>, // TODO: NBT Structure
    pub response_type: V::StructureTemplateResponseType,
}

// VERIFY: If this actually works
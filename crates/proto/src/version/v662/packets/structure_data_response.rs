use std::collections::HashMap;
use std::io::{Read, Write};
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;

#[packet(id = 133)]
#[derive(Clone, Debug)]
pub struct StructureDataResponsePacket<V: ProtoVersion> {
    pub structure_name: String,
    pub structure_nbt: Option<HashMap<String, nbtx::Value>>,
    pub response_type: V::StructureTemplateResponseType,
}

impl <V: ProtoVersion> ProtoCodec for StructureDataResponsePacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.structure_name.serialize(stream)?;
        match &self.structure_nbt {
            Some(nbt) => {
                bool::serialize(&true, stream)?;
                nbtx::to_bytes_in::<nbtx::NetworkLittleEndian>(stream, &nbt)?;
            },
            None => bool::serialize(&false, stream)?
        }
        self.response_type.serialize(stream)?;
        
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let structure_name = String::deserialize(stream)?;
        let structure_nbt = match bool::deserialize(stream)? {
            true => {
                Some(nbtx::from_bytes::<nbtx::NetworkLittleEndian, HashMap<String, nbtx::Value>>(stream)?)
            },
            false => None
        };
        let response_type = V::StructureTemplateResponseType::deserialize(stream)?;
        
        Ok(Self {
            structure_name,
            structure_nbt,
            response_type,
        })
    }

    fn size_hint(&self) -> usize {
        self.structure_name.size_hint()
        + match self.structure_nbt {
            Some(_) => bool::size_hint(&true) + 1,
            None => bool::size_hint(&false)
        }
        + self.response_type.size_hint()
    }
}

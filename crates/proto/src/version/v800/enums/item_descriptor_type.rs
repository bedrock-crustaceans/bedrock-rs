use std::io::Cursor;
use bedrockrs_macros::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE};

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum ItemDescriptorType {
    Invalid = 0,
    Default(DefaultDescriptor) = 1,
    Molang(MolangDescriptor) = 2,
    ItemTag(ItemTagDescriptor) = 3,
    Deferred(DeferredDescriptor) = 4,
    ComplexAlias(ComplexAliasDescriptor) = 5,
}

#[derive(Clone, Debug)]
pub struct DefaultDescriptor {
    pub item_id: i16,
    pub aux_value: i16
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MolangDescriptor {
    pub tag_expression: String,
    pub molang_version: u8,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ItemTagDescriptor {
    pub item_tag: String, 
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DeferredDescriptor {
    pub full_name: String,
    #[endianness(le)]
    pub aux_value: i16
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ComplexAliasDescriptor {
    pub name: String
}

impl ProtoCodec for DefaultDescriptor {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <i16 as ProtoCodecLE>::proto_serialize(&self.item_id, stream)?;
        if (self.item_id != 0) {
            <i16 as ProtoCodecLE>::proto_serialize(&self.aux_value, stream)?;
        }
        
        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let item_id = <i16 as ProtoCodecLE>::proto_deserialize(stream)?;
        let aux_value = match item_id != 0 {
            true => <i16 as ProtoCodecLE>::proto_deserialize(stream)?,
            false => 0
        };
        
        Ok(Self { item_id, aux_value })
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<i16>()
        + match self.aux_value != 0 {
            true => size_of::<i16>(),
            false => 0
        }
    }
}

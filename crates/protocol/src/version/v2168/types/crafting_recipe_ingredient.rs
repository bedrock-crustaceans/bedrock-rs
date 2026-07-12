use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};
use std::mem::size_of;

const AUX_WILDCARD: i32 = 0x7fff;

#[derive(Clone, Debug)]
pub struct CraftingRecipeIngredient {
    pub descriptor: CraftingItemDescriptor,
    pub stack_size: i32,
}

#[derive(Clone, Debug)]
pub enum CraftingItemDescriptor {
    Empty,
    Name { item_id: String, aux_value: i32 },
    Molang { tag_expression: String, molang_version: i16 },
    ItemTag { item_tag: String },
}

impl CraftingItemDescriptor {
    fn type_id(&self) -> &'static str {
        match self {
            CraftingItemDescriptor::Empty => "empty",
            CraftingItemDescriptor::Name { .. } => "name",
            CraftingItemDescriptor::Molang { .. } => "molang",
            CraftingItemDescriptor::ItemTag { .. } => "item_tag",
        }
    }
}

fn write_str<W: Write>(value: &str, stream: &mut W) -> Result<(), ProtoCodecError> {
    let len: u32 = value.len().try_into()?;
    <u32 as ProtoCodecVAR>::serialize(&len, stream)?;
    stream.write_all(value.as_bytes())?;

    Ok(())
}

fn to_aux_value(value: i32) -> i32 {
    match value == -1 {
        true => AUX_WILDCARD,
        false => value,
    }
}

fn from_aux_value(value: i32) -> i32 {
    match value == AUX_WILDCARD {
        true => -1,
        false => value,
    }
}

impl ProtoCodec for CraftingRecipeIngredient {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match &self.descriptor {
            CraftingItemDescriptor::Empty => {
                <u32 as ProtoCodecVAR>::serialize(&0, stream)?;
                <i32 as ProtoCodecVAR>::serialize(&to_aux_value(-1), stream)?;
            }
            descriptor => {
                <u32 as ProtoCodecVAR>::serialize(&1, stream)?;
                write_str(descriptor.type_id(), stream)?;

                match descriptor {
                    CraftingItemDescriptor::Name { item_id, aux_value } => {
                        item_id.serialize(stream)?;
                        <i32 as ProtoCodecVAR>::serialize(&to_aux_value(*aux_value), stream)?;
                    }
                    CraftingItemDescriptor::Molang {
                        tag_expression,
                        molang_version,
                    } => {
                        tag_expression.serialize(stream)?;
                        <i16 as ProtoCodecLE>::serialize(molang_version, stream)?;
                    }
                    CraftingItemDescriptor::ItemTag { item_tag } => {
                        item_tag.serialize(stream)?;
                        <i32 as ProtoCodecVAR>::serialize(&to_aux_value(-1), stream)?;
                    }
                    CraftingItemDescriptor::Empty => unreachable!(),
                }
            }
        }

        <i32 as ProtoCodecVAR>::serialize(&self.stack_size, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let variant = <u32 as ProtoCodecVAR>::deserialize(stream)?;

        let descriptor = match variant {
            0 => {
                <i32 as ProtoCodecVAR>::deserialize(stream)?;
                CraftingItemDescriptor::Empty
            }
            _ => {
                let type_id = String::deserialize(stream)?;

                match type_id.as_str() {
                    "empty" => CraftingItemDescriptor::Empty,
                    "name" => {
                        let item_id = String::deserialize(stream)?;
                        let aux_value =
                            from_aux_value(<i32 as ProtoCodecVAR>::deserialize(stream)?);
                        CraftingItemDescriptor::Name { item_id, aux_value }
                    }
                    "molang" => {
                        let tag_expression = String::deserialize(stream)?;
                        let molang_version = <i16 as ProtoCodecLE>::deserialize(stream)?;
                        CraftingItemDescriptor::Molang {
                            tag_expression,
                            molang_version,
                        }
                    }
                    "item_tag" => {
                        let item_tag = String::deserialize(stream)?;
                        <i32 as ProtoCodecVAR>::deserialize(stream)?;
                        CraftingItemDescriptor::ItemTag { item_tag }
                    }
                    other => {
                        return Err(ProtoCodecError::InvalidEnumID(
                            other.to_string(),
                            "CraftingItemDescriptor",
                        ));
                    }
                }
            }
        };

        let stack_size = <i32 as ProtoCodecVAR>::deserialize(stream)?;

        Ok(Self {
            descriptor,
            stack_size,
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<u32>()
            + match &self.descriptor {
                CraftingItemDescriptor::Empty => size_of::<i32>(),
                CraftingItemDescriptor::Name { item_id, .. } => {
                    4 + self.descriptor.type_id().len() + item_id.size_hint() + size_of::<i32>()
                }
                CraftingItemDescriptor::Molang { tag_expression, .. } => {
                    4 + self.descriptor.type_id().len()
                        + tag_expression.size_hint()
                        + size_of::<i16>()
                }
                CraftingItemDescriptor::ItemTag { item_tag } => {
                    4 + self.descriptor.type_id().len() + item_tag.size_hint() + size_of::<i32>()
                }
            }
            + size_of::<i32>()
    }
}

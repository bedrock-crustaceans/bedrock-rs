use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub enum ItemDescriptorType {
    Empty,
    Name {
        item_identifier: String,
        aux_value: i32,
    },
    Molang {
        tag_expression: String,
        molang_version: i16,
    },
    ItemTag {
        item_tag: String,
    },
}

impl ItemDescriptorType {
    fn type_id(&self) -> u32 {
        match self {
            ItemDescriptorType::Empty => 0,
            ItemDescriptorType::Name { .. } => 1,
            ItemDescriptorType::Molang { .. } => 2,
            ItemDescriptorType::ItemTag { .. } => 3,
        }
    }
}

impl ProtoCodec for ItemDescriptorType {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let type_id = self.type_id();
        <u32 as ProtoCodecVAR>::serialize(&type_id, stream)?;
        <u8 as ProtoCodec>::serialize(&(type_id as u8), stream)?;

        match self {
            ItemDescriptorType::Empty => {}
            ItemDescriptorType::Name {
                item_identifier,
                aux_value,
            } => {
                <String as ProtoCodec>::serialize(item_identifier, stream)?;
                <i32 as ProtoCodecVAR>::serialize(aux_value, stream)?;
            }
            ItemDescriptorType::Molang {
                tag_expression,
                molang_version,
            } => {
                <String as ProtoCodec>::serialize(tag_expression, stream)?;
                <i16 as ProtoCodecLE>::serialize(molang_version, stream)?;
            }
            ItemDescriptorType::ItemTag { item_tag } => {
                <String as ProtoCodec>::serialize(item_tag, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let type_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let _legacy_type_id = <u8 as ProtoCodec>::deserialize(stream)?;

        Ok(match type_id {
            0 => ItemDescriptorType::Empty,
            1 => ItemDescriptorType::Name {
                item_identifier: <String as ProtoCodec>::deserialize(stream)?,
                aux_value: <i32 as ProtoCodecVAR>::deserialize(stream)?,
            },
            2 => ItemDescriptorType::Molang {
                tag_expression: <String as ProtoCodec>::deserialize(stream)?,
                molang_version: <i16 as ProtoCodecLE>::deserialize(stream)?,
            },
            3 => ItemDescriptorType::ItemTag {
                item_tag: <String as ProtoCodec>::deserialize(stream)?,
            },
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    other.to_string(),
                    "ItemDescriptorType",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        <u32 as ProtoCodecVAR>::size_hint(&self.type_id())
            + size_of::<u8>()
            + match self {
                ItemDescriptorType::Empty => 0,
                ItemDescriptorType::Name {
                    item_identifier,
                    aux_value,
                } => {
                    <String as ProtoCodec>::size_hint(item_identifier)
                        + <i32 as ProtoCodecVAR>::size_hint(aux_value)
                }
                ItemDescriptorType::Molang {
                    tag_expression,
                    molang_version,
                } => {
                    <String as ProtoCodec>::size_hint(tag_expression)
                        + <i16 as ProtoCodecLE>::size_hint(molang_version)
                }
                ItemDescriptorType::ItemTag { item_tag } => {
                    <String as ProtoCodec>::size_hint(item_tag)
                }
            }
    }
}

use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 325)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerUpdateEntityOverridesPacket<V: ProtoVersion> {
    pub entity_unique_id: V::ActorUniqueID,
    #[endianness(var)]
    pub property_index: u32,
    pub update_type: UpdateType,
}

#[derive(Clone, Debug)]
pub enum UpdateType {
    ClearOverrides,
    RemoveOverride,
    SetIntOverride { value: i32 },
    SetFloatOverride { value: f32 },
}

impl UpdateType {
    fn type_id(&self) -> u32 {
        match self {
            Self::ClearOverrides => 0,
            Self::RemoveOverride => 1,
            Self::SetIntOverride { .. } => 2,
            Self::SetFloatOverride { .. } => 3,
        }
    }

    fn string_id(&self) -> &'static str {
        match self {
            Self::ClearOverrides => "clearoverrides",
            Self::RemoveOverride => "removeoverride",
            Self::SetIntOverride { .. } => "setintoverride",
            Self::SetFloatOverride { .. } => "setfloatoverride",
        }
    }
}

impl ProtoCodec for UpdateType {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <u32 as ProtoCodecVAR>::serialize(&self.type_id(), stream)?;
        <String as ProtoCodec>::serialize(&self.string_id().to_string(), stream)?;
        match self {
            Self::ClearOverrides | Self::RemoveOverride => {}
            Self::SetIntOverride { value } => <i32 as ProtoCodecLE>::serialize(value, stream)?,
            Self::SetFloatOverride { value } => <f32 as ProtoCodecLE>::serialize(value, stream)?,
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let type_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        <String as ProtoCodec>::deserialize(stream)?;

        Ok(match type_id {
            0 => Self::ClearOverrides,
            1 => Self::RemoveOverride,
            2 => Self::SetIntOverride {
                value: <i32 as ProtoCodecLE>::deserialize(stream)?,
            },
            3 => Self::SetFloatOverride {
                value: <f32 as ProtoCodecLE>::deserialize(stream)?,
            },
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    format!("{other}"),
                    "UpdateType",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.type_id())
            + ProtoCodec::size_hint(&self.string_id().to_string())
            + match self {
                Self::ClearOverrides | Self::RemoveOverride => 0,
                Self::SetIntOverride { value } => ProtoCodecLE::size_hint(value),
                Self::SetFloatOverride { value } => ProtoCodecLE::size_hint(value),
            }
    }
}

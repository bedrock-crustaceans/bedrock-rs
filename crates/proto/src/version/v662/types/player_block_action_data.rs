use crate::v662::enums::PlayerActionType;
use crate::version::versions::ProtoVersion;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct PlayerBlockActionData<V: ProtoVersion> {
    pub action_type: PlayerActionType,
    pub position: Option<V::BlockPos>,
    pub facing: Option<i32>,
}

impl <V: ProtoVersion> ProtoCodec for PlayerBlockActionData<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.action_type.serialize(stream)?;
        match self.action_type {
            PlayerActionType::StartDestroyBlock
            | PlayerActionType::AbortDestroyBlock
            | PlayerActionType::CrackBlock
            | PlayerActionType::PredictDestroyBlock
            | PlayerActionType::ContinueDestroyBlock => {
                V::BlockPos::serialize(self.position.as_ref().ok_or(ProtoCodecError::ExpectedSome("position"))?, stream)?;
                <i32 as ProtoCodecVAR>::serialize(self.facing.as_ref().ok_or(ProtoCodecError::ExpectedSome("facing"))?, stream)?;
            }
            _ => {}
        }
        
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let action_type = PlayerActionType::deserialize(stream)?;
        let (position, facing) = match action_type {
            PlayerActionType::StartDestroyBlock
            | PlayerActionType::AbortDestroyBlock
            | PlayerActionType::CrackBlock
            | PlayerActionType::PredictDestroyBlock
            | PlayerActionType::ContinueDestroyBlock => {
                (
                    Some(V::BlockPos::deserialize(stream)?),
                    Some(<i32 as ProtoCodecVAR>::deserialize(stream)?),
                )
            }
            _ => (None, None)
        };
        
        Ok(Self {
            action_type,
            position,
            facing
        })
    }

    fn size_hint(&self) -> usize {
        self.action_type.size_hint()
        + match self.action_type {
            PlayerActionType::StartDestroyBlock
            | PlayerActionType::AbortDestroyBlock
            | PlayerActionType::CrackBlock
            | PlayerActionType::PredictDestroyBlock
            | PlayerActionType::ContinueDestroyBlock => {
                self.position.as_ref().map_or(0, V::BlockPos::size_hint)
                + self.facing.as_ref().map_or(0, <i32 as ProtoCodecVAR>::size_hint)
            },
            _ => 0
        }
    }
}

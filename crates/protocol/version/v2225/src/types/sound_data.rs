use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub enum SoundData {
    Stop,
    SetVolume { volume: f32 },
    SetPitch { pitch: f32 },
    Fade { duration: f32, target_volume: f32 },
    SeekTo { seconds: f32 },
    Pause,
    Resume,
}

impl SoundData {
    fn type_id(&self) -> u8 {
        match self {
            Self::Stop => 0,
            Self::SetVolume { .. } => 1,
            Self::SetPitch { .. } => 2,
            Self::Fade { .. } => 3,
            Self::SeekTo { .. } => 4,
            Self::Pause => 5,
            Self::Resume => 6,
        }
    }
}

impl ProtoCodec for SoundData {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let type_id = self.type_id();
        <u32 as ProtoCodecVAR>::serialize(&u32::from(type_id), stream)?;
        <u8 as ProtoCodec>::serialize(&type_id, stream)?;
        match self {
            Self::Stop | Self::Pause | Self::Resume => {}
            Self::SetVolume { volume: value }
            | Self::SetPitch { pitch: value }
            | Self::SeekTo { seconds: value } => {
                <f32 as ProtoCodecLE>::serialize(value, stream)?;
            }
            Self::Fade {
                duration,
                target_volume,
            } => {
                <f32 as ProtoCodecLE>::serialize(duration, stream)?;
                <f32 as ProtoCodecLE>::serialize(target_volume, stream)?;
            }
        }
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let type_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let echo = <u8 as ProtoCodec>::deserialize(stream)?;
        if u32::from(echo) != type_id {
            return Err(ProtoCodecError::FormatMismatch(
                "sound data type and its echo differ",
            ));
        }
        Ok(match type_id {
            0 => Self::Stop,
            1 => Self::SetVolume {
                volume: <f32 as ProtoCodecLE>::deserialize(stream)?,
            },
            2 => Self::SetPitch {
                pitch: <f32 as ProtoCodecLE>::deserialize(stream)?,
            },
            3 => Self::Fade {
                duration: <f32 as ProtoCodecLE>::deserialize(stream)?,
                target_volume: <f32 as ProtoCodecLE>::deserialize(stream)?,
            },
            4 => Self::SeekTo {
                seconds: <f32 as ProtoCodecLE>::deserialize(stream)?,
            },
            5 => Self::Pause,
            6 => Self::Resume,
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    other.to_string(),
                    "SoundData",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        let payload = match self {
            Self::Stop | Self::Pause | Self::Resume => 0,
            Self::Fade { .. } => 2 * size_of::<f32>(),
            _ => size_of::<f32>(),
        };
        <u32 as ProtoCodecVAR>::size_hint(&u32::from(self.type_id())) + size_of::<u8>() + payload
    }
}

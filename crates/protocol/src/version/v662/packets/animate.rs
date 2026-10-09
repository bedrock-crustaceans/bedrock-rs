use bedrock_protocol_core::ProtoCodecVAR;
use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Cursor, Read, Write, copy};

#[packet(id = 44)]
#[derive(Clone, Debug)]
pub struct AnimatePacket<V: ProtoVersion> {
    pub action: AnimateAction,
    pub target_runtime_id: V::ActorRuntimeID,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
#[allow(clippy::enum_variant_names)]
pub enum AnimateAction {
    NoAction = 0,
    Swing = 1,
    WakeUp = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
    RowRight {
        #[endianness(le)]
        rowing_time: f32,
    } = 128,
    RowLeft {
        #[endianness(le)]
        rowing_time: f32,
    } = 129,
}

impl<V: ProtoVersion> ProtoCodec for AnimatePacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <AnimateAction as ProtoCodec>::serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        <i32 as ProtoCodecVAR>::serialize(&(<i32 as ProtoCodecVAR>::deserialize(&mut action_cursor)?), stream)?;
        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.target_runtime_id, stream)?;
        copy(&mut action_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        <i32 as ProtoCodecVAR>::serialize(&(<i32 as ProtoCodecVAR>::deserialize(stream)?), &mut action_stream)?;
        let target_runtime_id = <V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?;
        stream.read_to_end(&mut action_stream)?;

        let mut action_cursor = Cursor::new(action_stream.as_slice());
        let action = <AnimateAction as ProtoCodec>::deserialize(&mut action_cursor)?;

        Ok(Self {
            action,
            target_runtime_id,
        })
    }

    fn size_hint(&self) -> usize {
        self.action.size_hint() + self.target_runtime_id.size_hint()
    }
}

// TODO: verify ProtoCodec impl

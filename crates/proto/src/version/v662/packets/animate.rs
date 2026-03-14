use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Cursor, Read, Write, copy};
use varint_rs::{VarintReader, VarintWriter};

#[packet(id = 44)]
#[derive(Clone, Debug)]
pub struct AnimatePacket<V: ProtoVersion> {
    pub action: Action,
    pub target_runtime_id: V::ActorRuntimeID,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
#[allow(clippy::enum_variant_names)]
pub enum Action {
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
        <Action as ProtoCodec>::serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i32_varint(action_cursor.read_i32_varint()?)?;
        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.target_runtime_id, stream)?;
        copy(&mut action_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i32_varint(stream.read_i32_varint()?)?;
        let target_runtime_id = <V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?;
        stream.read_to_end(&mut action_stream)?;

        let mut action_cursor = Cursor::new(action_stream.as_slice());
        let action = <Action as ProtoCodec>::deserialize(&mut action_cursor)?;

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

use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::{Cursor, Read};
use varint_rs::{VarintReader, VarintWriter};

#[gamepacket(id = 44)]
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
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <Action as ProtoCodec>::proto_serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i32_varint(action_cursor.read_i32_varint()?)?;
        <V::ActorRuntimeID as ProtoCodec>::proto_serialize(&self.target_runtime_id, stream)?;
        action_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i32_varint(stream.read_i32_varint()?)?;
        let target_runtime_id = <V::ActorRuntimeID as ProtoCodec>::proto_deserialize(stream)?;
        stream.read_to_end(&mut action_stream)?;

        let mut action_cursor = Cursor::new(action_stream.as_slice());
        let action = <Action as ProtoCodec>::proto_deserialize(&mut action_cursor)?;

        Ok(Self {
            action,
            target_runtime_id,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.action.get_size_prediction() + self.target_runtime_id.get_size_prediction()
    }
}

// TODO: verify ProtoCodec impl

use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};

#[packet(id = 33)]
#[derive(Clone, Debug)]
pub struct InteractPacket<V: ProtoVersion> {
    pub action: Action,
    pub target_runtime_id: V::ActorRuntimeID,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Action {
    Invalid = 0,
    Interact = 1,
    Damage = 2,
    StopRiding {
        #[endianness(le)]
        position_x: f32,
        #[endianness(le)]
        position_y: f32,
        #[endianness(le)]
        position_z: f32,
    } = 3,
    InteractUpdate {
        #[endianness(le)]
        position_x: f32,
        #[endianness(le)]
        position_y: f32,
        #[endianness(le)]
        position_z: f32,
    } = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}

impl<V: ProtoVersion> ProtoCodec for InteractPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <Action as ProtoCodec>::serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i8(action_cursor.read_i8()?)?;
        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.target_runtime_id, stream)?;
        copy(&mut action_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i8(stream.read_i8()?)?;
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

// VERIFY: ProtoCodec impl

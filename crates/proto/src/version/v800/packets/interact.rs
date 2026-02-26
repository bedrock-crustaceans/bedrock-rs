use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

#[gamepacket(id = 33)]
#[derive(Clone, Debug)]
pub struct InteractPacket {
    pub action: Action,
    pub target_runtime_id: ActorRuntimeID,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum Action {
    Invalid = 0,
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

impl ProtoCodec for InteractPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();
        <Action as ProtoCodec>::proto_serialize(&self.action, &mut action_stream)?;
        let mut action_cursor = Cursor::new(action_stream.as_slice());

        stream.write_i8(action_cursor.read_i8()?)?;
        <ActorRuntimeID as ProtoCodec>::proto_serialize(&self.target_runtime_id, stream)?;
        action_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut action_stream: Vec<u8> = Vec::new();

        action_stream.write_i8(stream.read_i8()?)?;
        let target_runtime_id = <ActorRuntimeID as ProtoCodec>::proto_deserialize(stream)?;
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

// VERIFY: ProtoCodec impl

use crate::ProtoVersion;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE};
use std::io::{Read, Write};

// Header bitfield flags
const FLAG_HAS_X: u16 = 0x01;
const FLAG_HAS_Y: u16 = 0x02;
const FLAG_HAS_Z: u16 = 0x04;
const FLAG_HAS_PITCH: u16 = 0x08; // rotation_x
const FLAG_HAS_YAW: u16 = 0x10; // rotation_y
const FLAG_HAS_HEAD_YAW: u16 = 0x20; // rotation_y_head

#[derive(Clone, Debug)]
pub struct MoveActorDeltaData<V: ProtoVersion> {
    pub actor_runtime_id: V::ActorRuntimeID,
    pub header: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: i8,
    pub rotation_y: i8,
    pub rotation_y_head: i8,
}

impl<V: ProtoVersion> ProtoCodec for MoveActorDeltaData<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.actor_runtime_id, stream)?;
        <u16 as ProtoCodecLE>::serialize(&self.header, stream)?;

        if self.header & FLAG_HAS_X != 0 {
            <f32 as ProtoCodecLE>::serialize(&self.position_x, stream)?;
        }
        if self.header & FLAG_HAS_Y != 0 {
            <f32 as ProtoCodecLE>::serialize(&self.position_y, stream)?;
        }
        if self.header & FLAG_HAS_Z != 0 {
            <f32 as ProtoCodecLE>::serialize(&self.position_z, stream)?;
        }
        if self.header & FLAG_HAS_PITCH != 0 {
            <i8 as ProtoCodec>::serialize(&self.rotation_x, stream)?;
        }
        if self.header & FLAG_HAS_YAW != 0 {
            <i8 as ProtoCodec>::serialize(&self.rotation_y, stream)?;
        }
        if self.header & FLAG_HAS_HEAD_YAW != 0 {
            <i8 as ProtoCodec>::serialize(&self.rotation_y_head, stream)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let actor_runtime_id = <V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?;
        let header = <u16 as ProtoCodecLE>::deserialize(stream)?;

        let position_x = if header & FLAG_HAS_X != 0 {
            <f32 as ProtoCodecLE>::deserialize(stream)?
        } else {
            0.0
        };
        let position_y = if header & FLAG_HAS_Y != 0 {
            <f32 as ProtoCodecLE>::deserialize(stream)?
        } else {
            0.0
        };
        let position_z = if header & FLAG_HAS_Z != 0 {
            <f32 as ProtoCodecLE>::deserialize(stream)?
        } else {
            0.0
        };
        let rotation_x = if header & FLAG_HAS_PITCH != 0 {
            <i8 as ProtoCodec>::deserialize(stream)?
        } else {
            0
        };
        let rotation_y = if header & FLAG_HAS_YAW != 0 {
            <i8 as ProtoCodec>::deserialize(stream)?
        } else {
            0
        };
        let rotation_y_head = if header & FLAG_HAS_HEAD_YAW != 0 {
            <i8 as ProtoCodec>::deserialize(stream)?
        } else {
            0
        };

        Ok(Self {
            actor_runtime_id,
            header,
            position_x,
            position_y,
            position_z,
            rotation_x,
            rotation_y,
            rotation_y_head,
        })
    }

    fn size_hint(&self) -> usize {
        let mut size = <V::ActorRuntimeID as ProtoCodec>::size_hint(&self.actor_runtime_id);
        size += 2; // header (u16)

        if self.header & FLAG_HAS_X != 0 {
            size += 4; // position_x (f32)
        }
        if self.header & FLAG_HAS_Y != 0 {
            size += 4; // position_y (f32)
        }
        if self.header & FLAG_HAS_Z != 0 {
            size += 4; // position_z (f32)
        }
        if self.header & FLAG_HAS_PITCH != 0 {
            size += 1; // rotation_x (i8)
        }
        if self.header & FLAG_HAS_YAW != 0 {
            size += 1; // rotation_y (i8)
        }
        if self.header & FLAG_HAS_HEAD_YAW != 0 {
            size += 1; // rotation_y_head (i8)
        }

        size
    }
}

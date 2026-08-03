use crate::ProtoVersion;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::ProtoCodec;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const FLAG_HAS_X: u16 = 0x01;
pub const FLAG_HAS_Y: u16 = 0x02;
pub const FLAG_HAS_Z: u16 = 0x04;
pub const FLAG_HAS_ROT_X: u16 = 0x08;
pub const FLAG_HAS_ROT_Y: u16 = 0x10;
pub const FLAG_HAS_ROT_Z: u16 = 0x20;

#[derive(Clone, Debug)]
pub struct MoveActorDeltaData<V: ProtoVersion> {
    pub actor_runtime_id: V::ActorRuntimeID,
    pub header: u16,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub position_z: Option<f32>,
    pub rotation_x: Option<i8>,
    pub rotation_y: Option<i8>,
    pub rotation_y_head: Option<i8>,
}

impl<V: ProtoVersion> ProtoCodec for MoveActorDeltaData<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.actor_runtime_id.serialize(stream)?;
        stream.write_u16::<LittleEndian>(self.header)?;

        for (flag, value) in [
            (FLAG_HAS_X, self.position_x),
            (FLAG_HAS_Y, self.position_y),
            (FLAG_HAS_Z, self.position_z),
        ] {
            if self.header & flag != 0 {
                stream.write_f32::<LittleEndian>(value.unwrap_or_default())?;
            }
        }

        for (flag, value) in [
            (FLAG_HAS_ROT_X, self.rotation_x),
            (FLAG_HAS_ROT_Y, self.rotation_y),
            (FLAG_HAS_ROT_Z, self.rotation_y_head),
        ] {
            if self.header & flag != 0 {
                stream.write_i8(value.unwrap_or_default())?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let actor_runtime_id = V::ActorRuntimeID::deserialize(stream)?;
        let header = stream.read_u16::<LittleEndian>()?;

        let mut position = [None; 3];
        for (index, flag) in [FLAG_HAS_X, FLAG_HAS_Y, FLAG_HAS_Z].into_iter().enumerate() {
            if header & flag != 0 {
                position[index] = Some(stream.read_f32::<LittleEndian>()?);
            }
        }

        let mut rotation = [None; 3];
        for (index, flag) in [FLAG_HAS_ROT_X, FLAG_HAS_ROT_Y, FLAG_HAS_ROT_Z]
            .into_iter()
            .enumerate()
        {
            if header & flag != 0 {
                rotation[index] = Some(stream.read_i8()?);
            }
        }

        Ok(Self {
            actor_runtime_id,
            header,
            position_x: position[0],
            position_y: position[1],
            position_z: position[2],
            rotation_x: rotation[0],
            rotation_y: rotation[1],
            rotation_y_head: rotation[2],
        })
    }

    fn size_hint(&self) -> usize {
        self.actor_runtime_id.size_hint() + 17
    }
}

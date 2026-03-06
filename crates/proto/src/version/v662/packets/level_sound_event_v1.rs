use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};
use vek::Vec3;

#[packet(id = 24)]
#[derive(Clone, Debug)]
pub struct LevelSoundEventV1Packet<V: ProtoVersion> {
    pub event_id: V::LevelSoundEventType,
    pub position: Vec3<f32>,
    pub data: i32,
    pub actor_type: V::ActorType,
    pub baby_mob: bool,
    pub global: bool,
}

impl<V: ProtoVersion> ProtoCodec for LevelSoundEventV1Packet<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut event_id_stream: Vec<u8> = Vec::new();
        V::LevelSoundEventType::serialize(&self.event_id, &mut event_id_stream)?;
        let mut event_id_cursor = Cursor::new(event_id_stream.as_slice());

        stream.write_i8(event_id_cursor.read_u32_varint()? as i8)?;
        <Vec3<f32> as ProtoCodecLE>::serialize(&self.position, stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.data, stream)?;
        <V::ActorType as ProtoCodec>::serialize(&self.actor_type, stream)?;
        <bool as ProtoCodec>::serialize(&self.baby_mob, stream)?;
        <bool as ProtoCodec>::serialize(&self.global, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut event_id_stream: Vec<u8> = Vec::new();
        event_id_stream.write_u32_varint(stream.read_i8()? as u32)?;
        let mut event_id_cursor = Cursor::new(event_id_stream.as_slice());

        let event_id = V::LevelSoundEventType::deserialize(&mut event_id_cursor)?;
        let position = <Vec3<f32> as ProtoCodecLE>::deserialize(stream)?;
        let data = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let actor_type = <V::ActorType as ProtoCodec>::deserialize(stream)?;
        let baby_mob = <bool as ProtoCodec>::deserialize(stream)?;
        let global = <bool as ProtoCodec>::deserialize(stream)?;

        Ok(Self {
            event_id,
            position,
            data,
            actor_type,
            baby_mob,
            global,
        })
    }

    fn size_hint(&self) -> usize {
        size_of::<i8>()
            + ProtoCodecLE::size_hint(&self.position)
            + ProtoCodecVAR::size_hint(&self.data)
            + self.actor_type.size_hint()
            + self.baby_mob.size_hint()
            + self.global.size_hint()
    }
}

// VERIFY: ProtoCodec impl

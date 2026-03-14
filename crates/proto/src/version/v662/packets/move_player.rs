use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write, copy};
use vek::{Vec2, Vec3};

#[packet(id = 19)]
#[derive(Clone, Debug)]
pub struct MovePlayerPacket<V: ProtoVersion> {
    pub player_runtime_id: V::ActorRuntimeID,
    pub position: Vec3<f32>,
    pub rotation: Vec2<f32>,
    pub y_head_rotation: f32,
    pub position_mode: V::PlayerPositionMode,
    pub on_ground: bool,
    pub riding_runtime_id: V::ActorRuntimeID,
    pub tick: u64,
}

impl<V: ProtoVersion> ProtoCodec for MovePlayerPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut position_mode_stream: Vec<u8> = Vec::new();
        V::PlayerPositionMode::serialize(&self.position_mode, &mut position_mode_stream)?;
        let mut position_mode_cursor = Cursor::new(position_mode_stream.as_slice());

        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.player_runtime_id, stream)?;
        <Vec3<f32> as ProtoCodecLE>::serialize(&self.position, stream)?;
        <Vec2<f32> as ProtoCodecLE>::serialize(&self.rotation, stream)?;
        <f32 as ProtoCodecLE>::serialize(&self.y_head_rotation, stream)?;
        stream.write_i8(position_mode_cursor.read_i8()?)?;
        <bool as ProtoCodec>::serialize(&self.on_ground, stream)?;
        <V::ActorRuntimeID as ProtoCodec>::serialize(&self.riding_runtime_id, stream)?;
        copy(&mut position_mode_cursor, stream)?;
        <u64 as ProtoCodecVAR>::serialize(&self.tick, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut sub_stream = Vec::<u8>::new();

        let player_runtime_id = <V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?;
        let position = <Vec3<f32> as ProtoCodecLE>::deserialize(stream)?;
        let rotation = <Vec2<f32> as ProtoCodecLE>::deserialize(stream)?;
        let y_head_rotation = <f32 as ProtoCodecLE>::deserialize(stream)?;
        sub_stream.write_i8(stream.read_i8()?)?;
        let on_ground = <bool as ProtoCodec>::deserialize(stream)?;
        let riding_runtime_id = <V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?;
        stream.read_to_end(&mut sub_stream)?;

        let mut sub_cursor = Cursor::new(sub_stream.as_slice());
        let position_mode = V::PlayerPositionMode::deserialize(&mut sub_cursor)?;
        let tick = <u64 as ProtoCodecVAR>::deserialize(&mut sub_cursor)?;

        Ok(Self {
            player_runtime_id,
            position,
            rotation,
            y_head_rotation,
            position_mode,
            on_ground,
            riding_runtime_id,
            tick,
        })
    }

    fn size_hint(&self) -> usize {
        self.player_runtime_id.size_hint()
            + ProtoCodecLE::size_hint(&self.position)
            + ProtoCodecLE::size_hint(&self.rotation)
            + ProtoCodecLE::size_hint(&self.y_head_rotation)
            + self.position_mode.size_hint()
            + self.on_ground.size_hint()
            + self.riding_runtime_id.size_hint()
            + ProtoCodecVAR::size_hint(&self.tick)
    }
}

// VERIFY: ProtoCodec impl

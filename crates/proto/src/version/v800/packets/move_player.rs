use super::super::enums::PlayerPositionMode;
use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use bedrockrs_shared::actor_runtime_id::ActorRuntimeID;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use vek::{Vec2, Vec3};

#[gamepacket(id = 19)]
#[derive(Clone, Debug)]
pub struct MovePlayerPacket {
    pub player_runtime_id: ActorRuntimeID,
    pub position: Vec3<f32>,
    pub rotation: Vec2<f32>,
    pub y_head_rotation: f32,
    pub position_mode: PlayerPositionMode,
    pub on_ground: bool,
    pub riding_runtime_id: ActorRuntimeID,
    pub tick: u64,
}

impl ProtoCodec for MovePlayerPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut position_mode_stream: Vec<u8> = Vec::new();
        PlayerPositionMode::proto_serialize(
            &self.position_mode,
            &mut position_mode_stream,
        )?;
        let mut position_mode_cursor = Cursor::new(position_mode_stream.as_slice());

        <ActorRuntimeID as ProtoCodec>::proto_serialize(&self.player_runtime_id, stream)?;
        <Vec3<f32> as ProtoCodecLE>::proto_serialize(&self.position, stream)?;
        <Vec2<f32> as ProtoCodecLE>::proto_serialize(&self.rotation, stream)?;
        <f32 as ProtoCodecLE>::proto_serialize(&self.y_head_rotation, stream)?;
        stream.write_i8(position_mode_cursor.read_i8()?)?;
        <bool as ProtoCodec>::proto_serialize(&self.on_ground, stream)?;
        <ActorRuntimeID as ProtoCodec>::proto_serialize(&self.riding_runtime_id, stream)?;
        position_mode_cursor.read_to_end(stream)?;
        <u64 as ProtoCodecVAR>::proto_serialize(&self.tick, stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut sub_stream = Vec::<u8>::new();

        let player_runtime_id = <ActorRuntimeID as ProtoCodec>::proto_deserialize(stream)?;
        let position = <Vec3<f32> as ProtoCodecLE>::proto_deserialize(stream)?;
        let rotation = <Vec2<f32> as ProtoCodecLE>::proto_deserialize(stream)?;
        let y_head_rotation = <f32 as ProtoCodecLE>::proto_deserialize(stream)?;
        sub_stream.write_i8(stream.read_i8()?)?;
        let on_ground = <bool as ProtoCodec>::proto_deserialize(stream)?;
        let riding_runtime_id = <ActorRuntimeID as ProtoCodec>::proto_deserialize(stream)?;
        stream.read_to_end(&mut sub_stream)?;

        let mut sub_cursor = Cursor::new(sub_stream.as_slice());
        let position_mode =
            PlayerPositionMode::proto_deserialize(
                &mut sub_cursor,
            )?;
        let tick = <u64 as ProtoCodecVAR>::proto_deserialize(&mut sub_cursor)?;

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

    fn get_size_prediction(&self) -> usize {
        self.player_runtime_id.get_size_prediction()
            + ProtoCodecLE::get_size_prediction(&self.position)
            + ProtoCodecLE::get_size_prediction(&self.rotation)
            + ProtoCodecLE::get_size_prediction(&self.y_head_rotation)
            + self.position_mode.get_size_prediction()
            + self.on_ground.get_size_prediction()
            + self.riding_runtime_id.get_size_prediction()
            + ProtoCodecVAR::get_size_prediction(&self.tick)
    }
}

// VERIFY: ProtoCodec impl

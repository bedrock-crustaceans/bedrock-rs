use super::super::types::{ActorUniqueID, BlockPos, MapDecoration, MapItemTrackedActorUniqueID};
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::{Cursor, Read};
use varint_rs::{VarintReader, VarintWriter};

#[gamepacket(id = 67)]
#[derive(Clone, Debug)]
pub struct ClientBoundMapItemDataPacket {
    pub map_id: ActorUniqueID,
    pub type_flags: Type,
    pub dimension: i8,
    pub is_locked: bool,
    pub map_origin: BlockPos,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct PixelsEntry {
    #[endianness(var)]
    pub pixel: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum Type {
    Invalid = 0x0,
    TextureUpdate {
        #[endianness(var)]
        texture_width: i32,
        #[endianness(var)]
        texture_height: i32,
        #[endianness(var)]
        x_tex_coordinate: i32,
        #[endianness(var)]
        y_tex_coordinate: i32,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        pixels: Vec<PixelsEntry>,
    } = 0x2,
    DecorationUpdate {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        actor_ids: Vec<MapItemTrackedActorUniqueID>,
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        decoration_list: Vec<MapDecoration>,
    } = 0x4,
    Creation {
        #[vec_repr(u32)]
        #[vec_endianness(var)]
        map_id_list: Vec<ActorUniqueID>,
    } = 0x8,
}

impl ProtoCodec for ClientBoundMapItemDataPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        let mut type_flags_stream: Vec<u8> = Vec::new();
        <Type as ProtoCodec>::proto_serialize(&self.type_flags, &mut type_flags_stream)?;
        let mut type_flags_cursor = Cursor::new(type_flags_stream.as_slice());

        <ActorUniqueID as ProtoCodec>::proto_serialize(&self.map_id, stream)?;
        stream.write_u32_varint(type_flags_cursor.read_u32_varint()?)?;
        <i8 as ProtoCodec>::proto_serialize(&self.dimension, stream)?;
        <bool as ProtoCodec>::proto_serialize(&self.is_locked, stream)?;
        <BlockPos as ProtoCodec>::proto_serialize(&self.map_origin, stream)?;
        type_flags_cursor.read_to_end(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut type_flags_stream: Vec<u8> = Vec::new();

        let map_id = <ActorUniqueID as ProtoCodec>::proto_deserialize(stream)?;
        type_flags_stream.write_u32_varint(stream.read_u32_varint()?)?;
        let dimension = <i8 as ProtoCodec>::proto_deserialize(stream)?;
        let is_locked = <bool as ProtoCodec>::proto_deserialize(stream)?;
        let map_origin = <BlockPos as ProtoCodec>::proto_deserialize(stream)?;
        stream.read_to_end(&mut type_flags_stream)?;

        let mut type_flags_cursor = Cursor::new(type_flags_stream.as_slice());
        let type_flags = <Type as ProtoCodec>::proto_deserialize(&mut type_flags_cursor)?;

        Ok(Self {
            map_id,
            type_flags,
            dimension,
            is_locked,
            map_origin,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.map_id.get_size_prediction()
            + self.type_flags.get_size_prediction()
            + self.dimension.get_size_prediction()
            + self.is_locked.get_size_prediction()
            + self.map_origin.get_size_prediction()
    }
}

// VERIFY: ProtoCodec impl

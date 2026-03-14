use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Cursor, Read, Write, copy};
use varint_rs::{VarintReader, VarintWriter};

#[packet(id = 67)]
#[derive(Clone, Debug)]
pub struct ClientBoundMapItemDataPacket<V: ProtoVersion> {
    pub map_id: V::ActorUniqueID,
    pub type_flags: Type<V>,
    pub dimension: i8,
    pub is_locked: bool,
    pub map_origin: V::BlockPos,
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
pub enum Type<V: ProtoVersion> {
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
        pixels: Vec<PixelsEntry>,
    } = 0x2,
    DecorationUpdate {
        actor_ids: Vec<V::MapItemTrackedActorUniqueID>,
        decoration_list: Vec<V::MapDecoration>,
    } = 0x4,
    Creation {
        map_id_list: Vec<V::ActorUniqueID>,
    } = 0x8,
}

impl<V: ProtoVersion> ProtoCodec for ClientBoundMapItemDataPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut type_flags_stream: Vec<u8> = Vec::new();
        <Type<V> as ProtoCodec>::serialize(&self.type_flags, &mut type_flags_stream)?;
        let mut type_flags_cursor = Cursor::new(type_flags_stream.as_slice());

        <V::ActorUniqueID as ProtoCodec>::serialize(&self.map_id, stream)?;
        stream.write_u32_varint(type_flags_cursor.read_u32_varint()?)?;
        <i8 as ProtoCodec>::serialize(&self.dimension, stream)?;
        <bool as ProtoCodec>::serialize(&self.is_locked, stream)?;
        <V::BlockPos as ProtoCodec>::serialize(&self.map_origin, stream)?;
        copy(&mut type_flags_cursor, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut type_flags_stream: Vec<u8> = Vec::new();

        let map_id = <V::ActorUniqueID as ProtoCodec>::deserialize(stream)?;
        type_flags_stream.write_u32_varint(stream.read_u32_varint()?)?;
        let dimension = <i8 as ProtoCodec>::deserialize(stream)?;
        let is_locked = <bool as ProtoCodec>::deserialize(stream)?;
        let map_origin = <V::BlockPos as ProtoCodec>::deserialize(stream)?;
        stream.read_to_end(&mut type_flags_stream)?;

        let mut type_flags_cursor = Cursor::new(type_flags_stream.as_slice());
        let type_flags = <Type<V> as ProtoCodec>::deserialize(&mut type_flags_cursor)?;

        Ok(Self {
            map_id,
            type_flags,
            dimension,
            is_locked,
            map_origin,
        })
    }

    fn size_hint(&self) -> usize {
        self.map_id.size_hint()
            + self.type_flags.size_hint()
            + self.dimension.size_hint()
            + self.is_locked.size_hint()
            + self.map_origin.size_hint()
    }
}

// TODO: verify ProtoCodec impl

use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};
use std::marker::PhantomData;

#[packet(id = 86)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    pub position: PlaySoundPosition<V>,
    #[endianness(le)]
    pub volume: f32,
    #[endianness(le)]
    pub pitch: f32,
    #[endianness(var)]
    pub loop_count: i32,
    pub bypass_listener_range_check: bool,
    #[endianness(le)]
    pub server_sound_handle: Option<i64>,
    #[endianness(le)]
    pub playback_position_seconds: Option<f32>,
}

/// PlaySound stores position as signed varint coordinates scaled by 8.
#[derive(Clone, Debug)]
pub struct PlaySoundPosition<V: ProtoVersion> {
    pub x8: i32,
    pub y8: i32,
    pub z8: i32,
    _version: PhantomData<fn() -> V>,
}

impl<V: ProtoVersion> PlaySoundPosition<V> {
    pub const FIXED_POINT_SCALE: f32 = 8.0;

    pub const fn from_raw_x8(x8: i32, y8: i32, z8: i32) -> Self {
        Self {
            x8,
            y8,
            z8,
            _version: PhantomData,
        }
    }

    pub fn as_block_coords(&self) -> (f32, f32, f32) {
        (
            self.x8 as f32 / Self::FIXED_POINT_SCALE,
            self.y8 as f32 / Self::FIXED_POINT_SCALE,
            self.z8 as f32 / Self::FIXED_POINT_SCALE,
        )
    }
}

impl<V: ProtoVersion> ProtoCodec for PlaySoundPosition<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <i32 as ProtoCodecVAR>::serialize(&self.x8, stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.y8, stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.z8, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(Self::from_raw_x8(
            <i32 as ProtoCodecVAR>::deserialize(stream)?,
            <i32 as ProtoCodecVAR>::deserialize(stream)?,
            <i32 as ProtoCodecVAR>::deserialize(stream)?,
        ))
    }

    fn size_hint(&self) -> usize {
        <i32 as ProtoCodecVAR>::size_hint(&self.x8)
            + <i32 as ProtoCodecVAR>::size_hint(&self.y8)
            + <i32 as ProtoCodecVAR>::size_hint(&self.z8)
    }
}

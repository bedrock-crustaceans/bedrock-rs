use crate::ProtoVersion;
use bedrock_macros::packet;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE};
use std::io::{Read, Write};
use std::marker::PhantomData;
use varint_rs::{VarintReader, VarintWriter};

#[packet(id = 86)]
#[derive(Clone, Debug)]
pub struct PlaySoundPacket<V: ProtoVersion> {
    pub name: String,
    /// World-space position. On the wire this is encoded as a fixed-point block
    /// position (each coordinate multiplied by 8) using signed varints
    pub position: (f32, f32, f32),
    pub volume: f32,
    pub pitch: f32,
    pub server_sound_handle: Option<i64>,
    pub _marker: PhantomData<V>,
}

impl<V: ProtoVersion> ProtoCodec for PlaySoundPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <String as ProtoCodec>::serialize(&self.name, stream)?;

        // Position is encoded as a block position scaled by 8 (signed varint per coord)
        stream.write_i32_varint((self.position.0 * 8.0).round() as i32)?;
        stream.write_i32_varint((self.position.1 * 8.0).round() as i32)?;
        stream.write_i32_varint((self.position.2 * 8.0).round() as i32)?;

        <f32 as ProtoCodecLE>::serialize(&self.volume, stream)?;
        <f32 as ProtoCodecLE>::serialize(&self.pitch, stream)?;
        <Option<i64> as ProtoCodecLE>::serialize(&self.server_sound_handle, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let name = <String as ProtoCodec>::deserialize(stream)?;

        let position = (
            stream.read_i32_varint()? as f32 / 8.0,
            stream.read_i32_varint()? as f32 / 8.0,
            stream.read_i32_varint()? as f32 / 8.0,
        );

        let volume = <f32 as ProtoCodecLE>::deserialize(stream)?;
        let pitch = <f32 as ProtoCodecLE>::deserialize(stream)?;
        let server_sound_handle = <Option<i64> as ProtoCodecLE>::deserialize(stream)?;

        Ok(Self {
            name,
            position,
            volume,
            pitch,
            server_sound_handle,
            _marker: PhantomData,
        })
    }

    fn size_hint(&self) -> usize {
        let mut size = <String as ProtoCodec>::size_hint(&self.name);
        // 3 signed varints (1..=5 bytes each); use the max as a conservative hint
        size += 5 * 3;
        size += 4; // volume (f32)
        size += 4; // pitch (f32)
        size += <Option<i64> as ProtoCodecLE>::size_hint(&self.server_sound_handle);

        size
    }
}

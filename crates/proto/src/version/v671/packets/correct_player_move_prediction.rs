use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Cursor, Read, Write, copy};

#[packet(id = 161)]
#[derive(Clone, Debug)]
pub struct CorrectPlayerMovePredictionPacket<V: ProtoVersion> {
    pub prediction_type: V::PredictionType,
    pub position: (f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub on_ground: bool,
    pub tick: u64,
}

// TODO: redo this mess

impl<V: ProtoVersion> ProtoCodec for CorrectPlayerMovePredictionPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let mut prediction_type_stream: Vec<u8> = Vec::new();
        <V::PredictionType as ProtoCodec>::serialize(
            &self.prediction_type,
            &mut prediction_type_stream,
        )?;

        let mut prediction_type_cursor = Cursor::new(prediction_type_stream.as_slice());
        <u8 as ProtoCodec>::serialize(
            &<u8 as ProtoCodec>::deserialize(&mut prediction_type_cursor)?,
            stream,
        )?;

        <(f32, f32, f32) as ProtoCodecLE>::serialize(&self.position, stream)?;
        <(f32, f32, f32) as ProtoCodecLE>::serialize(&self.velocity, stream)?;

        copy(&mut prediction_type_cursor, stream)?;

        <bool as ProtoCodec>::serialize(&self.on_ground, stream)?;
        <u64 as ProtoCodecVAR>::serialize(&self.tick, stream)?;
        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let mut buffer: Vec<u8> = Vec::new();

        <u8 as ProtoCodec>::serialize(&<u8 as ProtoCodec>::deserialize(stream)?, &mut buffer)?;

        let position: (f32, f32, f32) = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;
        let velocity: (f32, f32, f32) = <(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?;

        stream.read_to_end(&mut buffer)?;
        let stream = &mut Cursor::new(buffer.as_slice());

        let prediction_type = <V::PredictionType as ProtoCodec>::deserialize(stream)?;

        let on_ground: bool = <bool as ProtoCodec>::deserialize(stream)?;
        let tick: u64 = <u64 as ProtoCodecVAR>::deserialize(stream)?;

        let val = Self {
            prediction_type,
            position,
            velocity,
            on_ground,
            tick,
        };

        Ok(val)
    }
    fn size_hint(&self) -> usize {
        1
    }
}

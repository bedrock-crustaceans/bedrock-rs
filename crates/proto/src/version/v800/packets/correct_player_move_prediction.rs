use std::io::{Cursor, Read};
use super::super::enums::PredictionType;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::{Vec2, Vec3};
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use bedrockrs_proto_core::error::ProtoCodecError;

#[gamepacket(id = 161)]
#[derive(Clone, Debug)]
pub struct CorrectPlayerMovePredictionPacket {
    pub prediction_type: PredictionType,
    pub position: Vec3<f32>,
    pub velocity: Vec3<f32>,
    pub on_ground: bool,
    pub tick: u64,
}

impl ProtoCodec for CorrectPlayerMovePredictionPacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        
        let mut prediction_type_stream: Vec<u8> = Vec::new();
        <PredictionType as ProtoCodec>::proto_serialize(&self.prediction_type, &mut prediction_type_stream)?;
        
        let mut prediction_type_cursor = Cursor::new(prediction_type_stream.as_slice());
        <u8 as ProtoCodec>::proto_serialize(&<u8 as ProtoCodec>::proto_deserialize(&mut prediction_type_cursor)?, stream)?;
        
        <Vec3<f32> as ProtoCodecLE>::proto_serialize(&self.position, stream)?;
        <Vec3<f32> as ProtoCodecLE>::proto_serialize(&self.velocity, stream)?;
        
        prediction_type_cursor.read_to_end(stream)?;
        
        <bool as ProtoCodec>::proto_serialize(&self.on_ground, stream)?;
        <u64 as ProtoCodecVAR>::proto_serialize(&self.tick, stream)?;
        Ok(())
    }
    
    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let mut buffer: Vec<u8> = Vec::new();
        
        <u8 as ProtoCodec>::proto_serialize(
            &<u8 as ProtoCodec>::proto_deserialize(stream)?, 
            &mut buffer
        )?;
        
        let position: Vec3<f32> = <Vec3<f32> as ProtoCodecLE>::proto_deserialize(stream)?;
        let velocity: Vec3<f32> = <Vec3<f32> as ProtoCodecLE>::proto_deserialize(stream)?;
        
        stream.read_to_end(&mut buffer)?;
        let stream= &mut Cursor::new(buffer.as_slice());

        let prediction_type: PredictionType = <PredictionType as ProtoCodec>::proto_deserialize(stream)?;
        
        let on_ground: bool = <bool as ProtoCodec>::proto_deserialize(stream)?;
        let tick: u64 = <u64 as ProtoCodecVAR>::proto_deserialize(stream)?;
        
        let val = Self { prediction_type, position, velocity, on_ground, tick };
        
        Ok(val)
    }
    fn get_size_prediction(&self) -> usize { 1 }
}
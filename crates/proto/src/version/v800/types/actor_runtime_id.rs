use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::Cursor;
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[derive(Clone, Debug)]
pub struct ActorRuntimeID(pub u64);

impl ProtoCodec for ActorRuntimeID {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        stream.write_u64_varint(self.0)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Ok(Self(stream.read_u64_varint()?))
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<u64>()
    }
}
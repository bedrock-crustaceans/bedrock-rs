use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct NetworkItemInstanceDescriptor {
    id: i32,
    stack_size: Option<u16>,
    aux_value: Option<u32>,
    block_runtime_id: Option<i32>,
    user_data_buffer: Option<String>,
}

impl ProtoCodec for NetworkItemInstanceDescriptor {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::proto_serialize(&self.id, stream)?;

        match &self.id {
            0 => {}
            _ => {
                ProtoCodecLE::proto_serialize(self.stack_size.as_ref().unwrap(), stream)?;
                ProtoCodecVAR::proto_serialize(self.aux_value.as_ref().unwrap(), stream)?;
                ProtoCodecVAR::proto_serialize(self.block_runtime_id.as_ref().unwrap(), stream)?;
                ProtoCodec::proto_serialize(self.user_data_buffer.as_ref().unwrap(), stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let id = ProtoCodecVAR::proto_deserialize(stream)?;

        let (stack_size, aux_value, block_runtime_id, user_data_buffer) = match &id {
            0 => (None, None, None, None),
            _ => {
                let stack_size = ProtoCodecLE::proto_deserialize(stream)?;
                let aux_value = ProtoCodecVAR::proto_deserialize(stream)?;
                let block_runtime_id = ProtoCodecVAR::proto_deserialize(stream)?;
                let user_data_buffer = ProtoCodec::proto_deserialize(stream)?;

                (
                    Some(stack_size),
                    Some(aux_value),
                    Some(block_runtime_id),
                    Some(user_data_buffer),
                )
            }
        };

        Ok(Self {
            id,
            stack_size,
            aux_value,
            block_runtime_id,
            user_data_buffer,
        })
    }

    fn get_size_prediction(&self) -> usize {
        ProtoCodecVAR::get_size_prediction(&self.id)
            + match self.id {
            0 => 0,
            _ => {
                ProtoCodecLE::get_size_prediction(self.stack_size.as_ref().unwrap())
                    + ProtoCodecVAR::get_size_prediction(self.aux_value.as_ref().unwrap())
                    + ProtoCodecVAR::get_size_prediction(self.block_runtime_id.as_ref().unwrap())
                    + ProtoCodec::get_size_prediction(self.user_data_buffer.as_ref().unwrap())
            }
        }
    }
}

// VERIFY: ProtoCodec impl

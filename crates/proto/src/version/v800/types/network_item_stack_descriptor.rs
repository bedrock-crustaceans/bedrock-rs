use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct NetworkItemStackDescriptor {
    id: i32,
    stack_size: Option<u16>,
    aux_value: Option<u32>,
    net_id_variant: Option<Option<i32>>,
    block_runtime_id: Option<i32>,
    user_data_buffer: Option<String>,
}

impl ProtoCodec for NetworkItemStackDescriptor {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::proto_serialize(&self.id, stream)?;

        match &self.id {
            0 => {}
            _ => {
                ProtoCodecLE::proto_serialize(self.stack_size.as_ref().unwrap(), stream)?;
                ProtoCodecVAR::proto_serialize(self.aux_value.as_ref().unwrap(), stream)?;
                <Option<i32> as ProtoCodecVAR>::proto_serialize(self.net_id_variant.as_ref().unwrap(), stream)?;
                ProtoCodecVAR::proto_serialize(self.block_runtime_id.as_ref().unwrap(), stream)?;
                String::proto_serialize(self.user_data_buffer.as_ref().unwrap(), stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let id = <i32 as ProtoCodecVAR>::proto_deserialize(stream)?;

        let (stack_size, aux_value, net_id_variant, block_runtime_id, user_data_buffer) = match id {
            0 => (None, None, None, None, None),
            _ => {
                let stack_size = <u16 as ProtoCodecLE>::proto_deserialize(stream)?;
                let aux_value = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
                let net_id_variant = <Option<i32> as ProtoCodecVAR>::proto_deserialize(stream)?;
                let block_runtime_id = <i32 as ProtoCodecVAR>::proto_deserialize(stream)?;
                let user_data_buffer = String::proto_deserialize(stream)?;

                (Some(stack_size), Some(aux_value), Some(net_id_variant), Some(block_runtime_id), Some(user_data_buffer))
            }
        };

        Ok(Self {
            id,
            stack_size,
            aux_value,
            net_id_variant,
            block_runtime_id,
            user_data_buffer,
        })
    }

    fn get_size_prediction(&self) -> usize {
        ProtoCodecVAR::get_size_prediction(&self.id) +
            match &self.id {
                0 => 0,
                _ => {
                    ProtoCodecLE::get_size_prediction(self.stack_size.as_ref().unwrap()) +
                        ProtoCodecVAR::get_size_prediction(self.aux_value.as_ref().unwrap()) +
                        <Option<i32> as ProtoCodecVAR>::get_size_prediction(self.net_id_variant.as_ref().unwrap()) +
                        ProtoCodecVAR::get_size_prediction(self.block_runtime_id.as_ref().unwrap()) +
                        String::get_size_prediction(self.user_data_buffer.as_ref().unwrap())
                }
            }
    }
}

// TODO: impl ProtoCodec
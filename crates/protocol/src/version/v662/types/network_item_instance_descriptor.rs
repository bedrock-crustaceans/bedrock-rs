use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct NetworkItemInstanceDescriptor {
    id: i32,
    stack_size: Option<u16>,
    aux_value: Option<u32>,
    block_runtime_id: Option<i32>,
    user_data_buffer: Option<Vec<u8>>,
}

impl ProtoCodec for NetworkItemInstanceDescriptor {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        ProtoCodecVAR::serialize(&self.id, stream)?;

        match &self.id {
            0 => {}
            _ => {
                ProtoCodecLE::serialize(self.stack_size.as_ref().ok_or(ProtoCodecError::ExpectedSome("stack_size"))?, stream)?;
                ProtoCodecVAR::serialize(self.aux_value.as_ref().ok_or(ProtoCodecError::ExpectedSome("aux_value"))?, stream)?;
                ProtoCodecVAR::serialize(self.block_runtime_id.as_ref().ok_or(ProtoCodecError::ExpectedSome("block_runtime_id"))?, stream)?;
                ProtoCodec::serialize(self.user_data_buffer.as_ref().ok_or(ProtoCodecError::ExpectedSome("user_data_buffer"))?, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let id = ProtoCodecVAR::deserialize(stream)?;

        let (stack_size, aux_value, block_runtime_id, user_data_buffer) = match &id {
            0 => (None, None, None, None),
            _ => {
                let stack_size = ProtoCodecLE::deserialize(stream)?;
                let aux_value = ProtoCodecVAR::deserialize(stream)?;
                let block_runtime_id = ProtoCodecVAR::deserialize(stream)?;
                let user_data_buffer = ProtoCodec::deserialize(stream)?;

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

    fn size_hint(&self) -> usize {
        ProtoCodecVAR::size_hint(&self.id)
            + match self.id {
                0 => 0,
                _ => {
                    self.stack_size.as_ref().map_or(0, ProtoCodecLE::size_hint)
                    + self.aux_value.as_ref().map_or(0, ProtoCodecVAR::size_hint)
                    + self.block_runtime_id.as_ref().map_or(0, ProtoCodecVAR::size_hint)
                    + self.user_data_buffer.as_ref().map_or(0, ProtoCodec::size_hint)
                }
            }
    }
}

// VERIFY: ProtoCodec impl

use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use std::io::Cursor;
use std::mem::size_of;

macro_rules! impl_proto_option {
    ($name:ident) => {
        impl<T: $name> $name for Option<T> {
            fn proto_serialize(&self, buf: &mut Vec<u8>) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                match self {
                    Some(v) => {
                        bool::proto_serialize(&true, buf)?;
                        T::proto_serialize(&v, buf)?;
                    }
                    None => bool::proto_serialize(&false, buf)?,
                }

                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(match bool::proto_deserialize(stream)? {
                    false => None,
                    true => Some(T::proto_deserialize(stream)?),
                })
            }

            fn get_size_prediction(&self) -> usize {
                match self {
                    Some(v) => T::get_size_prediction(v) + size_of::<u8>(),
                    None => size_of::<u8>(),
                }
            }
        }
    };
}

impl_proto_option!(ProtoCodec);
impl_proto_option!(ProtoCodecLE);
impl_proto_option!(ProtoCodecBE);
impl_proto_option!(ProtoCodecVAR);

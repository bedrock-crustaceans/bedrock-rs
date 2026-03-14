use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::size_of;

macro_rules! impl_proto_option {
    ($name:ident) => {
        impl<T: $name> $name for Option<T> {
            fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                match self {
                    Some(v) => {
                        bool::serialize(&true, stream)?;
                        T::serialize(&v, stream)?;
                    }
                    None => bool::serialize(&false, stream)?,
                }

                Ok(())
            }

            fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(match bool::deserialize(stream)? {
                    false => None,
                    true => Some(T::deserialize(stream)?),
                })
            }

            fn size_hint(&self) -> usize {
                match self {
                    Some(v) => T::size_hint(v) + size_of::<u8>(),
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

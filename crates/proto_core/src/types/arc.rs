use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use crate::ProtoCodec;
use std::io::{Read, Write};
use varint_rs::{VarintReader, VarintWriter};

macro_rules! impl_proto_arc_slice {
    ($name:ident) => {
        impl<T: $name> $name for ::std::sync::Arc<[T]> {
            fn serialize<W: ::std::io::Write>(
                &self,
                stream: &mut W,
            ) -> Result<(), ProtoCodecError> {
                <u32 as ProtoCodecVAR>::serialize(&(self.len() as u32), stream)?;
                for i in self {
                    T::serialize(i, stream)?;
                }
                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    vec.push(T::deserialize(stream)?);
                }
                Ok(vec.into())
            }

            fn size_hint(&self) -> usize {
                <u32 as ProtoCodecVAR>::size_hint(&(self.len() as u32))
                    + self.iter().map(|i| T::size_hint(i)).sum::<usize>()
            }
        }
    };
}

impl_proto_arc_slice!(ProtoCodec);
impl_proto_arc_slice!(ProtoCodecLE);
impl_proto_arc_slice!(ProtoCodecBE);
impl_proto_arc_slice!(ProtoCodecVAR);

impl ProtoCodec for ::std::sync::Arc<str> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        stream.write_u32_varint(self.len().into())?;
        stream.write_all(self.as_bytes())?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let len = stream.read_u32_varint()? as usize;
        
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        
        Ok(String::from_utf8(buf)?.into())
    }

    fn size_hint(&self) -> usize {
        <u32 as ProtoCodecVAR>::size_hint(&(self.len() as u32))
        + self.len() * size_of::<u8>()
    }
}

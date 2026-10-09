use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;

macro_rules! impl_proto_vec {
    ($name:ident) => {
        impl<T: $name> $name for Vec<T> {
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
                let mut vec = Vec::new();
                for _ in 0..len {
                    let _span = crate::trace::field("item", Some(vec.len()));
                    vec.push(T::deserialize(stream)?);
                }
                Ok(vec)
            }

            fn size_hint(&self) -> usize {
                <u32 as ProtoCodecVAR>::size_hint(&(self.len() as u32))
                    + self.iter().map(|i| T::size_hint(i)).sum::<usize>()
            }
        }
    };
}

impl_proto_vec!(ProtoCodec);
impl_proto_vec!(ProtoCodecLE);
impl_proto_vec!(ProtoCodecBE);
impl_proto_vec!(ProtoCodecVAR);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn vec_with_huge_length_prefix_and_no_items_is_an_eof_error() {
        let mut stream = Cursor::new([0xff, 0xff, 0xff, 0xff, 0x0f]);
        let result = <Vec<[u64; 64]> as ProtoCodecLE>::deserialize(&mut stream);
        assert!(
            matches!(result, Err(ProtoCodecError::IOError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof),
            "expected EOF, got {result:?}"
        );
    }
}

use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use seq_macro::seq;

macro_rules! impl_proto_tuple {
    ($name:ident, 0) => {
        impl $name for () {
            fn serialize<W: ::std::io::Write>(&self, _stream: &mut W) -> Result<(), ProtoCodecError> {
                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(_stream: &mut R) -> Result<Self, ProtoCodecError> {
                Ok(())
            }

            fn size_hint(&self) -> usize {
                0
            }
        }
    };
    ($name:ident, $size:literal) => {
        impl<T: $name> $name for seq!(N in 0..$size { ( #(T, )* ) }) {
            fn serialize<W: ::std::io::Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                seq!(N in 0..$size {
                    self.N.serialize(stream)?;
                });

                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                seq!(N in 0..$size {
                    let tuple = (
                        #( T::deserialize(stream)?, )*
                    );
                });

                Ok(tuple)
            }

            fn size_hint(&self) -> usize {
                let mut size: usize = 0;
                seq!(N in 0..$size {
                    size += self.N.size_hint();
                });
                size
            }
        }
    };
}

impl_proto_tuple!(ProtoCodec, 0);
impl_proto_tuple!(ProtoCodec, 1);
impl_proto_tuple!(ProtoCodec, 2);
impl_proto_tuple!(ProtoCodec, 3);
impl_proto_tuple!(ProtoCodec, 4);
impl_proto_tuple!(ProtoCodec, 5);
impl_proto_tuple!(ProtoCodec, 6);
impl_proto_tuple!(ProtoCodec, 7);
impl_proto_tuple!(ProtoCodec, 8);
impl_proto_tuple!(ProtoCodec, 9);
impl_proto_tuple!(ProtoCodec, 10);
impl_proto_tuple!(ProtoCodec, 11);
impl_proto_tuple!(ProtoCodec, 12);

impl_proto_tuple!(ProtoCodecLE, 0);
impl_proto_tuple!(ProtoCodecLE, 1);
impl_proto_tuple!(ProtoCodecLE, 2);
impl_proto_tuple!(ProtoCodecLE, 3);
impl_proto_tuple!(ProtoCodecLE, 4);
impl_proto_tuple!(ProtoCodecLE, 5);
impl_proto_tuple!(ProtoCodecLE, 6);
impl_proto_tuple!(ProtoCodecLE, 7);
impl_proto_tuple!(ProtoCodecLE, 8);
impl_proto_tuple!(ProtoCodecLE, 9);
impl_proto_tuple!(ProtoCodecLE, 10);
impl_proto_tuple!(ProtoCodecLE, 11);
impl_proto_tuple!(ProtoCodecLE, 12);

impl_proto_tuple!(ProtoCodecBE, 0);
impl_proto_tuple!(ProtoCodecBE, 1);
impl_proto_tuple!(ProtoCodecBE, 2);
impl_proto_tuple!(ProtoCodecBE, 3);
impl_proto_tuple!(ProtoCodecBE, 4);
impl_proto_tuple!(ProtoCodecBE, 5);
impl_proto_tuple!(ProtoCodecBE, 6);
impl_proto_tuple!(ProtoCodecBE, 7);
impl_proto_tuple!(ProtoCodecBE, 8);
impl_proto_tuple!(ProtoCodecBE, 9);
impl_proto_tuple!(ProtoCodecBE, 10);
impl_proto_tuple!(ProtoCodecBE, 11);
impl_proto_tuple!(ProtoCodecBE, 12);

impl_proto_tuple!(ProtoCodecVAR, 0);
impl_proto_tuple!(ProtoCodecVAR, 1);
impl_proto_tuple!(ProtoCodecVAR, 2);
impl_proto_tuple!(ProtoCodecVAR, 3);
impl_proto_tuple!(ProtoCodecVAR, 4);
impl_proto_tuple!(ProtoCodecVAR, 5);
impl_proto_tuple!(ProtoCodecVAR, 6);
impl_proto_tuple!(ProtoCodecVAR, 7);
impl_proto_tuple!(ProtoCodecVAR, 8);
impl_proto_tuple!(ProtoCodecVAR, 9);
impl_proto_tuple!(ProtoCodecVAR, 10);
impl_proto_tuple!(ProtoCodecVAR, 11);
impl_proto_tuple!(ProtoCodecVAR, 12);

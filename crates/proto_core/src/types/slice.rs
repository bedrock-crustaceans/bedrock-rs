use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use seq_macro::seq;
use std::io::{Read, Write};

macro_rules! impl_proto_slice {
    ($name:ident, 0) => {
        impl<T> $name for [T; 0] {
            fn serialize<W: Write>(&self, _stream: &mut W) -> Result<(), ProtoCodecError> {
                Ok(())
            }

            fn deserialize<R: Read>(_stream: &mut R) -> Result<Self, ProtoCodecError> {
                Ok([])
            }

            fn size_hint(&self) -> usize {
                0
            }
        }
    };
    ($name:ident, $size:literal) => {
        impl<T: $name> $name for [T; $size] {
            fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                seq!(N in 0..$size {
                    self[N].serialize(stream)?;
                });

                Ok(())
            }

            fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                seq!(N in 0..$size {
                    let buf = [
                        #( T::deserialize(stream)?, )*
                    ];
                });

                Ok(buf)
            }

            fn size_hint(&self) -> usize {
                self.iter().fold(0, |acc, x| acc + x.size_hint())
            }
        }
    };
}

impl_proto_slice!(ProtoCodec, 0);
impl_proto_slice!(ProtoCodec, 1);
impl_proto_slice!(ProtoCodec, 2);
impl_proto_slice!(ProtoCodec, 3);
impl_proto_slice!(ProtoCodec, 4);
impl_proto_slice!(ProtoCodec, 5);
impl_proto_slice!(ProtoCodec, 6);
impl_proto_slice!(ProtoCodec, 7);
impl_proto_slice!(ProtoCodec, 8);
impl_proto_slice!(ProtoCodec, 9);
impl_proto_slice!(ProtoCodec, 10);
impl_proto_slice!(ProtoCodec, 11);
impl_proto_slice!(ProtoCodec, 12);

impl_proto_slice!(ProtoCodecLE, 0);
impl_proto_slice!(ProtoCodecLE, 1);
impl_proto_slice!(ProtoCodecLE, 2);
impl_proto_slice!(ProtoCodecLE, 3);
impl_proto_slice!(ProtoCodecLE, 4);
impl_proto_slice!(ProtoCodecLE, 5);
impl_proto_slice!(ProtoCodecLE, 6);
impl_proto_slice!(ProtoCodecLE, 7);
impl_proto_slice!(ProtoCodecLE, 8);
impl_proto_slice!(ProtoCodecLE, 9);
impl_proto_slice!(ProtoCodecLE, 10);
impl_proto_slice!(ProtoCodecLE, 11);
impl_proto_slice!(ProtoCodecLE, 12);

impl_proto_slice!(ProtoCodecBE, 0);
impl_proto_slice!(ProtoCodecBE, 1);
impl_proto_slice!(ProtoCodecBE, 2);
impl_proto_slice!(ProtoCodecBE, 3);
impl_proto_slice!(ProtoCodecBE, 4);
impl_proto_slice!(ProtoCodecBE, 5);
impl_proto_slice!(ProtoCodecBE, 6);
impl_proto_slice!(ProtoCodecBE, 7);
impl_proto_slice!(ProtoCodecBE, 8);
impl_proto_slice!(ProtoCodecBE, 9);
impl_proto_slice!(ProtoCodecBE, 10);
impl_proto_slice!(ProtoCodecBE, 11);
impl_proto_slice!(ProtoCodecBE, 12);

impl_proto_slice!(ProtoCodecVAR, 0);
impl_proto_slice!(ProtoCodecVAR, 1);
impl_proto_slice!(ProtoCodecVAR, 2);
impl_proto_slice!(ProtoCodecVAR, 3);
impl_proto_slice!(ProtoCodecVAR, 4);
impl_proto_slice!(ProtoCodecVAR, 5);
impl_proto_slice!(ProtoCodecVAR, 6);
impl_proto_slice!(ProtoCodecVAR, 7);
impl_proto_slice!(ProtoCodecVAR, 8);
impl_proto_slice!(ProtoCodecVAR, 9);
impl_proto_slice!(ProtoCodecVAR, 10);
impl_proto_slice!(ProtoCodecVAR, 11);
impl_proto_slice!(ProtoCodecVAR, 12);

use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use seq_macro::seq;
use std::io::Cursor;

macro_rules! impl_proto_slice {
    ($name:ident, 0) => {
        impl<T> $name for [T; 0] {
            fn proto_serialize(&self, _stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
                Ok(())
            }

            fn proto_deserialize(_stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
                Ok([])
            }

            fn get_size_prediction(&self) -> usize {
                0
            }
        }
    };
    ($name:ident, $size:literal) => {
        impl<T: $name> $name for [T; $size] {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
                seq!(N in 0..$size {
                    self[N].proto_serialize(stream)?;
                });

                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
                seq!(N in 0..$size {
                    let buf = [
                        #( T::proto_deserialize(stream)?, )*
                    ];
                });

                Ok(buf)
            }

            fn get_size_prediction(&self) -> usize {
                self[0].get_size_prediction() * $size
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

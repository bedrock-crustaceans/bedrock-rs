use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use seq_macro::seq;
use std::io::Cursor;

macro_rules! impl_proto_tuple {
    ($name:ident, 0) => {
        impl $name for () {
            fn proto_serialize(&self, _stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
                Ok(())
            }

            fn proto_deserialize(_stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
                Ok(())
            }

            fn get_size_prediction(&self) -> usize {
                0
            }
        }
    };
    ($name:ident, $size:literal) => {
        impl<T: $name> $name for seq!(N in 0..$size { ( #(T, )* ) }) {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
                seq!(N in 0..$size {
                    self.N.proto_serialize(stream)?;
                });

                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
                seq!(N in 0..$size {
                    let tuple = (
                        #( T::proto_deserialize(stream)?, )*
                    );
                });

                Ok(tuple)
            }

            fn get_size_prediction(&self) -> usize {
                self.0.get_size_prediction() * $size
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

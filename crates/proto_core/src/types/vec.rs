use std::io::Cursor;

use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use vek::{Vec2, Vec3};

macro_rules! impl_proto_vec {
    ($name:ident) => {
        impl<T: $name> $name for Vec<T> {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
                <u32 as ProtoCodecVAR>::proto_serialize(&(self.len() as u32), stream)?;
                for i in self {
                    T::proto_serialize(i, stream)?;
                }
                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
                let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    vec.push(T::proto_deserialize(stream)?);
                }
                Ok(vec)
            }

            fn get_size_prediction(&self) -> usize {
                <u32 as ProtoCodecVAR>::get_size_prediction(&(self.len() as u32))
                    + self
                        .iter()
                        .map(|i| T::get_size_prediction(i))
                        .sum::<usize>()
            }
        }
    };
}

macro_rules! impl_proto_vec2 {
    ($name:ident) => {
        impl<T: $name> $name for Vec2<T> {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                T::proto_serialize(&self.x, stream)?;
                T::proto_serialize(&self.y, stream)?;

                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(Self {
                    x: T::proto_deserialize(stream)?,
                    y: T::proto_deserialize(stream)?,
                })
            }

            fn get_size_prediction(&self) -> usize {
                self.x.get_size_prediction() * 2
            }
        }
    };
}

macro_rules! impl_proto_vec3 {
    ($name:ident) => {
        impl<T: $name> $name for Vec3<T> {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                T::proto_serialize(&self.x, stream)?;
                T::proto_serialize(&self.y, stream)?;
                T::proto_serialize(&self.z, stream)?;

                Ok(())
            }

            fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(Self {
                    x: T::proto_deserialize(stream)?,
                    y: T::proto_deserialize(stream)?,
                    z: T::proto_deserialize(stream)?,
                })
            }

            fn get_size_prediction(&self) -> usize {
                self.x.get_size_prediction() * 3
            }
        }
    };
}

impl_proto_vec!(ProtoCodec);
impl_proto_vec!(ProtoCodecLE);
impl_proto_vec!(ProtoCodecBE);
impl_proto_vec!(ProtoCodecVAR);

impl_proto_vec2!(ProtoCodec);
impl_proto_vec2!(ProtoCodecLE);
impl_proto_vec2!(ProtoCodecBE);
impl_proto_vec2!(ProtoCodecVAR);

impl_proto_vec3!(ProtoCodec);
impl_proto_vec3!(ProtoCodecLE);
impl_proto_vec3!(ProtoCodecBE);
impl_proto_vec3!(ProtoCodecVAR);

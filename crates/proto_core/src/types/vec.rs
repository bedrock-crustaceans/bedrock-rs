use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use crate::ProtoCodec;
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
            fn serialize<W: ::std::io::Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                T::serialize(&self.x, stream)?;
                T::serialize(&self.y, stream)?;

                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(Self {
                    x: T::deserialize(stream)?,
                    y: T::deserialize(stream)?,
                })
            }

            fn size_hint(&self) -> usize {
                self.x.size_hint() * 2
            }
        }
    };
}

macro_rules! impl_proto_vec3 {
    ($name:ident) => {
        impl<T: $name> $name for Vec3<T> {
            fn serialize<W: ::std::io::Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError>
            where
                Self: Sized,
            {
                T::serialize(&self.x, stream)?;
                T::serialize(&self.y, stream)?;
                T::serialize(&self.z, stream)?;

                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ProtoCodecError>
            where
                Self: Sized,
            {
                Ok(Self {
                    x: T::deserialize(stream)?,
                    y: T::deserialize(stream)?,
                    z: T::deserialize(stream)?,
                })
            }

            fn size_hint(&self) -> usize {
                self.x.size_hint() * 3
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

use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use crate::ProtoCodec;
use vek::{Vec2, Vec3};

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

impl_proto_vec2!(ProtoCodec);
impl_proto_vec2!(ProtoCodecLE);
impl_proto_vec2!(ProtoCodecBE);
impl_proto_vec2!(ProtoCodecVAR);

impl_proto_vec3!(ProtoCodec);
impl_proto_vec3!(ProtoCodecLE);
impl_proto_vec3!(ProtoCodecBE);
impl_proto_vec3!(ProtoCodecVAR);

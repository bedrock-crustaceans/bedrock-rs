use crate::ProtoVersion;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub enum DataItemType<V: ProtoVersion> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    NBT(nbtx::Value),
    Pos(V::BlockPos),
    Int64(i64),
    Vec3((f32, f32, f32)),
}

impl<V: ProtoVersion> DataItemType<V> {
    fn type_id(&self) -> u32 {
        match self {
            Self::Byte(_) => 0,
            Self::Short(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::NBT(_) => 5,
            Self::Pos(_) => 6,
            Self::Int64(_) => 7,
            Self::Vec3(_) => 8,
        }
    }
}

impl<V: ProtoVersion> ProtoCodec for DataItemType<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let type_id = self.type_id();
        <u32 as ProtoCodecVAR>::serialize(&type_id, stream)?;
        <u8 as ProtoCodec>::serialize(&(type_id as u8), stream)?;
        match self {
            Self::Byte(v) => <i8 as ProtoCodec>::serialize(v, stream)?,
            Self::Short(v) => <i16 as ProtoCodecLE>::serialize(v, stream)?,
            Self::Int(v) => <i32 as ProtoCodecVAR>::serialize(v, stream)?,
            Self::Float(v) => <f32 as ProtoCodecLE>::serialize(v, stream)?,
            Self::String(v) => <String as ProtoCodec>::serialize(v, stream)?,
            Self::NBT(v) => nbtx::to_bytes_in::<nbtx::NetworkLittleEndian>(stream, v)?,
            Self::Pos(v) => <V::BlockPos as ProtoCodec>::serialize(v, stream)?,
            Self::Int64(v) => <i64 as ProtoCodecVAR>::serialize(v, stream)?,
            Self::Vec3(v) => <(f32, f32, f32) as ProtoCodecLE>::serialize(v, stream)?,
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let type_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        <u8 as ProtoCodec>::deserialize(stream)?;

        Ok(match type_id {
            0 => Self::Byte(<i8 as ProtoCodec>::deserialize(stream)?),
            1 => Self::Short(<i16 as ProtoCodecLE>::deserialize(stream)?),
            2 => Self::Int(<i32 as ProtoCodecVAR>::deserialize(stream)?),
            3 => Self::Float(<f32 as ProtoCodecLE>::deserialize(stream)?),
            4 => Self::String(<String as ProtoCodec>::deserialize(stream)?),
            5 => Self::NBT(nbtx::from_bytes::<nbtx::NetworkLittleEndian, _>(stream)?),
            6 => Self::Pos(<V::BlockPos as ProtoCodec>::deserialize(stream)?),
            7 => Self::Int64(<i64 as ProtoCodecVAR>::deserialize(stream)?),
            8 => Self::Vec3(<(f32, f32, f32) as ProtoCodecLE>::deserialize(stream)?),
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    format!("{other}"),
                    "DataItemType",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        let type_id_size = ProtoCodecVAR::size_hint(&self.type_id()) + 1;

        type_id_size
            + match self {
                Self::Byte(v) => ProtoCodec::size_hint(v),
                Self::Short(v) => ProtoCodecLE::size_hint(v),
                Self::Int(v) => ProtoCodecVAR::size_hint(v),
                Self::Float(v) => ProtoCodecLE::size_hint(v),
                Self::String(v) => ProtoCodec::size_hint(v),
                Self::NBT(_) => 1,
                Self::Pos(v) => ProtoCodec::size_hint(v),
                Self::Int64(v) => ProtoCodecVAR::size_hint(v),
                Self::Vec3(v) => ProtoCodecLE::size_hint(v),
            }
    }
}

use crate::ProtoCodec;
use crate::endian::{ProtoCodecBE, ProtoCodecLE, ProtoCodecVAR};
use crate::error::ProtoCodecError;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;
use std::io::{Read, Write};
use std::mem::size_of;

impl ProtoCodec for u8 {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        Ok(stream.write_u8(*self)?)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(stream.read_u8()?)
    }

    fn size_hint(&self) -> usize {
        size_of::<u8>()
    }
}

impl ProtoCodec for i8 {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        Ok(stream.write_i8(*self)?)
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        Ok(stream.read_i8()?)
    }

    fn size_hint(&self) -> usize {
        size_of::<i8>()
    }
}

macro_rules! impl_proto_codec_le {
    ($int:ident) => {
        paste! {
            impl ProtoCodecLE for $int {
                fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                    Ok(WriteBytesExt::[<write_ $int>]::<LittleEndian>(stream, *self)?)
                }

                fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                    Ok(ReadBytesExt::[<read_ $int>]::<LittleEndian>(stream)?)
                }

                fn size_hint(&self) -> usize {
                    size_of::<$int>()
                }
            }
        }
    };
}

macro_rules! impl_proto_codec_be {
    ($int:ident) => {
        paste! {
            impl ProtoCodecBE for $int {
                fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                    Ok(WriteBytesExt::[<write_ $int>]::<BigEndian>(stream, *self)?)
                }

                fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                    Ok(ReadBytesExt::[<read_ $int>]::<BigEndian>(stream)?)
                }

                fn size_hint(&self) -> usize {
                    size_of::<$int>()
                }
            }
        }
    };
}

macro_rules! impl_proto_codec_var {
    ($unsigned:ident, $signed:ident) => {
        impl ProtoCodecVAR for $unsigned {
            fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                let mut value = *self;
                while value >= 0x80 {
                    stream.write_u8(value as u8 | 0x80)?;
                    value >>= 7;
                }
                Ok(stream.write_u8(value as u8)?)
            }

            fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                const MAX_BYTES: usize = ($unsigned::BITS as usize).div_ceil(7);
                let mut decoded: $unsigned = 0;
                for i in 0..MAX_BYTES {
                    let byte = stream.read_u8()?;
                    decoded |= ((byte & 0x7f) as $unsigned) << (7 * i);
                    if byte & 0x80 == 0 {
                        return Ok(decoded);
                    }
                }
                Err(ProtoCodecError::VarintTooLong(MAX_BYTES))
            }

            fn size_hint(&self) -> usize {
                size_of::<$unsigned>()
            }
        }

        impl ProtoCodecVAR for $signed {
            fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
                let zigzag = ((*self << 1) ^ (*self >> ($signed::BITS - 1))) as $unsigned;
                <$unsigned as ProtoCodecVAR>::serialize(&zigzag, stream)
            }

            fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
                let zigzag = <$unsigned as ProtoCodecVAR>::deserialize(stream)?;
                Ok((zigzag >> 1) as $signed ^ -((zigzag & 1) as $signed))
            }

            fn size_hint(&self) -> usize {
                size_of::<$signed>()
            }
        }
    };
}

impl_proto_codec_le!(u16);
impl_proto_codec_le!(i16);
impl_proto_codec_le!(u32);
impl_proto_codec_le!(i32);
impl_proto_codec_le!(u64);
impl_proto_codec_le!(i64);
impl_proto_codec_le!(u128);
impl_proto_codec_le!(i128);
impl_proto_codec_le!(f32);
impl_proto_codec_le!(f64);

impl_proto_codec_be!(u16);
impl_proto_codec_be!(i16);
impl_proto_codec_be!(u32);
impl_proto_codec_be!(i32);
impl_proto_codec_be!(u64);
impl_proto_codec_be!(i64);
impl_proto_codec_be!(u128);
impl_proto_codec_be!(i128);
impl_proto_codec_be!(f32);
impl_proto_codec_be!(f64);

impl_proto_codec_var!(u16, i16);
impl_proto_codec_var!(u32, i32);
impl_proto_codec_var!(u64, i64);
impl_proto_codec_var!(u128, i128);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn decode_var<T: ProtoCodecVAR>(bytes: &[u8]) -> Result<T, ProtoCodecError> {
        T::deserialize(&mut Cursor::new(bytes))
    }

    #[test]
    fn u32_varint_longer_than_five_bytes_is_rejected() {
        let result = decode_var::<u32>(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]);
        assert!(
            matches!(result, Err(ProtoCodecError::VarintTooLong(5))),
            "got {result:?}"
        );
    }

    #[test]
    fn u64_varint_longer_than_ten_bytes_is_rejected() {
        let result = decode_var::<u64>(&[0xff; 11]);
        assert!(
            matches!(result, Err(ProtoCodecError::VarintTooLong(10))),
            "got {result:?}"
        );
    }

    #[test]
    fn varints_round_trip_at_the_edges() {
        for value in [0u32, 1, 127, 128, 16_383, 16_384, u32::MAX] {
            let mut encoded = Vec::new();
            <u32 as ProtoCodecVAR>::serialize(&value, &mut encoded).unwrap();
            assert_eq!(decode_var::<u32>(&encoded).unwrap(), value);
        }
        for value in [0i32, -1, 1, i32::MIN, i32::MAX] {
            let mut encoded = Vec::new();
            <i32 as ProtoCodecVAR>::serialize(&value, &mut encoded).unwrap();
            assert_eq!(decode_var::<i32>(&encoded).unwrap(), value);
        }
        for value in [0i64, -1, i64::MIN, i64::MAX] {
            let mut encoded = Vec::new();
            <i64 as ProtoCodecVAR>::serialize(&value, &mut encoded).unwrap();
            assert_eq!(decode_var::<i64>(&encoded).unwrap(), value);
        }
        for value in [0u128, u128::MAX] {
            let mut encoded = Vec::new();
            <u128 as ProtoCodecVAR>::serialize(&value, &mut encoded).unwrap();
            assert_eq!(decode_var::<u128>(&encoded).unwrap(), value);
        }
    }
}

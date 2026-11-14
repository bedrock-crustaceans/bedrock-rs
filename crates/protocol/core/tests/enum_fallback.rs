use bedrock_macros::ProtoCodec;
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::Cursor;

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
enum Platform {
    Google = 1,
    Win32 = 8,
    #[enum_fallback]
    Unknown(i32),
}

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
enum Sound {
    Hit = 0,
    #[enum_fallback]
    Unknown(u32),
}

#[derive(ProtoCodec, Clone, Debug, PartialEq)]
#[enum_repr(u8)]
#[repr(u8)]
enum Strict {
    A = 0,
}

fn encode<T: ProtoCodec>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    value.serialize(&mut bytes).unwrap();
    bytes
}

#[test]
fn unlisted_discriminant_decodes_into_the_fallback() {
    let bytes = 99i32.to_le_bytes();
    let value = Platform::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
    assert_eq!(value, Platform::Unknown(99));
}

#[test]
fn fallback_round_trips_the_original_bytes() {
    let bytes = 99i32.to_le_bytes();
    let value = Platform::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
    assert_eq!(encode(&value), bytes);
}

#[test]
fn fallback_size_hint_matches_the_encoded_length() {
    let bytes = 99i32.to_le_bytes();
    let value = Platform::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
    assert_eq!(value.size_hint(), bytes.len());
}

#[test]
fn listed_discriminant_still_decodes_to_its_variant() {
    let bytes = 8i32.to_le_bytes();
    let value = Platform::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
    assert_eq!(value, Platform::Win32);
}

#[test]
fn varint_fallback_round_trips() {
    let bytes = [0xac, 0x02];
    let value = Sound::deserialize(&mut Cursor::new(&bytes[..])).unwrap();
    assert_eq!(value, Sound::Unknown(300));
    assert_eq!(encode(&value), bytes);
}

#[test]
fn enum_without_fallback_still_rejects_unknown_ids() {
    let result = Strict::deserialize(&mut Cursor::new(&[5u8][..]));
    assert!(matches!(result, Err(ProtoCodecError::InvalidEnumID(..))));
}

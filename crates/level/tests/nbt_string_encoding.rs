//! Pins the behavior of the NBT string codec for the on-disk (little-endian)
//! format used by block-palette entries, level settings, and actor records.
//!
//! Bedrock's on-disk NBT stores TAG_String payloads in a modified UTF-8 form:
//!   - an embedded NUL is written as the overlong two-byte sequence 0xC0 0x80
//!   - a supplementary-plane scalar (e.g. an emoji) is written as a surrogate
//!     pair of two three-byte sequences rather than one four-byte sequence
//!
//! These tests feed hand-built payloads through the same deserialization entry
//! point the crate uses for palettes and settings, so any future change in how
//! the codec treats those byte classes is caught here.

use std::collections::HashMap;
use std::io::Cursor;

/// Builds a little-endian NBT root compound `{ "s": TAG_String(payload) }`.
///
/// Layout: TAG_Compound(0x0A), u16 root-name length (0), then one field:
/// TAG_String(0x08), u16 name length, name bytes, u16 payload length,
/// payload bytes, terminated by TAG_End(0x00).
fn root_with_string(payload: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.push(0x0A);
    b.extend_from_slice(&0u16.to_le_bytes());
    b.push(0x08);
    b.extend_from_slice(&1u16.to_le_bytes());
    b.push(b's');
    b.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    b.extend_from_slice(payload);
    b.push(0x00);
    b
}

fn decode_string_field(payload: &[u8]) -> Result<String, nbtx::Error> {
    let bytes = root_with_string(payload);
    let mut cur = Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cur)?;
    match value {
        nbtx::Value::Compound(mut m) => match m.remove("s") {
            Some(nbtx::Value::String(s)) => Ok(s),
            other => panic!("expected string field, got {other:?}"),
        },
        other => panic!("expected compound, got {other:?}"),
    }
}

/// Plain ASCII decodes unchanged. This is the overwhelmingly common case for
/// palette names and record keys.
#[test]
fn ascii_round_trips() {
    assert_eq!(decode_string_field(b"minecraft:stone").unwrap(), "minecraft:stone");
}

/// A NUL encoded as the modified-UTF-8 overlong sequence 0xC0 0x80 is currently
/// rejected by the codec. When modified-UTF-8 support lands this must instead
/// decode to a single U+0000.
#[test]
fn embedded_nul_overlong_currently_rejected() {
    let err = decode_string_field(&[b'a', 0xC0, 0x80, b'b']).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("utf-8"),
        "expected a utf-8 decode failure, got: {msg}"
    );
}

/// A supplementary-plane scalar written as a surrogate pair (here U+1F600,
/// surrogates D83D/DE00 -> ED A0 BD ED B8 80) is currently rejected. When
/// modified-UTF-8 support lands this must decode to the single scalar U+1F600.
#[test]
fn surrogate_pair_currently_rejected() {
    let err = decode_string_field(&[0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("utf-8"),
        "expected a utf-8 decode failure, got: {msg}"
    );
}

/// Encoding currently emits standard UTF-8: a scalar NUL as a single 0x00 and a
/// supplementary scalar as one four-byte sequence. A modified-UTF-8 writer would
/// instead emit 0xC0 0x80 and a six-byte surrogate pair respectively. This pins
/// the current output so a change in the write path is noticed.
#[test]
fn encoding_currently_emits_standard_utf8() {
    let mut m = HashMap::new();
    m.insert("s".to_string(), nbtx::Value::String("a\u{0}\u{1F600}".to_string()));
    let out = nbtx::to_le_bytes(&nbtx::Value::Compound(m)).unwrap();
    // Trailing byte is TAG_End; the six bytes before it are the string payload
    // 61 00 F0 9F 98 80 (scalar NUL as one 0x00, emoji as one four-byte scalar).
    let payload = &out[out.len() - 7..out.len() - 1];
    assert_eq!(payload, &[0x61, 0x00, 0xF0, 0x9F, 0x98, 0x80]);
}

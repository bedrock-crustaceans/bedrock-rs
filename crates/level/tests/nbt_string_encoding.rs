//! Pins the behavior of the NBT string codec for the on-disk (little-endian)
//! format used by block-palette entries, level settings, and actor records.
//!
//! Bedrock's on-disk NBT stores TAG_String payloads in a modified UTF-8 form:
//!   - an embedded NUL is written as the overlong two-byte sequence 0xC0 0x80
//!   - a supplementary-plane scalar (e.g. an emoji) is written as a surrogate
//!     pair of two three-byte sequences rather than one four-byte sequence
//!
//! The contract these tests pin: **those byte sequences round-trip verbatim, and
//! are not interpreted.** `nbtx::Value::String` holds a `bstr::BString` — raw
//! bytes — so a payload goes in and comes back out byte for byte whether or not
//! it is valid standard UTF-8. That is lossless, which is what this crate wants
//! (see `TODO.md`, "Lossless by default"), but it is *not* the same as decoding
//! modified UTF-8: the NUL case yields the two bytes `C0 80`, not one `00`, and
//! the surrogate-pair case yields six bytes, not one U+1F600 scalar. Nothing
//! anywhere converts the escaping into Unicode scalars yet.
//!
//! Note the raw-bytes guarantee is specific to `Value` (and to a `BString`
//! field). A `#[derive(Facet)]` struct field typed `String` still validates
//! UTF-8 and errors on these payloads, exactly as it did before.
//!
//! These tests feed hand-built payloads through the same deserialization entry
//! point the crate uses for palettes and settings, so any future change in how
//! the codec treats those byte classes is caught here.
//!
//! The payloads here are synthetic. `nbt_actor_storage_key.rs` pins the same
//! contract against a real one: the field
//! `internalComponents.EntityStorageKeyComponent.StorageKey` in real
//! `actorprefix` records is exactly this kind of string tag, and its 8-byte
//! payload is frequently not valid UTF-8 in practice.

use std::io::Cursor;

use bstr::{BStr, BString};

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

fn decode_string_field(payload: &[u8]) -> Result<BString, nbtx::Error> {
    let bytes = root_with_string(payload);
    let mut cur = Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cur)?;
    match value {
        nbtx::Value::Compound(mut m) => match m.shift_remove(BStr::new("s")) {
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
    assert_eq!(
        decode_string_field(b"minecraft:stone").unwrap(),
        "minecraft:stone"
    );
}

/// A NUL encoded as the modified-UTF-8 overlong sequence 0xC0 0x80 survives as
/// those two bytes. It is *not* folded into a single U+0000 — nothing decodes
/// the escaping — but it is no longer rejected either.
#[test]
fn embedded_nul_overlong_round_trips_as_raw_bytes() {
    let payload: &[u8] = &[b'a', 0xC0, 0x80, b'b'];
    assert_eq!(decode_string_field(payload).unwrap().as_slice(), payload);
}

/// A supplementary-plane scalar written as a surrogate pair (here U+1F600,
/// surrogates D83D/DE00 -> ED A0 BD ED B8 80) survives as those six bytes,
/// rather than being rejected as invalid UTF-8 or folded into one scalar.
#[test]
fn surrogate_pair_round_trips_as_raw_bytes() {
    let payload: &[u8] = &[0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80];
    assert_eq!(decode_string_field(payload).unwrap().as_slice(), payload);
}

/// The write side is a passthrough too: whatever bytes a `Value::String` holds
/// are the bytes that land on the wire. Encoding the raw modified-UTF-8 forms
/// reproduces them exactly, so a decode/encode cycle is byte-identical.
#[test]
fn encoding_emits_string_bytes_verbatim() {
    let payload: &[u8] = &[b'a', 0xC0, 0x80, 0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80];

    let mut m = nbtx::Compound::new();
    m.insert("s".into(), nbtx::Value::String(BString::from(payload)));
    let out = nbtx::to_le_bytes(&nbtx::Value::Compound(m)).unwrap();

    // Trailing byte is TAG_End; the payload sits immediately before it.
    let written = &out[out.len() - 1 - payload.len()..out.len() - 1];
    assert_eq!(written, payload);

    // And the whole document round-trips back to the same bytes.
    assert_eq!(decode_string_field(payload).unwrap().as_slice(), payload);
}

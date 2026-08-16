//! `0x77` [`crate::key::KeyVariant::AabbVolumes`]: axis-aligned bounding-box
//! volume records for a chunk.
//!
//! Observed only in one world in the corpus (39 records, all naming
//! `minecraft:trial_chambers`), so what follows is everything the grammar
//! commits to and no more -- the *semantics* of the entry and trailer
//! fields are not established, only their sizes and count-derived lengths,
//! which hold with zero exceptions across all 39 real records:
//!
//! - **Header**: three `i32`s. Every real record has `(1, 1, 0)`, but
//!   nothing else in the corpus explains what they mean, so they are
//!   preserved verbatim rather than asserted.
//! - **Structure id**: a `u16` little-endian length prefix followed by that
//!   many bytes. Every real record's bytes happen to be the ASCII string
//!   `minecraft:trial_chambers`, but this is not asserted to be UTF-8 --
//!   only one string has ever been observed here, which is too little to
//!   commit to an encoding beyond "length-prefixed bytes".
//! - **Entry count**: one `i32`, `N`.
//! - **Entries**: `N` fixed 28-byte records, each a leading `i32` id
//!   followed by 24 opaque bytes (six more `i32`s in every real record).
//!   The trailing six integers are consistent in magnitude with two
//!   3-coordinate corners -- a plausible bounding-box min/max -- but that
//!   pairing is not asserted; only the entry's fixed 28-byte size is. The
//!   id does not follow one fixed order: most records count down from `N`
//!   to 1, but 3 of the 39 real records count up from 1 to `N` instead,
//!   with no other distinguishing property found, so nothing about id
//!   ordering is asserted either.
//! - **Trailer**: whatever bytes remain, always exactly `4 * (3*N + 2)`
//!   bytes long. That length holds without exception across all 39 real
//!   records (including `N = 0`, where it would be 8 bytes), but the
//!   trailer's internal shape is not modeled -- it is carried as an opaque
//!   blob, validated only by this length.
//!
//! A record whose bytes do not fit this grammar (too short for the header
//! or the string, a string or entry count than overruns the value, or a
//! trailer whose length does not match the formula) is malformed per
//! decision 11, not merely unmodelled, and decoding it is an error.

use crate::error::{Error, Result};

/// One fixed 28-byte entry within an [`AabbVolumes`] record: a leading
/// `i32` id (surfaced -- see the module docs for why nothing about its
/// ordering is asserted) followed by six opaque `i32`s carried through
/// verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AabbVolumesEntry {
    pub id: i32,
    pub fields: [i32; 6],
}

const ENTRY_LEN: usize = 4 + 6 * 4;

impl AabbVolumesEntry {
    fn decode(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), ENTRY_LEN);
        let mut ints = bytes
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().expect("chunk is exactly 4 bytes")));
        let id = ints.next().expect("28 bytes is at least one i32");
        let mut fields = [0i32; 6];
        for (slot, value) in fields.iter_mut().zip(ints) {
            *slot = value;
        }
        Self { id, fields }
    }

    fn encode_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.to_le_bytes());
        for field in self.fields {
            out.extend_from_slice(&field.to_le_bytes());
        }
    }
}

/// A decoded `0x77` AabbVolumes value. See the module docs for what each
/// field does and does not commit to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AabbVolumes {
    /// The three leading header integers, preserved verbatim -- every real
    /// record has `(1, 1, 0)`, but that is not enforced on decode.
    pub header: (i32, i32, i32),
    /// The length-prefixed structure id's raw bytes (every real record
    /// spells `minecraft:trial_chambers` here, but this is carried as bytes
    /// rather than `String` since only one value has ever been observed).
    pub structure_id: Vec<u8>,
    /// The `N` fixed-size entries following the count field.
    pub entries: Vec<AabbVolumesEntry>,
    /// Whatever bytes remain after the entries, carried through verbatim.
    /// Always exactly `4 * (3*entries.len() + 2)` bytes on a real record --
    /// enforced on decode, but the bytes themselves are not interpreted.
    pub trailer: Vec<u8>,
}

/// The trailer length in bytes a well-formed record's entry count implies.
fn expected_trailer_len(entry_count: usize) -> Option<usize> {
    // `entry_count` comes straight from an on-disk `i32` field with no
    // upper bound of its own; guard the arithmetic rather than let a
    // corrupt or hostile count byte overflow it.
    3usize
        .checked_mul(entry_count)?
        .checked_add(2)?
        .checked_mul(4)
}

/// Decodes a `0x77` AabbVolumes value. See the module docs for the grammar.
pub fn decode_aabb_volumes(bytes: &[u8]) -> Result<AabbVolumes> {
    const HEADER_LEN: usize = 12;
    const STRING_LEN_LEN: usize = 2;
    const COUNT_LEN: usize = 4;

    if bytes.len() < HEADER_LEN {
        return Err(Error::Invalid(
            "aabbVolumes value is shorter than its header",
        ));
    }
    let header = (
        i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
        i32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        i32::from_le_bytes(bytes[8..12].try_into().unwrap()),
    );

    let after_header = &bytes[HEADER_LEN..];
    if after_header.len() < STRING_LEN_LEN {
        return Err(Error::Invalid(
            "aabbVolumes value is shorter than its structure-id length prefix",
        ));
    }
    let string_len = u16::from_le_bytes(after_header[0..2].try_into().unwrap()) as usize;
    let after_string_len = &after_header[STRING_LEN_LEN..];
    if after_string_len.len() < string_len {
        return Err(Error::Invalid(
            "aabbVolumes structure-id length prefix overruns the value",
        ));
    }
    let structure_id = after_string_len[..string_len].to_vec();

    let after_string = &after_string_len[string_len..];
    if after_string.len() < COUNT_LEN {
        return Err(Error::Invalid(
            "aabbVolumes value is shorter than its entry count",
        ));
    }
    let count = i32::from_le_bytes(after_string[0..4].try_into().unwrap());
    let count = usize::try_from(count)
        .map_err(|_| Error::Invalid("aabbVolumes entry count is negative"))?;

    let after_count = &after_string[COUNT_LEN..];
    let entries_len = count
        .checked_mul(ENTRY_LEN)
        .ok_or(Error::Invalid("aabbVolumes entry count overflows"))?;
    if after_count.len() < entries_len {
        return Err(Error::Invalid("aabbVolumes entry count overruns the value"));
    }
    let entries = after_count[..entries_len]
        .chunks_exact(ENTRY_LEN)
        .map(AabbVolumesEntry::decode)
        .collect();

    let trailer = &after_count[entries_len..];
    let expected =
        expected_trailer_len(count).ok_or(Error::Invalid("aabbVolumes entry count overflows"))?;
    if trailer.len() != expected {
        return Err(Error::Invalid(
            "aabbVolumes trailer length does not match the entry count",
        ));
    }

    Ok(AabbVolumes {
        header,
        structure_id,
        entries,
        trailer: trailer.to_vec(),
    })
}

/// Encodes a `0x77` AabbVolumes value: the inverse of
/// [`decode_aabb_volumes`]. Every field this crate does not interpret
/// (`structure_id`, the entries' `fields`, `trailer`) is written back
/// verbatim, so a decoded real record re-encodes byte-identically.
pub fn encode_aabb_volumes(value: &AabbVolumes) -> Result<Vec<u8>> {
    let string_len = u16::try_from(value.structure_id.len())
        .map_err(|_| Error::Invalid("aabbVolumes structure id is too long to encode"))?;
    let count = i32::try_from(value.entries.len())
        .map_err(|_| Error::Invalid("aabbVolumes has too many entries to encode"))?;

    let mut out = Vec::with_capacity(
        12 + 2
            + value.structure_id.len()
            + 4
            + value.entries.len() * ENTRY_LEN
            + value.trailer.len(),
    );
    out.extend_from_slice(&value.header.0.to_le_bytes());
    out.extend_from_slice(&value.header.1.to_le_bytes());
    out.extend_from_slice(&value.header.2.to_le_bytes());
    out.extend_from_slice(&string_len.to_le_bytes());
    out.extend_from_slice(&value.structure_id);
    out.extend_from_slice(&count.to_le_bytes());
    for entry in &value.entries {
        entry.encode_into(&mut out);
    }
    out.extend_from_slice(&value.trailer);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bytes() -> Vec<u8> {
        // Shaped like a real one-entry record: header (1,1,0), structure id
        // "x" (not asserted to be a real structure name), one entry, and a
        // trailer of the formula-mandated 4*(3*1+2) = 20 bytes.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1i32.to_le_bytes());
        bytes.extend_from_slice(&1i32.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(b"x");
        bytes.extend_from_slice(&1i32.to_le_bytes()); // count = 1
        bytes.extend_from_slice(&7i32.to_le_bytes()); // entry id
        for field in [1i32, 2, 3, 4, 5, 6] {
            bytes.extend_from_slice(&field.to_le_bytes());
        }
        bytes.extend_from_slice(&[0xaa; 20]); // trailer, 4*(3*1+2) bytes
        bytes
    }

    #[test]
    fn decodes_a_synthetic_well_formed_record() {
        let bytes = sample_bytes();
        let value = decode_aabb_volumes(&bytes).unwrap();
        assert_eq!(value.header, (1, 1, 0));
        assert_eq!(value.structure_id, b"x");
        assert_eq!(value.entries.len(), 1);
        assert_eq!(value.entries[0].id, 7);
        assert_eq!(value.entries[0].fields, [1, 2, 3, 4, 5, 6]);
        assert_eq!(value.trailer, vec![0xaa; 20]);
    }

    #[test]
    fn round_trips_a_synthetic_well_formed_record() {
        let bytes = sample_bytes();
        let value = decode_aabb_volumes(&bytes).unwrap();
        let re_encoded = encode_aabb_volumes(&value).unwrap();
        assert_eq!(re_encoded, bytes);
    }

    #[test]
    fn decodes_a_zero_entry_record() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes()); // empty structure id
        bytes.extend_from_slice(&0i32.to_le_bytes()); // count = 0
        bytes.extend_from_slice(&[0u8; 8]); // trailer, 4*(3*0+2) = 8 bytes
        let value = decode_aabb_volumes(&bytes).unwrap();
        assert!(value.entries.is_empty());
        assert_eq!(value.trailer.len(), 8);
        assert_eq!(encode_aabb_volumes(&value).unwrap(), bytes);
    }

    #[test]
    fn does_not_assert_the_header_is_one_one_zero() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&9i32.to_le_bytes());
        bytes.extend_from_slice(&8i32.to_le_bytes());
        bytes.extend_from_slice(&7i32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 8]);
        let value = decode_aabb_volumes(&bytes).unwrap();
        assert_eq!(value.header, (9, 8, 7));
    }

    #[test]
    fn rejects_a_value_shorter_than_the_header() {
        assert!(decode_aabb_volumes(&[0u8; 11]).is_err());
    }

    #[test]
    fn rejects_a_string_length_prefix_that_overruns_the_value() {
        let mut bytes = vec![0u8; 12];
        bytes.extend_from_slice(&100u16.to_le_bytes());
        bytes.extend_from_slice(b"short");
        assert!(decode_aabb_volumes(&bytes).is_err());
    }

    #[test]
    fn rejects_an_entry_count_that_overruns_the_value() {
        let mut bytes = vec![0u8; 12];
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&5i32.to_le_bytes()); // claims 5 entries
        // No entry bytes follow at all.
        assert!(decode_aabb_volumes(&bytes).is_err());
    }

    #[test]
    fn rejects_a_negative_entry_count() {
        let mut bytes = vec![0u8; 12];
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&(-1i32).to_le_bytes());
        assert!(decode_aabb_volumes(&bytes).is_err());
    }

    #[test]
    fn rejects_a_trailer_length_that_does_not_match_the_formula() {
        let mut bytes = vec![0u8; 12];
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes()); // count = 0, expects 8-byte trailer
        bytes.extend_from_slice(&[0u8; 7]); // one byte short
        assert!(decode_aabb_volumes(&bytes).is_err());
    }
}

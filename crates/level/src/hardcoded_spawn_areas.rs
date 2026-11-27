//! `0x39` [`crate::key::KeyVariant::HardcodedSpawnAreas`]: mob-structure
//! spawn volumes for a chunk (the game hardcodes a spawn-eligible region
//! around fortresses, swamp huts, ocean monuments, and pillager outposts
//! rather than relying on ordinary light-level spawning there).
//!
//! The value is a count-prefixed list: an `i32` count followed by that many
//! fixed 25-byte records -- six little-endian `i32`s followed by one tag
//! byte. Every real record in the corpus (32 across two fixtures, one to
//! seven entries per key) fits this exactly, with the count always equal to
//! the number of 25-byte records the rest of the value actually holds.
//!
//! The six integers are consistent with an axis-aligned bounding box (an
//! observed real entry: `(-176, 64, 176, -170, 70, 184)` -- a small, cubical
//! volume, each pair's first value no greater than its second), but nothing
//! in the corpus ties them to specific axis roles beyond that, so they are
//! surfaced as a plain array in on-disk order rather than named `min`/`max`
//! fields. The tag byte's enumeration is likewise not established: the
//! corpus shows exactly four distinct values (1, 2, 3, 5 -- notably no 0 and
//! no 4), and neither reference decodes this key at all, so nothing here
//! asserts what a value means. It is carried as a raw `u8`.

use crate::error::{Error, Result};

/// Byte length of one record: six `i32`s (24 bytes) plus one tag byte.
const RECORD_LEN: usize = 6 * 4 + 1;

/// One `0x39` record. See the module docs for what is and is not
/// established about its fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardcodedSpawnArea {
    /// The six on-disk integers, in on-disk order. Consistent in magnitude
    /// with an axis-aligned bounding box, but not asserted as one -- see the
    /// module docs.
    pub bounds: [i32; 6],
    /// The trailing tag byte. Every real value observed is one of `1`, `2`,
    /// `3`, or `5`; nothing establishes what any of them mean.
    pub tag: u8,
}

impl HardcodedSpawnArea {
    fn decode(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), RECORD_LEN);
        let mut bounds = [0i32; 6];
        for (i, slot) in bounds.iter_mut().enumerate() {
            *slot = i32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        }
        Self {
            bounds,
            tag: bytes[24],
        }
    }

    fn encode_into(&self, out: &mut Vec<u8>) {
        for value in self.bounds {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.push(self.tag);
    }
}

/// Decodes a `0x39` `HardcodedSpawnAreas` value. See the module docs for the
/// grammar.
///
/// Per decision 11, a count that disagrees with the number of bytes actually
/// present -- including a negative count, or a value too short to hold even
/// the count field -- is malformed rather than merely unmodelled, and is an
/// error rather than a truncated or padded result.
///
/// # Errors
///
/// Returns [`Error::Invalid`] if `bytes` is shorter than the 4-byte count, if
/// the count is negative or overflows, or if the bytes after the count are
/// not exactly `count` records long.
pub fn decode_hardcoded_spawn_areas(bytes: &[u8]) -> Result<Vec<HardcodedSpawnArea>> {
    let Some((count, rest)) = bytes.split_first_chunk::<4>() else {
        return Err(Error::Invalid(
            "hardcodedSpawnAreas value is shorter than its count",
        ));
    };
    let count = i32::from_le_bytes(*count);
    let count = usize::try_from(count)
        .map_err(|_| Error::Invalid("hardcodedSpawnAreas count is negative"))?;

    let expected_len = count
        .checked_mul(RECORD_LEN)
        .ok_or(Error::Invalid("hardcodedSpawnAreas count overflows"))?;
    if rest.len() != expected_len {
        return Err(Error::Invalid(
            "hardcodedSpawnAreas count does not match the value length",
        ));
    }

    Ok(rest
        .as_chunks::<RECORD_LEN>()
        .0
        .iter()
        .map(|area| HardcodedSpawnArea::decode(area))
        .collect())
}

/// Encodes a `0x39` `HardcodedSpawnAreas` value: the inverse of
/// [`decode_hardcoded_spawn_areas`]. Every field is written back verbatim,
/// so a decoded real record re-encodes byte-identically.
///
/// # Errors
///
/// Returns [`Error::Invalid`] if there are more areas than an `i32` count can
/// express.
pub fn encode_hardcoded_spawn_areas(areas: &[HardcodedSpawnArea]) -> Result<Vec<u8>> {
    let count = i32::try_from(areas.len())
        .map_err(|_| Error::Invalid("too many hardcodedSpawnAreas entries to encode"))?;

    let mut out = Vec::with_capacity(4 + areas.len() * RECORD_LEN);
    out.extend_from_slice(&count.to_le_bytes());
    for area in areas {
        area.encode_into(&mut out);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From the corpus: `v1_17_20/caves_and_cliffs`, the sole `0x39` record
    /// in that fixture -- one entry, tag `2`.
    #[test]
    fn decodes_a_real_single_entry_record() {
        let bytes = [
            0x01, 0x00, 0x00, 0x00, 0x50, 0xff, 0xff, 0xff, 0x40, 0x00, 0x00, 0x00, 0xb0, 0x00,
            0x00, 0x00, 0x56, 0xff, 0xff, 0xff, 0x46, 0x00, 0x00, 0x00, 0xb8, 0x00, 0x00, 0x00,
            0x02,
        ];
        let areas = decode_hardcoded_spawn_areas(&bytes).unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].bounds, [-176, 64, 176, -170, 70, 184]);
        assert_eq!(areas[0].tag, 2);
    }

    /// From the corpus: `v1_18`, chunk key `f7ffffffffffffff0100000039`
    /// (dimension 1, i.e. the Nether) -- seven entries, all tag `1`.
    #[test]
    fn decodes_a_real_seven_entry_record() {
        let bytes = [
            0x07, 0x00, 0x00, 0x00, 0x70, 0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0xf8, 0xff,
            0xff, 0xff, 0x70, 0xff, 0xff, 0xff, 0x3f, 0x00, 0x00, 0x00, 0xfc, 0xff, 0xff, 0xff,
            0x01, 0x71, 0xff, 0xff, 0xff, 0x36, 0x00, 0x00, 0x00, 0xf4, 0xff, 0xff, 0xff, 0x7d,
            0xff, 0xff, 0xff, 0x43, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x01, 0x7e, 0xff,
            0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0xf8, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff,
            0x3f, 0x00, 0x00, 0x00, 0xfc, 0xff, 0xff, 0xff, 0x01, 0x7e, 0xff, 0xff, 0xff, 0x41,
            0x00, 0x00, 0x00, 0xf8, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0x47, 0x00, 0x00,
            0x00, 0xfc, 0xff, 0xff, 0xff, 0x01, 0x7e, 0xff, 0xff, 0xff, 0x41, 0x00, 0x00, 0x00,
            0xf3, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0x47, 0x00, 0x00, 0x00, 0xf7, 0xff,
            0xff, 0xff, 0x01, 0x7e, 0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0xfd, 0xff, 0xff,
            0xff, 0x7f, 0xff, 0xff, 0xff, 0x3f, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x01,
            0x7e, 0xff, 0xff, 0xff, 0x3e, 0x00, 0x00, 0x00, 0xf0, 0xff, 0xff, 0xff, 0x7f, 0xff,
            0xff, 0xff, 0x47, 0x00, 0x00, 0x00, 0xf2, 0xff, 0xff, 0xff, 0x01,
        ];
        assert_eq!(bytes.len(), 179);
        let areas = decode_hardcoded_spawn_areas(&bytes).unwrap();
        assert_eq!(areas.len(), 7);
        assert!(areas.iter().all(|a| a.tag == 1));
        assert_eq!(areas[0].bounds, [-144, 57, -8, -144, 63, -4]);
        assert_eq!(areas[6].bounds, [-130, 62, -16, -129, 71, -14]);
    }

    /// From the corpus: `v1_18`, chunk key `fbffffff030000000100000039` --
    /// two entries, both tag `1`.
    #[test]
    fn decodes_a_real_two_entry_record() {
        let bytes = [
            0x02, 0x00, 0x00, 0x00, 0xb0, 0xff, 0xff, 0xff, 0x30, 0x00, 0x00, 0x00, 0x3f, 0x00,
            0x00, 0x00, 0xbd, 0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00,
            0x01, 0xbe, 0xff, 0xff, 0xff, 0x30, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0xbf,
            0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x01,
        ];
        assert_eq!(bytes.len(), 54);
        let areas = decode_hardcoded_spawn_areas(&bytes).unwrap();
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].bounds, [-80, 48, 63, -67, 57, 63]);
        assert_eq!(areas[1].bounds, [-66, 48, 63, -65, 57, 63]);
    }

    #[test]
    fn decodes_a_zero_entry_record() {
        let bytes = 0i32.to_le_bytes();
        let areas = decode_hardcoded_spawn_areas(&bytes).unwrap();
        assert!(areas.is_empty());
    }

    #[test]
    fn round_trips_every_real_record_shape() {
        for bytes in [
            [
                0x01, 0x00, 0x00, 0x00, 0x50, 0xff, 0xff, 0xff, 0x40, 0x00, 0x00, 0x00, 0xb0, 0x00,
                0x00, 0x00, 0x56, 0xff, 0xff, 0xff, 0x46, 0x00, 0x00, 0x00, 0xb8, 0x00, 0x00, 0x00,
                0x02,
            ]
            .to_vec(),
            vec![
                0x02, 0x00, 0x00, 0x00, 0xb0, 0xff, 0xff, 0xff, 0x30, 0x00, 0x00, 0x00, 0x3f, 0x00,
                0x00, 0x00, 0xbd, 0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00,
                0x01, 0xbe, 0xff, 0xff, 0xff, 0x30, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0xbf,
                0xff, 0xff, 0xff, 0x39, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x01,
            ],
        ] {
            let areas = decode_hardcoded_spawn_areas(&bytes).unwrap();
            assert_eq!(encode_hardcoded_spawn_areas(&areas).unwrap(), bytes);
        }
    }

    #[test]
    fn decode_rejects_an_empty_value() {
        assert!(decode_hardcoded_spawn_areas(&[]).is_err());
    }

    #[test]
    fn decode_rejects_a_negative_count() {
        let bytes = (-1i32).to_le_bytes();
        assert!(decode_hardcoded_spawn_areas(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_a_count_that_overruns_the_value() {
        let mut bytes = 5i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; RECORD_LEN]); // only one record's worth, claims five
        assert!(decode_hardcoded_spawn_areas(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_trailing_bytes_past_the_last_full_record() {
        let mut bytes = 1i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; RECORD_LEN]);
        bytes.push(0xaa); // one stray byte past the one record the count promises
        assert!(decode_hardcoded_spawn_areas(&bytes).is_err());
    }
}

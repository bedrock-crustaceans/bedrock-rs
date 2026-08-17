//! `0x3b` [`crate::key::KeyVariant::Checksums`]: per-record consistency
//! hashes for a chunk's own other stored records.
//!
//! The value is a count-prefixed list: an `i32` count followed by that many
//! fixed 11-byte records -- a little-endian `i16` tag, a `u8` subtag, and an
//! 8-byte hash. Every real record in the corpus (700 across three fixtures:
//! `v1_16`, `v1_17_20`, and `v1_17_20/caves_and_cliffs`) fits this exactly,
//! with the count always equal to the number of 11-byte records the rest of
//! the value actually holds. No fixture past `v1_17_20` carries this key at
//! all -- the era that writes it is narrow.
//!
//! The tag is not an independent enumeration: every one of the 700 real
//! records' tag values equals the on-disk tag byte of one of this same
//! chunk's *other* record kinds -- `0x2f` [`crate::key::KeyVariant::SubChunk`],
//! `0x2d` [`crate::key::KeyVariant::HeightMap`],
//! `0x2b` [`crate::key::KeyVariant::Biome3d`],
//! `0x31` [`crate::key::KeyVariant::BlockEntity`], or
//! `0x32` [`crate::key::KeyVariant::Entity`] -- exactly the set a chunk of
//! that era actually has records for. Within a run of same-tag records the
//! subtag counts up from 0 and resets to 0 when the tag changes -- see the
//! tests for real examples -- consistent with "the Nth record of this
//! kind" (a subchunk's records appear in ascending index order), but that
//! pairing is not asserted by the decoder, only the tag/subtag split
//! itself.
//!
//! The trailing 8 bytes are a hash of something in the named record, with
//! an unknown seed; this module does not attempt to determine either and
//! carries them as an opaque `[u8; 8]`. Decoding only recovers the record
//! boundaries and the tag/subtag pair, which is what makes a decoded value
//! re-encode byte-identically without needing to reproduce the hash.

use crate::error::{Error, Result};

/// Byte length of one record: a 2-byte tag, a 1-byte subtag, and an 8-byte
/// hash.
const RECORD_LEN: usize = 2 + 1 + 8;

/// One `0x3b` record. See the module docs for what is and is not
/// established about its fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChecksumRecord {
    /// The record kind this checksum covers, equal to that kind's on-disk
    /// tag byte in every real record observed (see the module docs).
    pub tag: i16,
    /// An ordinal within same-tag records, observed to count up from 0 and
    /// reset at each tag change.
    pub subtag: u8,
    /// The hash itself, opaque -- see the module docs for why this crate
    /// does not attempt to interpret it.
    pub hash: [u8; 8],
}

impl ChecksumRecord {
    fn decode(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), RECORD_LEN);
        let tag = i16::from_le_bytes([bytes[0], bytes[1]]);
        let subtag = bytes[2];
        let mut hash = [0u8; 8];
        hash.copy_from_slice(&bytes[3..11]);
        Self { tag, subtag, hash }
    }

    fn encode_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.tag.to_le_bytes());
        out.push(self.subtag);
        out.extend_from_slice(&self.hash);
    }
}

/// Decodes a `0x3b` Checksums value. See the module docs for the grammar.
///
/// Per decision 11, a count that disagrees with the number of bytes actually
/// present -- including a negative count, or a value too short to hold even
/// the count field -- is malformed rather than merely unmodelled, and is an
/// error rather than a truncated or padded result.
///
/// Per architecture decision 13, this key's write policy stays preserve --
/// this decoder exists to expose the record structure the corpus
/// establishes, not to change how a write path handles the key, since what
/// is hashed and with what seed remain unknown and this crate does not
/// regenerate the hash bytes.
pub fn decode_checksums(bytes: &[u8]) -> Result<Vec<ChecksumRecord>> {
    if bytes.len() < 4 {
        return Err(Error::Invalid("checksums value is shorter than its count"));
    }
    let count = i32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let count =
        usize::try_from(count).map_err(|_| Error::Invalid("checksums count is negative"))?;

    let rest = &bytes[4..];
    let expected_len = count
        .checked_mul(RECORD_LEN)
        .ok_or(Error::Invalid("checksums count overflows"))?;
    if rest.len() != expected_len {
        return Err(Error::Invalid(
            "checksums count does not match the value length",
        ));
    }

    Ok(rest
        .chunks_exact(RECORD_LEN)
        .map(ChecksumRecord::decode)
        .collect())
}

/// Encodes a `0x3b` Checksums value: the inverse of [`decode_checksums`].
/// Every field -- including the opaque hash -- is written back verbatim, so
/// a decoded real record re-encodes byte-identically.
pub fn encode_checksums(records: &[ChecksumRecord]) -> Result<Vec<u8>> {
    let count = i32::try_from(records.len())
        .map_err(|_| Error::Invalid("too many checksum records to encode"))?;

    let mut out = Vec::with_capacity(4 + records.len() * RECORD_LEN);
    out.extend_from_slice(&count.to_le_bytes());
    for record in records {
        record.encode_into(&mut out);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// From the corpus: `v1_16`, chunk key `00000000000000003b` -- six
    /// records: `SubChunk` 0-2, `HeightMap` 0, then `SubChunk` 3-4.
    #[test]
    fn decodes_a_real_record_with_subchunk_and_heightmap_tags() {
        let bytes = [
            0x06, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x2b, 0xbe, 0x41, 0x93, 0xf1, 0x63, 0xd6,
            0xdd, 0x2f, 0x00, 0x01, 0x18, 0x21, 0x0b, 0x3c, 0xe2, 0xa8, 0xfa, 0x19, 0x2f, 0x00,
            0x02, 0x39, 0xb6, 0x52, 0x13, 0xdf, 0xe3, 0x6c, 0x9a, 0x2d, 0x00, 0x00, 0x5f, 0x3a,
            0xaa, 0xdb, 0x2a, 0x43, 0xcd, 0x81, 0x2f, 0x00, 0x03, 0xd3, 0x90, 0x45, 0x9a, 0xb3,
            0x4d, 0x6c, 0xe6, 0x2f, 0x00, 0x04, 0x28, 0xdb, 0xa5, 0x3d, 0x36, 0xd8, 0x5a, 0x07,
        ];
        assert_eq!(bytes.len(), 70);
        let records = decode_checksums(&bytes).unwrap();
        assert_eq!(records.len(), 6);
        let tags_and_subtags: Vec<(i16, u8)> = records.iter().map(|r| (r.tag, r.subtag)).collect();
        assert_eq!(
            tags_and_subtags,
            vec![
                (0x2f, 0),
                (0x2f, 1),
                (0x2f, 2),
                (0x2d, 0),
                (0x2f, 3),
                (0x2f, 4),
            ]
        );
        assert_eq!(
            records[0].hash,
            [0x2b, 0xbe, 0x41, 0x93, 0xf1, 0x63, 0xd6, 0xdd]
        );
    }

    /// From the corpus: `v1_16`, chunk key `00000000020000003b` -- the same
    /// shape as above but ending in an `Entity` (`0x32`) record instead of a
    /// second `SubChunk`.
    #[test]
    fn decodes_a_real_record_with_an_entity_tag() {
        let bytes = [
            0x06, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0xb9, 0xb0, 0xd1, 0x90, 0x56, 0xba, 0x5b,
            0x7e, 0x2f, 0x00, 0x01, 0x9a, 0x97, 0x6c, 0xdb, 0xac, 0x5c, 0xa5, 0x7b, 0x2f, 0x00,
            0x02, 0x9c, 0xf3, 0xc3, 0xa5, 0xb7, 0x4f, 0x23, 0x81, 0x2d, 0x00, 0x00, 0x4e, 0xf9,
            0xb6, 0xf2, 0x71, 0x7e, 0x34, 0x32, 0x2f, 0x00, 0x03, 0x72, 0x9f, 0x0e, 0x01, 0x1a,
            0x52, 0xab, 0xf2, 0x32, 0x00, 0x00, 0x8c, 0x51, 0x55, 0x49, 0x72, 0xeb, 0xab, 0x9e,
        ];
        assert_eq!(bytes.len(), 70);
        let records = decode_checksums(&bytes).unwrap();
        assert_eq!(records.last().map(|r| (r.tag, r.subtag)), Some((0x32, 0)));
    }

    /// From the corpus: `v1_16`, chunk key `00000000e9ffffff3b` -- ends in a
    /// `BlockEntity` (`0x31`) record.
    #[test]
    fn decodes_a_real_record_with_a_block_entity_tag() {
        let bytes = [
            0x07, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0xed, 0x5c, 0x9f, 0x65, 0x72, 0xe5, 0x5a,
            0xc1, 0x2f, 0x00, 0x01, 0x00, 0x78, 0xbe, 0x58, 0xbc, 0x24, 0x69, 0xac, 0x2f, 0x00,
            0x02, 0x64, 0x8c, 0x90, 0x91, 0x23, 0x1e, 0x5c, 0x51, 0x2d, 0x00, 0x00, 0x4f, 0x51,
            0x4a, 0xee, 0x44, 0x70, 0x2d, 0xe4, 0x2f, 0x00, 0x03, 0xb3, 0xc9, 0xf6, 0x83, 0xbd,
            0x0e, 0x49, 0xec, 0x2f, 0x00, 0x04, 0xde, 0x3d, 0x48, 0x80, 0xdd, 0x68, 0x36, 0xbf,
            0x31, 0x00, 0x00, 0x76, 0xc5, 0xeb, 0x10, 0xf6, 0x10, 0xac, 0x81,
        ];
        assert_eq!(bytes.len(), 81);
        let records = decode_checksums(&bytes).unwrap();
        assert_eq!(records.len(), 7);
        assert_eq!(records.last().map(|r| (r.tag, r.subtag)), Some((0x31, 0)));
    }

    /// From the corpus: `v1_17_20/caves_and_cliffs`, chunk key
    /// `00000000000000003b` -- a 3D-biome chunk, so the singleton non-`SubChunk`
    /// entry is `Biome3d` (`0x2b`) rather than `HeightMap`, and it appears
    /// second (after the first `SubChunk`) rather than after all of them.
    #[test]
    fn decodes_a_real_record_with_a_biome3d_tag() {
        let bytes = [
            0x0a, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0xe1, 0xd7, 0x70, 0xc5, 0x40, 0xa5, 0xc6,
            0x88, 0x2b, 0x00, 0x00, 0x95, 0xd7, 0x7f, 0x8e, 0xae, 0xa7, 0xee, 0x12, 0x2f, 0x00,
            0x01, 0xe8, 0x43, 0xe4, 0x0b, 0xdc, 0xc9, 0x35, 0xf9, 0x2f, 0x00, 0x02, 0x33, 0xf0,
            0x73, 0xc6, 0xa9, 0x57, 0x27, 0xe9, 0x2f, 0x00, 0x03, 0x4a, 0x72, 0xde, 0x30, 0x07,
            0x2f, 0x60, 0xad, 0x2f, 0x00, 0x04, 0xca, 0x82, 0xec, 0x00, 0xca, 0xca, 0x04, 0x45,
            0x2f, 0x00, 0x05, 0x3e, 0xfb, 0x5a, 0x2c, 0x63, 0x16, 0x19, 0x39, 0x2f, 0x00, 0x06,
            0x08, 0xca, 0x41, 0x75, 0x2b, 0x9e, 0xb1, 0x96, 0x2f, 0x00, 0x07, 0x65, 0x66, 0x0d,
            0x6a, 0xca, 0xd8, 0x8f, 0xc7, 0x2f, 0x00, 0x08, 0x7a, 0xd4, 0xfe, 0xbc, 0x7c, 0x4e,
            0x29, 0x15,
        ];
        assert_eq!(bytes.len(), 114);
        let records = decode_checksums(&bytes).unwrap();
        assert_eq!(records.len(), 10);
        let tags_and_subtags: Vec<(i16, u8)> = records.iter().map(|r| (r.tag, r.subtag)).collect();
        assert_eq!(
            tags_and_subtags,
            vec![
                (0x2f, 0),
                (0x2b, 0),
                (0x2f, 1),
                (0x2f, 2),
                (0x2f, 3),
                (0x2f, 4),
                (0x2f, 5),
                (0x2f, 6),
                (0x2f, 7),
                (0x2f, 8),
            ]
        );
    }

    #[test]
    fn decodes_a_zero_entry_record() {
        let bytes = 0i32.to_le_bytes();
        assert!(decode_checksums(&bytes).unwrap().is_empty());
    }

    #[test]
    fn round_trips_every_real_record_shape() {
        let real_records: [&[u8]; 4] = [
            &[
                0x06, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x2b, 0xbe, 0x41, 0x93, 0xf1, 0x63, 0xd6,
                0xdd, 0x2f, 0x00, 0x01, 0x18, 0x21, 0x0b, 0x3c, 0xe2, 0xa8, 0xfa, 0x19, 0x2f, 0x00,
                0x02, 0x39, 0xb6, 0x52, 0x13, 0xdf, 0xe3, 0x6c, 0x9a, 0x2d, 0x00, 0x00, 0x5f, 0x3a,
                0xaa, 0xdb, 0x2a, 0x43, 0xcd, 0x81, 0x2f, 0x00, 0x03, 0xd3, 0x90, 0x45, 0x9a, 0xb3,
                0x4d, 0x6c, 0xe6, 0x2f, 0x00, 0x04, 0x28, 0xdb, 0xa5, 0x3d, 0x36, 0xd8, 0x5a, 0x07,
            ],
            &[
                0x0a, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0xe1, 0xd7, 0x70, 0xc5, 0x40, 0xa5, 0xc6,
                0x88, 0x2b, 0x00, 0x00, 0x95, 0xd7, 0x7f, 0x8e, 0xae, 0xa7, 0xee, 0x12, 0x2f, 0x00,
                0x01, 0xe8, 0x43, 0xe4, 0x0b, 0xdc, 0xc9, 0x35, 0xf9, 0x2f, 0x00, 0x02, 0x33, 0xf0,
                0x73, 0xc6, 0xa9, 0x57, 0x27, 0xe9, 0x2f, 0x00, 0x03, 0x4a, 0x72, 0xde, 0x30, 0x07,
                0x2f, 0x60, 0xad, 0x2f, 0x00, 0x04, 0xca, 0x82, 0xec, 0x00, 0xca, 0xca, 0x04, 0x45,
                0x2f, 0x00, 0x05, 0x3e, 0xfb, 0x5a, 0x2c, 0x63, 0x16, 0x19, 0x39, 0x2f, 0x00, 0x06,
                0x08, 0xca, 0x41, 0x75, 0x2b, 0x9e, 0xb1, 0x96, 0x2f, 0x00, 0x07, 0x65, 0x66, 0x0d,
                0x6a, 0xca, 0xd8, 0x8f, 0xc7, 0x2f, 0x00, 0x08, 0x7a, 0xd4, 0xfe, 0xbc, 0x7c, 0x4e,
                0x29, 0x15,
            ],
            &0i32.to_le_bytes(),
            &[0x00, 0x00, 0x00, 0x00],
        ];
        for bytes in real_records {
            let records = decode_checksums(bytes).unwrap();
            assert_eq!(encode_checksums(&records).unwrap(), bytes);
        }
    }

    #[test]
    fn decode_rejects_an_empty_value() {
        assert!(decode_checksums(&[]).is_err());
    }

    #[test]
    fn decode_rejects_a_negative_count() {
        let bytes = (-1i32).to_le_bytes();
        assert!(decode_checksums(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_a_count_that_overruns_the_value() {
        let mut bytes = 5i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; RECORD_LEN]); // one record's worth, claims five
        assert!(decode_checksums(&bytes).is_err());
    }

    #[test]
    fn decode_rejects_trailing_bytes_past_the_last_full_record() {
        let mut bytes = 1i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; RECORD_LEN]);
        bytes.push(0xaa);
        assert!(decode_checksums(&bytes).is_err());
    }
}

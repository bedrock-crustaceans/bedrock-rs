//! `0x38` [`crate::key::KeyVariant::BorderBlocks`]: the set of border-block
//! positions for a chunk.
//!
//! The value is a one-byte count followed by that many single-byte
//! positions -- `count:u8` then `count` more bytes, total length
//! `count + 1`, with no padding or trailer. Every real record in the corpus
//! (11 across two fixtures) fits this exactly: the first byte always equals
//! the number of bytes that follow it.
//!
//! Each position byte packs two 4-bit local chunk coordinates in 0..16 (the
//! values seen span the full byte range, and the positions per chunk trace
//! contiguous edge-hugging runs, consistent with two packed nibble
//! coordinates rather than one flat count) -- but which nibble is X and
//! which is Z is not established from the available evidence, so this
//! stays an opaque `u8` rather than a struct that would have to guess an
//! axis order.

use crate::error::{Error, Result};

/// Decodes a `0x38` BorderBlocks value: a one-byte count followed by that
/// many single-byte positions, in on-disk order.
///
/// Per decision 11, a count byte that disagrees with the number of bytes
/// that follow it is malformed (not merely unmodelled) and is an error
/// rather than a truncated or padded result. An empty value (missing even
/// the count byte) is likewise malformed.
pub fn decode_border_blocks(bytes: &[u8]) -> Result<Vec<u8>> {
    let Some((&count, positions)) = bytes.split_first() else {
        return Err(Error::Invalid("borderBlocks value is empty"));
    };
    if positions.len() != count as usize {
        return Err(Error::Invalid(
            "borderBlocks count byte does not match the number of positions that follow it",
        ));
    }
    Ok(positions.to_vec())
}

/// Encodes a `0x38` BorderBlocks value: the inverse of
/// [`decode_border_blocks`]. Errors rather than silently truncating if
/// `positions` is longer than a `u8` count can express -- no real chunk
/// needs more than 256 border-block positions (a 16x16 chunk has only 256
/// columns total), so this bound is never expected to bite in practice.
pub fn encode_border_blocks(positions: &[u8]) -> Result<Vec<u8>> {
    let count = u8::try_from(positions.len()).map_err(|_| {
        Error::Invalid("too many positions to encode as a borderBlocks value (max 255)")
    })?;
    let mut out = Vec::with_capacity(1 + positions.len());
    out.push(count);
    out.extend_from_slice(positions);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_accepts_a_real_shaped_record() {
        // From the corpus: chunk (-2, 0) in v1_18_30, 8 bytes, count=7.
        let bytes = [0x07, 0x0a, 0x0b, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f];
        let positions = decode_border_blocks(&bytes).unwrap();
        assert_eq!(positions, vec![0x0a, 0x0b, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]);
    }

    #[test]
    fn decode_accepts_a_single_entry_record() {
        // From the corpus: chunk (0, 0) in v1_18_30, 2 bytes, count=1.
        let bytes = [0x01, 0x82];
        assert_eq!(decode_border_blocks(&bytes).unwrap(), vec![0x82]);
    }

    #[test]
    fn decode_accepts_an_empty_position_list() {
        assert_eq!(decode_border_blocks(&[0x00]).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn decode_rejects_an_empty_value() {
        assert!(decode_border_blocks(&[]).is_err());
    }

    #[test]
    fn decode_rejects_a_count_byte_that_disagrees_with_the_value_length() {
        assert!(decode_border_blocks(&[0x02, 0x01]).is_err());
        assert!(decode_border_blocks(&[0x00, 0x01]).is_err());
    }

    #[test]
    fn round_trips_through_encode_and_decode() {
        let positions = vec![0x00, 0x11, 0x22, 0xff];
        let bytes = encode_border_blocks(&positions).unwrap();
        assert_eq!(bytes, vec![0x04, 0x00, 0x11, 0x22, 0xff]);
        assert_eq!(decode_border_blocks(&bytes).unwrap(), positions);
    }

    #[test]
    fn round_trips_an_empty_position_list() {
        let bytes = encode_border_blocks(&[]).unwrap();
        assert_eq!(bytes, vec![0x00]);
        assert_eq!(decode_border_blocks(&bytes).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn encode_rejects_more_than_255_positions() {
        let positions = vec![0u8; 256];
        assert!(encode_border_blocks(&positions).is_err());
    }

    #[test]
    fn encode_accepts_exactly_255_positions() {
        let positions = vec![0u8; 255];
        let bytes = encode_border_blocks(&positions).unwrap();
        assert_eq!(bytes[0], 255);
        assert_eq!(bytes.len(), 256);
    }
}

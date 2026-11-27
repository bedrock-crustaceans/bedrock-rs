//! `0x36` [`crate::key::KeyVariant::FinalizedState`]: whether a chunk still
//! needs ticking or population before it is considered fully generated.
//!
//! Per architecture decision 13 this record is **preserved, not
//! recomputed** on write -- editing a block does not change how far a
//! chunk's generation has progressed, and only a newly created chunk needs
//! a value chosen for it at all.
//!
//! # On-disk width
//!
//! Every one of the 11,331 real records in the corpus (all fourteen
//! worlds, `v1_12` through the 1.26 seed world) is exactly 4 bytes: a
//! little-endian `i32`. The 1-byte width -- [`FinalizedState::Byte`], a
//! signed byte -- has zero corpus occurrences, not even in `v1_12`, the
//! oldest fixture, but is still modeled: [`decode_finalized_state`] accepts
//! both widths, and [`encode_finalized_state`] reproduces whichever width
//! was decoded, so a real (4-byte) record round-trips exactly and a
//! 1-byte one would too.
//!
//! # Value semantics
//!
//! Of the 11,331 real records, exactly two distinct values were observed:
//! `0` and `2`. `1` never appears in the corpus. Read as a three-state
//! enum (`0` = needs ticking, `1` = needs population, `2` = fully
//! generated), the stage value `1` -- [`Stage::NeedsPopulation`] -- is
//! named from documentation and unobserved in any record. [`Stage`] carries
//! an [`Stage::Unknown`] arm for any other value rather than failing to
//! decode it: an out-of-range value here is unmodelled content the game may
//! define more states for, not a malformed record (architecture decision
//! 11) -- the record's only structural constraint is its byte width.

use crate::error::{Error, Result};

/// A decoded `0x36` value, keeping the on-disk width it was read at. See
/// the module docs for why both widths are modeled despite only the 4-byte
/// form having real corpus evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizedState {
    /// The legacy 1-byte encoding. No real record found uses this width;
    /// see the module docs.
    Byte(i8),
    /// The current 4-byte little-endian encoding. Every real record found
    /// (11,331 of them) uses this width.
    Int(i32),
}

/// The generation-progress states [`FinalizedState`]'s value names. See the
/// module docs for which of these corpus evidence actually confirms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Value `0`: the chunk still needs ticking. Confirmed present in the
    /// corpus.
    NeedsTicking,
    /// Value `1`: the chunk still needs population. Not observed in any
    /// real record; named from documentation, not corpus evidence.
    NeedsPopulation,
    /// Value `2`: the chunk is fully generated. Confirmed present in the
    /// corpus -- the only other value found alongside `NeedsTicking`.
    Done,
    /// Any other value. Carries the raw value through rather than
    /// rejecting it -- see the module docs.
    Unknown(i32),
}

impl FinalizedState {
    /// The value as a plain integer, regardless of which width it was
    /// stored at.
    pub fn raw_value(&self) -> i32 {
        match self {
            FinalizedState::Byte(v) => i32::from(*v),
            FinalizedState::Int(v) => *v,
        }
    }

    /// Resolves [`Self::raw_value`] to a [`Stage`].
    pub fn stage(&self) -> Stage {
        match self.raw_value() {
            0 => Stage::NeedsTicking,
            1 => Stage::NeedsPopulation,
            2 => Stage::Done,
            other => Stage::Unknown(other),
        }
    }
}

/// Decodes a `0x36` value. Accepts both the 1-byte and 4-byte widths (see
/// the module docs); any other length is malformed per architecture
/// decision 11, since the record's width is its only structural
/// constraint.
///
/// # Errors
///
/// Returns [`Error::Invalid`] if `bytes` is neither 1 nor 4 bytes long.
pub fn decode_finalized_state(bytes: &[u8]) -> Result<FinalizedState> {
    match bytes {
        &[b] => Ok(FinalizedState::Byte(b.cast_signed())),
        [a, b, c, d] => Ok(FinalizedState::Int(i32::from_le_bytes([*a, *b, *c, *d]))),
        _ => Err(Error::Invalid(
            "finalizedState value is neither 1 nor 4 bytes",
        )),
    }
}

/// Encodes a `0x36` value: the inverse of [`decode_finalized_state`],
/// reproducing whichever width the value carries.
pub fn encode_finalized_state(value: &FinalizedState) -> Vec<u8> {
    match value {
        FinalizedState::Byte(v) => vec![v.cast_unsigned()],
        FinalizedState::Int(v) => v.to_le_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_real_shaped_needs_ticking_record() {
        let value = decode_finalized_state(&0i32.to_le_bytes()).unwrap();
        assert_eq!(value, FinalizedState::Int(0));
        assert_eq!(value.stage(), Stage::NeedsTicking);
    }

    #[test]
    fn decodes_a_real_shaped_done_record() {
        let value = decode_finalized_state(&2i32.to_le_bytes()).unwrap();
        assert_eq!(value, FinalizedState::Int(2));
        assert_eq!(value.stage(), Stage::Done);
    }

    #[test]
    fn decodes_a_legacy_one_byte_record() {
        let value = decode_finalized_state(&[1u8]).unwrap();
        assert_eq!(value, FinalizedState::Byte(1));
        assert_eq!(value.stage(), Stage::NeedsPopulation);
    }

    #[test]
    fn decodes_a_negative_legacy_byte_as_signed() {
        let value = decode_finalized_state(&[0xff]).unwrap();
        assert_eq!(value, FinalizedState::Byte(-1));
        assert_eq!(value.raw_value(), -1);
        assert_eq!(value.stage(), Stage::Unknown(-1));
    }

    #[test]
    fn unknown_value_does_not_error() {
        let value = decode_finalized_state(&9i32.to_le_bytes()).unwrap();
        assert_eq!(value.stage(), Stage::Unknown(9));
    }

    #[test]
    fn rejects_lengths_other_than_one_or_four() {
        assert!(decode_finalized_state(&[]).is_err());
        assert!(decode_finalized_state(&[0, 0]).is_err());
        assert!(decode_finalized_state(&[0, 0, 0]).is_err());
        assert!(decode_finalized_state(&[0, 0, 0, 0, 0]).is_err());
    }

    #[test]
    fn round_trips_both_widths() {
        for value in [
            FinalizedState::Byte(0),
            FinalizedState::Byte(-1),
            FinalizedState::Byte(2),
            FinalizedState::Int(0),
            FinalizedState::Int(2),
            FinalizedState::Int(-5),
            FinalizedState::Int(i32::MAX),
        ] {
            let bytes = encode_finalized_state(&value);
            assert_eq!(decode_finalized_state(&bytes).unwrap(), value);
        }
    }
}

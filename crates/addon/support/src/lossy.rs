//! Whether a number of a file survives the 32-bit float the game holds it in.

use std::fmt::Write as _;

/// A buffer for the decimal spelling of one float, on the stack.
struct Spelling {
    bytes: [u8; 64],
    len: usize,
}

impl std::fmt::Write for Spelling {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let end = self.len + text.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(std::fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Whether `number`, read into the `f32` nearest to it, is written back as another number:
/// the shortest decimal that names the float is not the number of the file (`0.123456789012`
/// is written `0.12345679`, `16777217` is written `16777216`; `0.1` is written `0.1`).
/// A number beyond a float is not this function's: its reader bounds it and says so.
#[allow(
    clippy::cast_possible_truncation,
    clippy::float_cmp,
    reason = "the narrowing is what is asked about, and it is exact or it is not"
)]
#[must_use]
pub fn narrows(number: f64) -> bool {
    let nearest = number as f32;
    if !nearest.is_finite() || f64::from(nearest) == number {
        return false;
    }
    let mut spelling = Spelling {
        bytes: [0; 64],
        len: 0,
    };
    if write!(spelling, "{nearest:?}").is_err() {
        return true;
    }
    std::str::from_utf8(&spelling.bytes[..spelling.len])
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        != Some(number)
}

#[cfg(test)]
mod tests {
    use super::narrows;

    #[test]
    fn a_number_is_lossy_when_its_float_is_spelled_as_another_number() {
        for kept in [
            0.0,
            -0.0,
            1.0,
            0.1,
            0.9,
            -22.5,
            16_777_216.0,
            1e-7,
            3.4e38,
            0.05,
        ] {
            assert!(!narrows(kept), "{kept}");
        }
        for lost in [0.123_456_789_012, 16_777_217.0, 0.100_000_000_1, 1e-50] {
            assert!(narrows(lost), "{lost}");
        }
        // Beyond a float: the reader's own finding, not this one.
        assert!(!narrows(1e39));
        assert!(!narrows(f64::NAN));
    }
}

//! The jsoncpp integer accessors as the spawn files' hand parsers call them (`asInt`, `asUInt`,
//! `asInt64`), for a member of any width: what `Scalars::Accessors` reads a derived integer
//! member by. Its float, bool and text reads are [`lenient`](super::lenient)'s accessors.

use crate::json::Value;
use crate::json::scalar::fits;

/// What an integer accessor makes of a value for a member of a width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Integer {
    /// An integer token the width holds, as written.
    Fits(i128),
    /// An integer token the width does not hold: its low bits, read as the member's sign.
    Beyond(i128),
    /// A value of another JSON type: a real truncated toward zero and saturating (a 64-bit
    /// member's through 32 bits, sign-extended, as the spawn readers store `asInt64`), a bool 0
    /// or 1, anything else (a `null` too) 0.
    Coerced(i128),
}

/// An integer token's two's-complement bits (the low 64), or `None` for a real or no number.
fn integer_bits(value: &Value) -> Option<u64> {
    let number = value.as_number().filter(|n| n.is_integer())?;
    #[allow(clippy::cast_sign_loss, reason = "two's-complement bits")]
    let bits = |i: i64| i as u64;
    // An integer token fits 64 bits: past them the parser hands over a real.
    number.to_i64().map(bits).or_else(|| number.to_u64())
}

/// An integer token's value.
fn integer(value: &Value) -> Option<i128> {
    let number = value.as_number().filter(|n| n.is_integer())?;
    number
        .to_i64()
        .map(i128::from)
        .or_else(|| number.to_u64().map(i128::from))
}

/// The low `bits` of `raw`, read signed or unsigned.
fn low_bits(raw: u64, bits: u8, signed: bool) -> i128 {
    if bits >= 64 {
        #[allow(clippy::cast_possible_wrap, reason = "two's-complement bits")]
        return if signed {
            i128::from(raw as i64)
        } else {
            i128::from(raw)
        };
    }
    let low = raw & ((1u64 << bits) - 1);
    if signed && low >> (bits - 1) & 1 == 1 {
        i128::from(low) - (1i128 << bits)
    } else {
        i128::from(low)
    }
}

/// The real `x` as the accessor of a member of `bits` and `signed` truncates it.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "the game's float-to-integer conversion saturates, as `as` does"
)]
fn truncated(x: f64, bits: u8, signed: bool) -> i128 {
    if bits >= 64 {
        // `asInt64` as the spawn readers store it: through `int`, sign-extended.
        let through = i64::from(x as i32);
        return if signed {
            i128::from(through)
        } else {
            i128::from(through as u64)
        };
    }
    let (low, high) = if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    };
    (x.trunc().clamp(low as f64, high as f64)) as i128
}

/// `asInt`/`asUInt`/`asInt64` of `value` for a member of `bits` and `signed`.
pub fn as_integer(value: &Value, bits: u8, signed: bool) -> Integer {
    if let (Some(raw), Some(written)) = (integer_bits(value), integer(value)) {
        let read = low_bits(raw, bits, signed);
        return if fits(written, bits, signed) {
            Integer::Fits(read)
        } else {
            Integer::Beyond(read)
        };
    }
    Integer::Coerced(match (value.as_number(), value.as_bool()) {
        (Some(number), _) => truncated(number.to_f64_lossy(), bits, signed),
        (None, Some(flag)) => i128::from(flag),
        (None, None) => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(token: &str, bits: u8, signed: bool) -> Integer {
        as_integer(
            &crate::json::from_str_jsonc(token).expect("a JSON value"),
            bits,
            signed,
        )
    }

    #[test]
    fn an_integer_keeps_its_low_bits_for_the_width() {
        assert_eq!(
            read("2147483647", 32, true),
            Integer::Fits(i128::from(i32::MAX))
        );
        assert_eq!(
            read("2147483648", 32, true),
            Integer::Beyond(i128::from(i32::MIN))
        );
        assert_eq!(read("4294967296", 32, true), Integer::Beyond(0));
        assert_eq!(
            read("-2147483649", 32, true),
            Integer::Beyond(i128::from(i32::MAX))
        );
        assert_eq!(read("18446744073709551615", 32, true), Integer::Beyond(-1));
        assert_eq!(read("4294967296", 32, false), Integer::Beyond(0));
        assert_eq!(read("-1", 32, false), Integer::Beyond(i128::from(u32::MAX)));
        assert_eq!(read("-1", 64, false), Integer::Beyond(i128::from(u64::MAX)));
        assert_eq!(
            read("18446744073709551615", 64, false),
            Integer::Fits(i128::from(u64::MAX))
        );
    }

    #[test]
    fn another_value_is_truncated_or_zero() {
        assert_eq!(
            read("1e12", 32, true),
            Integer::Coerced(i128::from(i32::MAX))
        );
        assert_eq!(read("-2.9", 32, false), Integer::Coerced(0));
        assert_eq!(read("2.9", 64, false), Integer::Coerced(2));
        assert_eq!(
            read("-1.5", 64, false),
            Integer::Coerced(i128::from(u64::MAX))
        );
        assert_eq!(read("true", 32, true), Integer::Coerced(1));
        assert_eq!(read("\"7\"", 32, true), Integer::Coerced(0));
        assert_eq!(read("null", 32, true), Integer::Coerced(0));
    }
}

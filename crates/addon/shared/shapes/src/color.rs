//! Colours: the one strict reader of a hex colour text, and [`HexOr`], the colour a member
//! writes either as that text or as an array of channels.
//!
//! The game's colour members differ in their channel domain (bytes, unclamped integers, unit
//! floats, Molang), in the order of an eight-digit text (`#RRGGBBAA` for a biome's,
//! `#AARRGGBB` for most others) and in what they write back, so a family keeps its own colour
//! type where those differ and shares only the hex reader.

use bedrock_addon_core::json::{Value, ValueExt};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};
use bedrock_addon_core::shared::read;
use bedrock_addon_core::shared::{ShapeError, ShapeErrorKind};
use facet_value::VArray;

/// Whether `text` is `#` and then exactly `digits` hex digits, in either case
/// (`^#[0-9a-fA-F]{digits}$`, the game's colour pattern).
pub fn is_hex(text: &str, digits: usize) -> bool {
    text.strip_prefix('#')
        .is_some_and(|rest| rest.len() == digits && rest.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The number a `#` and `digits` hex digits write (`#ff0080` with six digits is `0xff0080`);
/// `None` for any other text ([`is_hex`]) and for more digits than a `u32` holds (more than
/// eight).
pub fn parse_hex(text: &str, digits: usize) -> Option<u32> {
    if !is_hex(text, digits) {
        return None;
    }
    u32::from_str_radix(&text[1..], 16).ok()
}

/// The `N` bytes a `#` and `2 * N` hex digits write, first pair first (`#ff0080` is
/// `[0xff, 0x00, 0x80]`); `None` for any other text ([`is_hex`]).
pub fn hex_bytes<const N: usize>(text: &str) -> Option<[u8; N]> {
    if !is_hex(text, 2 * N) {
        return None;
    }
    let digits = &text[1..];
    let mut bytes = [0; N];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&digits[2 * index..2 * index + 2], 16).ok()?;
    }
    Some(bytes)
}

/// A colour written as a hex text or as an array of channels, kept in the form it was written
/// (biome and client biome colours, fog colours).
///
/// JSON: a string, kept as written (even one that is not a hex colour, so a later check can
/// report it), or an array of exactly as many channels as `C` holds. `C` says what a channel
/// is ([`Channels`]): `[i32; 3]` for `[r, g, b]` with integer channels the game clamps on use,
/// `[f32; 3]` for unit channels, and so on.
#[derive(Clone, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum HexOr<C: Channels> {
    /// The text, as written (`"#RRGGBB"`).
    Hex(String),
    /// The channels, as written (`[r, g, b]`).
    Array(C),
}

/// What one form of a [`HexOr`] array holds: a fixed number of channels of one kind, read
/// from and written to a JSON array.
pub trait Channels: Sized {
    /// The array's channels as written; the error says what is wrong with `value`.
    ///
    /// # Errors
    ///
    /// [`ShapeErrorKind::WrongType`] for a value that is no array or a channel of the wrong
    /// type, [`ShapeErrorKind::ArrayLength`] for another number of channels and
    /// [`ShapeErrorKind::OutOfRange`] for a channel the type cannot hold.
    fn read(value: &Value) -> Result<Self, ShapeErrorKind>;

    /// The channels as a JSON array.
    ///
    /// # Errors
    ///
    /// [`ShapeErrorKind::NotRepresentable`] for a channel with no JSON form (a non-finite
    /// float).
    fn write(&self) -> Result<Value, ShapeErrorKind>;
}

/// The array of `N` elements `value` holds, or why it is no such array: `expected` says what a
/// colour member takes, in words.
fn elements<'v, const N: usize>(
    value: &'v Value,
    expected: &'static str,
) -> Result<[&'v Value; N], ShapeErrorKind> {
    let array = value.as_array().ok_or(ShapeErrorKind::WrongType {
        expected,
        found: value.json_type(),
    })?;
    let len = array.len();
    let wrong_length = ShapeErrorKind::ArrayLength {
        expected: if N == 3 { "exactly 3" } else { "exactly 4" },
        len,
    };
    if len != N {
        return Err(wrong_length);
    }
    let elements: Vec<&Value> = array.iter().collect();
    elements.try_into().map_err(|_| wrong_length)
}

/// An integer channel (an integer token; a real one is the wrong type, even when integral).
fn integer(value: &Value, expected: &'static str) -> Result<i64, ShapeErrorKind> {
    match value.as_number() {
        Some(number) if !number.is_float() => number.to_i64().ok_or(ShapeErrorKind::OutOfRange),
        _ => Err(ShapeErrorKind::WrongType {
            expected,
            found: value.json_type(),
        }),
    }
}

/// What an integer colour member takes, in words.
const INTEGERS: &str = "a colour string or an array of integers";

/// What a unit colour member takes, in words.
const NUMBERS: &str = "a colour string or an array of numbers";

macro_rules! channels {
    ($($n:literal),+) => {
        $(
            /// `[r, g, b(, a)]` with integer channels, kept as written (the game clamps them
            /// on use).
            impl Channels for [i32; $n] {
                fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
                    let mut out = [0; $n];
                    for (slot, element) in out.iter_mut().zip(elements::<$n>(value, INTEGERS)?) {
                        *slot = i32::try_from(integer(element, INTEGERS)?)
                            .map_err(|_| ShapeErrorKind::OutOfRange)?;
                    }
                    Ok(out)
                }

                fn write(&self) -> Result<Value, ShapeErrorKind> {
                    Ok(Value::from(
                        self.iter()
                            .map(|channel| Value::from(i64::from(*channel)))
                            .collect::<VArray>(),
                    ))
                }
            }

            /// `[r, g, b(, a)]` with integer channels 0 to 255.
            impl Channels for [u8; $n] {
                fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
                    let mut out = [0; $n];
                    for (slot, element) in out.iter_mut().zip(elements::<$n>(value, INTEGERS)?) {
                        *slot = u8::try_from(integer(element, INTEGERS)?)
                            .map_err(|_| ShapeErrorKind::OutOfRange)?;
                    }
                    Ok(out)
                }

                fn write(&self) -> Result<Value, ShapeErrorKind> {
                    Ok(Value::from(
                        self.iter()
                            .map(|channel| Value::from(i64::from(*channel)))
                            .collect::<VArray>(),
                    ))
                }
            }

            /// `[r, g, b(, a)]` with number channels (an integer or a real token, finite as an
            /// `f32`), kept as written.
            impl Channels for [f32; $n] {
                fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
                    let mut out = [0.0; $n];
                    for (slot, element) in out.iter_mut().zip(elements::<$n>(value, NUMBERS)?) {
                        *slot = read::float(read::ROOT, element)
                            .map_err(|failure| ShapeError::from(failure).kind)?;
                    }
                    Ok(out)
                }

                fn write(&self) -> Result<Value, ShapeErrorKind> {
                    let mut array = VArray::new();
                    for channel in self {
                        array.push(
                            read::f32_value(*channel)
                                .map_err(|failure| ShapeError::from(failure).kind)?,
                        );
                    }
                    Ok(Value::from(array))
                }
            }
        )+
    };
}

channels!(3, 4);

impl<C: Channels> TryFrom<Value> for HexOr<C> {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        match value.as_string() {
            Some(text) => Ok(Self::Hex(text.as_str().to_owned())),
            None => C::read(&value).map(Self::Array),
        }
    }
}

impl<C: Channels> TryFrom<&HexOr<C>> for Value {
    type Error = ShapeErrorKind;

    fn try_from(color: &HexOr<C>) -> Result<Self, ShapeErrorKind> {
        match color {
            HexOr::Hex(text) => Ok(Value::from(text.as_str())),
            HexOr::Array(channels) => channels.write(),
        }
    }
}

/// A colour reads its own syntax: no Molang, no bound, no trigger.
impl<C: Channels> MolangFields for HexOr<C> {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl<C: Channels> SchemaFields for HexOr<C> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

/// A channel clamped to `0..=255`.
fn clamp(channel: i32) -> u8 {
    u8::try_from(channel.clamp(0, 255)).unwrap_or(u8::MAX)
}

impl HexOr<[i32; 3]> {
    /// The channels as the game uses them: the hex digits, or the array's channels clamped to
    /// `0..=255`. `None` for a text that is not `#` and six hex digits.
    pub fn rgb(&self) -> Option<[u8; 3]> {
        match self {
            Self::Hex(text) => hex_bytes::<3>(text),
            Self::Array(channels) => Some(channels.map(clamp)),
        }
    }
}

impl HexOr<[i32; 4]> {
    /// The channels as the game uses them: the hex digits, or the array's channels clamped to
    /// `0..=255`. `None` for a text that is not `#` and eight hex digits.
    #[must_use]
    pub fn rgba(&self) -> Option<[u8; 4]> {
        match self {
            Self::Hex(text) => hex_bytes::<4>(text),
            Self::Array(channels) => Some(channels.map(clamp)),
        }
    }
}

/// The three bytes of `#RRGGBB`, read the way the fog settings read their colour text: the
/// first character dropped when it is `#`, and 0 for a digit pair that is not hex.
fn lenient_bytes(text: &str) -> [u8; 3] {
    let digits = text.strip_prefix('#').unwrap_or(text);
    [0, 2, 4].map(|start| {
        digits
            .get(start..start + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .unwrap_or(0)
    })
}

impl HexOr<[u8; 3]> {
    /// The colour as a fog uses it: red, green and blue from 0 to 1 (each byte over 255); a
    /// digit pair of the text that is not hex is 0.
    pub fn rgb(&self) -> [f32; 3] {
        let bytes = match self {
            Self::Hex(text) => lenient_bytes(text),
            Self::Array(rgb) => *rgb,
        };
        bytes.map(|c| f32::from(c) / 255.0)
    }
}

impl HexOr<[f32; 3]> {
    /// The colour as a fog uses it: the text's bytes over 255 (a digit pair that is not hex is
    /// 0), or each channel clamped to 0 to 1.
    pub fn rgb(&self) -> [f32; 3] {
        match self {
            Self::Hex(text) => lenient_bytes(text).map(|c| f32::from(c) / 255.0),
            Self::Array(rgb) => rgb.map(|c| c.clamp(0.0, 1.0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hex_colour_is_a_hash_and_exactly_n_digits_of_either_case() {
        assert!(is_hex("#a0B1c2", 6));
        assert!(is_hex("#00000000", 8));
        assert!(is_hex("#", 0));
        assert!(!is_hex("#a0B1c2", 8));
        assert!(!is_hex("#a0B1c", 6));
        assert!(!is_hex("a0B1c2", 6));
        assert!(!is_hex("#a0B1cg", 6));
        assert!(!is_hex(" #a0B1c2", 6));
        assert!(!is_hex("#a0B1c2 ", 6));
        assert!(!is_hex("#+0B1c2", 6));
        // Six bytes that are not six characters: no hex digit is outside ASCII.
        assert!(!is_hex("#\u{e9}\u{e9}\u{e9}", 6));
    }

    #[test]
    fn a_hex_colour_parses_to_its_number() {
        assert_eq!(parse_hex("#ff0080", 6), Some(0x00ff_0080));
        assert_eq!(parse_hex("#FFFFFFFF", 8), Some(u32::MAX));
        assert_eq!(parse_hex("#ff0080", 8), None);
        assert_eq!(parse_hex("#ff008g", 6), None);
        assert_eq!(parse_hex("#+f0080", 6), None);
        assert_eq!(parse_hex("#ff00ff00ff", 10), None);
    }

    #[test]
    fn hex_bytes_are_the_digit_pairs_in_order() {
        assert_eq!(hex_bytes::<3>("#ff0080"), Some([0xff, 0x00, 0x80]));
        assert_eq!(hex_bytes::<4>("#80ff0001"), Some([0x80, 0xff, 0x00, 0x01]));
        assert_eq!(hex_bytes::<4>("#ff0080"), None);
        assert_eq!(hex_bytes::<3>("#ff00g0"), None);
        assert_eq!(hex_bytes::<3>("ff0080"), None);
    }
}

//! The colours of client biomes: an RGB and an RGBA colour with channels 0 to 255, kept in the
//! form they were written.

use facet::Facet;
use facet_value::VArray;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::JsonType;
use bedrock_addon_core::shared::ShapeErrorKind;

/// An opaque RGB colour: `"#RRGGBB"` or `[r, g, b]` with integer
/// channels, kept in the form it was written.
///
/// The game scales an array's channels by 1/255 and clamps them to `[0, 1]`, so a channel
/// outside `0..=255` is the nearest bound.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum Color255Rgb {
    /// `"#RRGGBB"`, as written.
    Hex(String),
    /// `[r, g, b]`.
    Array([i32; 3]),
}

impl Color255Rgb {
    /// The channels as the game uses them: the hex digits, or the array's channels clamped to
    /// `0..=255`. `None` for a text that is not `#` and six hex digits.
    pub fn rgb(&self) -> Option<[u8; 3]> {
        match self {
            Self::Hex(text) => {
                if !is_hex(text, 6) {
                    return None;
                }
                let channel = |i: usize| u8::from_str_radix(text.get(i..i + 2)?, 16).ok();
                Some([channel(1)?, channel(3)?, channel(5)?])
            }
            Self::Array(channels) => Some(channels.map(clamp)),
        }
    }
}

/// A colour with alpha: `"#RRGGBBAA"` or `[r, g, b, a]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum Color255Rgba {
    /// `"#RRGGBBAA"`, as written.
    Hex(String),
    /// `[r, g, b, a]`.
    Array([i32; 4]),
}

impl Color255Rgba {
    /// The channels as the game uses them: the hex digits, or the array's channels clamped to
    /// `0..=255`. `None` for a text that is not `#` and eight hex digits.
    #[must_use]
    pub fn rgba(&self) -> Option<[u8; 4]> {
        match self {
            Self::Hex(text) => {
                if !is_hex(text, 8) {
                    return None;
                }
                let channel = |i: usize| u8::from_str_radix(text.get(i..i + 2)?, 16).ok();
                Some([channel(1)?, channel(3)?, channel(5)?, channel(7)?])
            }
            Self::Array(channels) => Some(channels.map(clamp)),
        }
    }
}

/// A channel clamped to `0..=255`.
fn clamp(channel: i32) -> u8 {
    u8::try_from(channel.clamp(0, 255)).unwrap_or(u8::MAX)
}

pub(crate) use bedrock_addon_support::biome_json::is_hex;

/// Whether `value` is an RGB colour the game accepts: `"#RRGGBB"` or three integers.
pub(crate) fn is_rgb(value: &Value) -> bool {
    match value.as_string() {
        Some(text) => is_hex(text.as_str(), 6),
        None => channels::<3>(value).is_ok(),
    }
}

/// The channels of an `[r, g, b]` array outside `0..=255`, by index.
pub(crate) fn clamped_channels(value: &Value) -> Vec<usize> {
    channels::<3>(value)
        .map(|channels| {
            (0..3)
                .filter(|&i| !(0..=255).contains(&channels[i]))
                .collect()
        })
        .unwrap_or_default()
}

/// The JSON type of a value.
fn json_type(value: &Value) -> JsonType {
    if value.is_null() {
        JsonType::Null
    } else if value.is_bool() {
        JsonType::Bool
    } else if value.is_number() {
        JsonType::Number
    } else if value.is_string() {
        JsonType::String
    } else if value.is_array() {
        JsonType::Array
    } else {
        JsonType::Object
    }
}

/// `N` integer channels.
fn channels<const N: usize>(value: &Value) -> Result<[i32; N], ShapeErrorKind> {
    let wrong = |found: &Value| ShapeErrorKind::WrongType {
        expected: "a colour string or an array of integers",
        found: json_type(found),
    };
    let array = value.as_array().ok_or_else(|| wrong(value))?;
    if array.len() != N {
        return Err(ShapeErrorKind::ArrayLength {
            expected: if N == 3 { "exactly 3" } else { "exactly 4" },
            len: array.len(),
        });
    }
    let mut out = [0; N];
    for (slot, element) in out.iter_mut().zip(array.iter()) {
        *slot = match element.as_number() {
            Some(number) if !number.is_float() => number
                .to_i64()
                .and_then(|i| i32::try_from(i).ok())
                .ok_or(ShapeErrorKind::OutOfRange)?,
            _ => return Err(wrong(element)),
        };
    }
    Ok(out)
}

/// `channels` as a JSON array.
fn channel_array(channels: &[i32]) -> Value {
    let mut array = VArray::new();
    for channel in channels {
        array.push(Value::from(i64::from(*channel)));
    }
    Value::from(array)
}

impl TryFrom<Value> for Color255Rgb {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        match value.as_string() {
            Some(text) => Ok(Self::Hex(text.as_str().to_owned())),
            None => channels::<3>(&value).map(Self::Array),
        }
    }
}

impl From<&Color255Rgb> for Value {
    fn from(color: &Color255Rgb) -> Self {
        match color {
            Color255Rgb::Hex(text) => Value::from(text.as_str()),
            Color255Rgb::Array(rgb) => channel_array(rgb),
        }
    }
}

impl TryFrom<Value> for Color255Rgba {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        match value.as_string() {
            Some(text) => Ok(Self::Hex(text.as_str().to_owned())),
            None => channels::<4>(&value).map(Self::Array),
        }
    }
}

impl From<&Color255Rgba> for Value {
    fn from(color: &Color255Rgba) -> Self {
        match color {
            Color255Rgba::Hex(text) => Value::from(text.as_str()),
            Color255Rgba::Array(rgba) => channel_array(rgba),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An eight-digit text and an array resolve to their channels, the array clamped.
    #[test]
    fn rgba_resolves_hex_and_clamped_arrays() {
        assert_eq!(
            Color255Rgba::Hex("#01020304".into()).rgba(),
            Some([1, 2, 3, 4])
        );
        assert_eq!(
            Color255Rgba::Array([-5, 300, 7, 255]).rgba(),
            Some([0, 255, 7, 255])
        );
        assert_eq!(Color255Rgba::Hex("#010203".into()).rgba(), None);
    }

    /// Equal colours hash alike, so they can key a set.
    #[test]
    fn colours_can_key_a_set() {
        let set: std::collections::HashSet<Color255Rgb> =
            [Color255Rgb::Array([1, 2, 3]), Color255Rgb::Array([1, 2, 3])].into();
        assert_eq!(set.len(), 1);
    }
}

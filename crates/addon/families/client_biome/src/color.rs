//! The colours of client biomes: an RGB and an RGBA colour with channels 0 to 255, kept in the
//! form they were written.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::ShapeErrorKind;
use bedrock_addon_shared_shapes::{Channels, HexOr};

/// An opaque RGB colour: `"#RRGGBB"` or `[r, g, b]` with integer
/// channels, kept in the form it was written.
///
/// The game scales an array's channels by 1/255 and clamps them to `[0, 1]`, so a channel
/// outside `0..=255` is the nearest bound ([`HexOr::rgb`]).
pub type Color255Rgb = HexOr<[i32; 3]>;

/// A colour with alpha: `"#RRGGBBAA"` or `[r, g, b, a]` ([`HexOr::rgba`]).
pub type Color255Rgba = HexOr<[i32; 4]>;

// The game's colour pattern for biome colour members.
pub(crate) use bedrock_addon_shared_shapes::color::is_hex;

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

/// `N` integer channels, as a colour array reads them.
fn channels<const N: usize>(value: &Value) -> Result<[i32; N], ShapeErrorKind>
where
    [i32; N]: Channels,
{
    <[i32; N]>::read(value)
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

// The colours are read through their proxies, which judge their own form.

//! `description.spawn_egg`: the spawn egg item's look, as written and as vanilla reads it.

use crate::json::{StrictInt, StrictString};

/// `spawn_egg`: how the entity's spawn egg looks, either a texture or two colours.
///
/// Every member is kept as written; [`SpawnEgg::resolved`] gives the values vanilla reads,
/// with its defaults. The same shape at every format version.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct SpawnEgg {
    /// `texture`: the egg's item texture name.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub texture: Option<StrictString>,
    /// `texture_index`: which frame of that texture.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub texture_index: Option<StrictInt<i64>>,
    /// `base_color`: `"#RRGGBB"` or `"#AARRGGBB"`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub base_color: Option<StrictString>,
    /// `overlay_color`: the spots, same form as `base_color`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub overlay_color: Option<StrictString>,
}

/// A spawn egg as vanilla reads it (`SpawnEggInfo`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SpawnEggInfo<'a> {
    /// The texture name; empty when absent.
    pub texture: &'a str,
    /// The frame; 0 when absent or negative.
    pub texture_index: u32,
    /// The base colour; [`Color::NIL`] when absent or unreadable.
    pub base_color: Color,
    /// The overlay colour; [`Color::NIL`] when absent or unreadable.
    pub overlay_color: Color,
}

impl SpawnEgg {
    /// The egg as vanilla reads it: absent members take their defaults (an empty texture,
    /// frame 0, both colours [`Color::NIL`]), and a colour that is not `#` followed by a
    /// hexadecimal number keeps its default silently.
    pub fn resolved(&self) -> SpawnEggInfo<'_> {
        // The game fills the spawn egg info from zero and assigns it whole; `texture_index` is
        // read as an unsigned integer defaulting to 0, so a negative value reads as the default
        // (jsoncpp's exact conversion of a negative integer is not established).
        SpawnEggInfo {
            texture: self
                .texture
                .as_ref()
                .map_or("", |texture| texture.0.as_str()),
            texture_index: self
                .texture_index
                .and_then(|index| u32::try_from(index.0).ok())
                .unwrap_or(0),
            base_color: self
                .base_color
                .as_ref()
                .and_then(|color| Color::from_hex(&color.0))
                .unwrap_or(Color::NIL),
            overlay_color: self
                .overlay_color
                .as_ref()
                .and_then(|color| Color::from_hex(&color.0))
                .unwrap_or(Color::NIL),
        }
    }
}

/// An RGBA colour with components in `0.0..=1.0` (vanilla's colour type).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
    /// Alpha.
    pub a: f32,
}

impl Default for Color {
    /// [`Color::NIL`].
    fn default() -> Self {
        Self::NIL
    }
}

impl Color {
    /// Every component 0 (vanilla's nil colour), the spawn egg default.
    pub const NIL: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Reads `"#RRGGBB"` or `"#AARRGGBB"`: `#` and a hexadecimal number whose bits 16–23 are
    /// red, 8–15 green, 0–7 blue and 24–31 alpha, each divided by 255. So `"#RRGGBB"` has
    /// alpha 0. `None` without the `#`, without digits, with a non-hex digit, or above
    /// `0xFFFFFFFF`.
    pub fn from_hex(text: &str) -> Option<Color> {
        // The game reads `#` and then a base-16 unsigned 32-bit integer. Rejecting a trailing
        // non-hex digit is the crate's own choice: whether the game's conversion stops there or
        // fails is not established.
        let digits = text.strip_prefix('#')?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let [a, r, g, b] = u32::from_str_radix(digits, 16).ok()?.to_be_bytes();
        let channel = |value: u8| f32::from(value) / 255.0;
        Some(Color {
            r: channel(r),
            g: channel(g),
            b: channel(b),
            a: channel(a),
        })
    }
}

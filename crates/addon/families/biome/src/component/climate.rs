//! Climate, colour and classification components: `minecraft:climate`,
//! `minecraft:creature_spawn_probability`, `minecraft:humidity`, `minecraft:map_tints`,
//! `minecraft:partially_frozen`, `minecraft:tags` and `minecraft:village_type`.

use facet::Facet;
use facet_value::VArray;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::ValueExt;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictString, Value};
use bedrock_addon_core::shared::ShapeErrorKind;

/// `minecraft:climate`: temperature, downfall and snow cover. Every member is optional; an
/// absent one leaves the game's built-in value.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Climate {
    /// How much precipitation falls (JSON `downfall`); below `2^-23` the biome gets no rain.
    #[facet(default)]
    pub downfall: Option<StrictFloat<f32>>,
    /// The minimum and maximum snow cover, each multiple of 0.125 one snow layer (JSON
    /// `snow_accumulation`, `[min, max]`; the minimum must not exceed the maximum).
    #[facet(default)]
    pub snow_accumulation: Option<[StrictFloat<f32>; 2]>,
    /// The temperature (JSON `temperature`).
    #[facet(default)]
    pub temperature: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:creature_spawn_probability`: the chance that creatures spawn while chunks
/// generate.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct CreatureSpawnProbability {
    /// The probability, in `[0.0, 0.75]` (JSON `probability`).
    #[facet(default)]
    pub probability: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:humidity`: whether the biome counts as humid (fire spread, farmland).
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Humidity {
    /// Humid or dry (JSON `is_humid`, required).
    pub is_humid: StrictBool,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:map_tints`: the colours a map draws the biome's grass and foliage with.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MapTints {
    /// The grass colour (JSON `grass`, required).
    pub grass: GrassTint,
    /// The foliage colour (JSON `foliage`).
    #[facet(default)]
    pub foliage: Option<Color>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:map_tints.grass`: a fixed colour or the noise-based one, chosen by `type`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(tag = "type")]
#[repr(u8)]
pub enum GrassTint {
    /// `{"type": "tint", "tint": <colour>}`: one colour.
    #[facet(rename = "tint")]
    Tint {
        /// The colour (JSON `tint`, required).
        tint: Color,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// `{"type": "noise"}`: the colour the game picks by noise.
    #[facet(rename = "noise")]
    Noise {
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// An opaque RGB colour: `"#RRGGBB"` or `[r, g, b]` with integer
/// channels, kept in the form it was written.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum Color {
    /// `"#RRGGBB"`, as written.
    Hex(String),
    /// `[r, g, b]`.
    Rgb([i32; 3]),
}

/// A colour with alpha: `"#RRGGBBAA"` or `[r, g, b, a]`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum ColorRgba {
    /// `"#RRGGBBAA"`, as written.
    Hex(String),
    /// `[r, g, b, a]`.
    Rgba([i32; 4]),
}

/// `N` integer channels.
fn channels<const N: usize>(value: &Value) -> Result<[i32; N], ShapeErrorKind> {
    let wrong = |found: &Value| ShapeErrorKind::WrongType {
        expected: "a colour string or an array of integers",
        found: found.json_type(),
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

impl TryFrom<Value> for Color {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        match value.as_string() {
            Some(text) => Ok(Self::Hex(text.as_str().to_owned())),
            None => channels::<3>(&value).map(Self::Rgb),
        }
    }
}

impl From<&Color> for Value {
    fn from(color: &Color) -> Self {
        match color {
            Color::Hex(text) => Value::from(text.as_str()),
            Color::Rgb(rgb) => channel_array(rgb),
        }
    }
}

impl TryFrom<Value> for ColorRgba {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        match value.as_string() {
            Some(text) => Ok(Self::Hex(text.as_str().to_owned())),
            None => channels::<4>(&value).map(Self::Rgba),
        }
    }
}

impl From<&ColorRgba> for Value {
    fn from(color: &ColorRgba) -> Self {
        match color {
            ColorRgba::Hex(text) => Value::from(text.as_str()),
            ColorRgba::Rgba(rgba) => channel_array(rgba),
        }
    }
}

/// `minecraft:partially_frozen`: the biome is partly frozen by noise (no members).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PartiallyFrozen {
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:tags`: the biome's tags, which features, spawn rules and Molang queries test.
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Tags {
    /// The tags (JSON `tags`, required).
    pub tags: Vec<StrictString>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Tags {
    /// Whether the biome has `tag`.
    pub fn contains(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.0 == tag)
    }
}

/// `minecraft:village_type` (format 1.26.0): which village style generates in the biome.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct VillageType {
    /// The style (JSON `type`, required).
    #[facet(rename = "type")]
    pub kind: VillageKind,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl VillageType {
    /// The component with `kind` and no unknown members.
    pub fn new(kind: VillageKind) -> Self {
        Self {
            kind,
            extra: Extra::default(),
        }
    }
}

/// A village style; the game numbers the five styles 0 to 4 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum VillageKind {
    Desert,
    Ice,
    Savanna,
    Taiga,
    Default,
}

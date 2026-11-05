//! Climate, colour and classification components: `minecraft:climate`,
//! `minecraft:creature_spawn_probability`, `minecraft:humidity`, `minecraft:map_tints`,
//! `minecraft:partially_frozen`, `minecraft:tags` and `minecraft:village_type`.

use facet::Facet;

use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_shared_shapes::HexOr;

/// `minecraft:climate`: temperature, downfall and snow cover. Every member is optional; an
/// absent one leaves the game's built-in value.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Climate {
    /// How much precipitation falls (JSON `downfall`); below `2^-23` the biome gets no rain.
    pub downfall: Option<f32>,
    /// The minimum and maximum snow cover, each multiple of 0.125 one snow layer (JSON
    /// `snow_accumulation`, `[min, max]`; the minimum must not exceed the maximum).
    pub snow_accumulation: Option<[f32; 2]>,
    /// The temperature (JSON `temperature`).
    pub temperature: Option<f32>,
}

/// `minecraft:creature_spawn_probability`: the chance that creatures spawn while chunks
/// generate.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct CreatureSpawnProbability {
    /// The probability, in `[0.0, 0.75]` (JSON `probability`).
    pub probability: Option<f32>,
}

/// `minecraft:humidity`: whether the biome counts as humid (fire spread, farmland).
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Humidity {
    /// Humid or dry (JSON `is_humid`, required).
    pub is_humid: bool,
}

/// `minecraft:map_tints`: the colours a map draws the biome's grass and foliage with.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MapTints {
    /// The grass colour (JSON `grass`, required).
    pub grass: GrassTint,
    /// The foliage colour (JSON `foliage`).
    pub foliage: Option<Color>,
}

/// `minecraft:map_tints.grass`: a fixed colour or the noise-based one, chosen by `type`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(tag = "type")]
#[repr(u8)]
pub enum GrassTint {
    /// `{"type": "tint", "tint": <colour>}`: one colour.
    #[facet(rename = "tint")]
    #[facet(schema::unknown = "keep")]
    Tint {
        /// The colour (JSON `tint`, required).
        tint: Color,
    },
    /// `{"type": "noise"}`: the colour the game picks by noise.
    #[facet(rename = "noise")]
    #[facet(schema::unknown = "keep")]
    Noise {},
}

/// An opaque RGB colour: `"#RRGGBB"` or `[r, g, b]` with integer
/// channels, kept in the form it was written.
pub type Color = HexOr<[i32; 3]>;

/// A colour with alpha: `"#RRGGBBAA"` or `[r, g, b, a]`.
pub type ColorRgba = HexOr<[i32; 4]>;

/// `minecraft:partially_frozen`: the biome is partly frozen by noise (no members).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PartiallyFrozen {}

/// `minecraft:tags`: the biome's tags, which features, spawn rules and Molang queries test.
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Tags {
    /// The tags (JSON `tags`, required).
    pub tags: Vec<String>,
}

impl Tags {
    /// Whether the biome has `tag`.
    pub fn contains(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// `minecraft:village_type` (format 1.26.0): which village style generates in the biome.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct VillageType {
    /// The style (JSON `type`, required).
    #[facet(rename = "type")]
    pub kind: VillageKind,
}

impl VillageType {
    /// The component with `kind` and no unknown members.
    pub fn new(kind: VillageKind) -> Self {
        Self { kind }
    }
}

/// A village style; the game numbers the five styles 0 to 4 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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

// The colours are read through their proxies, which judge their own form (shapes' impls); the
// tint is a data enum, its members checked by hand (`validate.rs`).
bedrock_addon_core::impl_schema_leaf!(GrassTint);
bedrock_addon_core::impl_no_molang!(GrassTint);

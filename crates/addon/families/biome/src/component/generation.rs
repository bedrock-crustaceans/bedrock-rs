//! World-generation components: `minecraft:mountain_parameters`,
//! `minecraft:multinoise_generation_rules`, `minecraft:overworld_generation_rules`,
//! `minecraft:overworld_height` and `minecraft:replace_biomes`.

use facet::Facet;
use facet_value::VArray;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::ValueExt;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictString, Value};
use bedrock_addon_core::shared::{BlockSpecifier, ShapeErrorKind};

/// `minecraft:mountain_parameters`: steep-slope materials and the top slide of mountain
/// terrain. Every member is optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MountainParameters {
    /// The block on steep slopes and the sides it covers (JSON `steep_material_adjustment`).
    #[facet(default)]
    pub steep_material_adjustment: Option<SteepMaterialAdjustment>,
    /// The density reduction near the world's top (JSON `top_slide`).
    #[facet(default)]
    pub top_slide: Option<TopSlide>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `mountain_parameters.steep_material_adjustment`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SteepMaterialAdjustment {
    /// The block placed on steep slopes (JSON `material`).
    #[facet(default)]
    pub material: Option<BlockSpecifier>,
    /// Slopes facing north (JSON `north_slopes`).
    #[facet(default)]
    pub north_slopes: Option<StrictBool>,
    /// Slopes facing south (JSON `south_slopes`).
    #[facet(default)]
    pub south_slopes: Option<StrictBool>,
    /// Slopes facing west (JSON `west_slopes`).
    #[facet(default)]
    pub west_slopes: Option<StrictBool>,
    /// Slopes facing east (JSON `east_slopes`).
    #[facet(default)]
    pub east_slopes: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `mountain_parameters.top_slide`.
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TopSlide {
    /// Whether the top slide applies (JSON `enabled`, required).
    pub enabled: StrictBool,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:multinoise_generation_rules`: where the biome lands in the multi-noise climate
/// space. The game applies it only to biomes in the `minecraft` namespace.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MultinoiseGenerationRules {
    /// JSON `target_temperature`.
    #[facet(default)]
    pub target_temperature: Option<StrictFloat<f32>>,
    /// JSON `target_humidity`.
    #[facet(default)]
    pub target_humidity: Option<StrictFloat<f32>>,
    /// JSON `target_altitude`.
    #[facet(default)]
    pub target_altitude: Option<StrictFloat<f32>>,
    /// JSON `target_weirdness`.
    #[facet(default)]
    pub target_weirdness: Option<StrictFloat<f32>>,
    /// JSON `weight`.
    #[facet(default)]
    pub weight: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:overworld_generation_rules`: the legacy overworld generator's transformations
/// and climates.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct OverworldGenerationRules {
    /// The biomes that replace this one as hills (JSON `hills_transformation`).
    #[facet(default)]
    pub hills_transformation: Option<WeightedBiomes>,
    /// The biomes that replace this one as its mutated variant (JSON `mutate_transformation`).
    #[facet(default)]
    pub mutate_transformation: Option<WeightedBiomes>,
    /// The climates and weights the biome generates for (JSON `generate_for_climates`).
    #[facet(default)]
    pub generate_for_climates: Option<Vec<WeightedClimate>>,
    /// The biomes along its rivers (JSON `river_transformation`).
    #[facet(default)]
    pub river_transformation: Option<WeightedBiomes>,
    /// The biomes along its shores (JSON `shore_transformation`).
    #[facet(default)]
    pub shore_transformation: Option<WeightedBiomes>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A transformation target: one biome name, or a list of names, each with an optional weight.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum WeightedBiomes {
    /// `"<biome>"`: one biome of weight 1.
    One(String),
    /// `[<entry>, …]`.
    List(Vec<WeightedBiome>),
}

/// One entry of a [`WeightedBiomes`] list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeightedBiome {
    /// `"<biome>"`: weight 1.
    Name(String),
    /// `[<name or weight>, <name or weight>]`, as written; the game requires the name first
    /// and the weight second.
    Pair([NameOrWeight; 2]),
}

/// One element of a two-element weighted entry: a name or a non-negative integer weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrWeight {
    /// A string.
    Name(String),
    /// A non-negative integer.
    Weight(u32),
}

/// An entry of `generate_for_climates`: `[<category or weight>, <category or weight>]`, as
/// written; the game requires the category first and the weight second.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct WeightedClimate(pub [CategoryOrWeight; 2]);

/// One element of a [`WeightedClimate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CategoryOrWeight {
    /// A temperature category.
    Category(TemperatureCategory),
    /// A non-negative integer.
    Weight(u32),
}

/// A temperature category of the legacy overworld generator; the game numbers the five
/// categories 0 to 4 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum TemperatureCategory {
    Medium,
    Warm,
    Lukewarm,
    Cold,
    Frozen,
}

impl TemperatureCategory {
    /// The JSON spelling.
    pub fn name(self) -> &'static str {
        match self {
            Self::Medium => "medium",
            Self::Warm => "warm",
            Self::Lukewarm => "lukewarm",
            Self::Cold => "cold",
            Self::Frozen => "frozen",
        }
    }

    /// The category spelled `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "medium" => Self::Medium,
            "warm" => Self::Warm,
            "lukewarm" => Self::Lukewarm,
            "cold" => Self::Cold,
            "frozen" => Self::Frozen,
            _ => return None,
        })
    }
}

/// A non-negative integer weight.
fn weight(value: &Value) -> Option<u32> {
    let number = value.as_number()?;
    if number.is_float() {
        return None;
    }
    u32::try_from(number.to_i64()?).ok()
}

/// A two-element array, or the shape error.
fn pair(value: &Value) -> Result<[&Value; 2], ShapeErrorKind> {
    let array = value.as_array().ok_or(ShapeErrorKind::WrongType {
        expected: "an array of 2 elements",
        found: value.json_type(),
    })?;
    match array.as_slice() {
        [a, b] => Ok([a, b]),
        other => Err(ShapeErrorKind::ArrayLength {
            expected: "exactly 2",
            len: other.len(),
        }),
    }
}

impl NameOrWeight {
    fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
        if let Some(name) = value.as_string() {
            return Ok(Self::Name(name.as_str().to_owned()));
        }
        weight(value)
            .map(Self::Weight)
            .ok_or(ShapeErrorKind::WrongType {
                expected: "a biome name or a non-negative integer",
                found: value.json_type(),
            })
    }

    fn write(&self) -> Value {
        match self {
            Self::Name(name) => Value::from(name.as_str()),
            Self::Weight(weight) => Value::from(u64::from(*weight)),
        }
    }
}

impl WeightedBiome {
    fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
        if let Some(name) = value.as_string() {
            return Ok(Self::Name(name.as_str().to_owned()));
        }
        let [a, b] = pair(value)?;
        Ok(Self::Pair([NameOrWeight::read(a)?, NameOrWeight::read(b)?]))
    }

    fn write(&self) -> Value {
        match self {
            Self::Name(name) => Value::from(name.as_str()),
            Self::Pair([a, b]) => {
                let mut array = VArray::new();
                array.push(a.write());
                array.push(b.write());
                Value::from(array)
            }
        }
    }
}

impl TryFrom<Value> for WeightedBiomes {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        if let Some(name) = value.as_string() {
            return Ok(Self::One(name.as_str().to_owned()));
        }
        let array = value.as_array().ok_or(ShapeErrorKind::WrongType {
            expected: "a biome name or an array",
            found: value.json_type(),
        })?;
        array
            .iter()
            .map(WeightedBiome::read)
            .collect::<Result<_, _>>()
            .map(Self::List)
    }
}

impl From<&WeightedBiomes> for Value {
    fn from(biomes: &WeightedBiomes) -> Self {
        match biomes {
            WeightedBiomes::One(name) => Value::from(name.as_str()),
            WeightedBiomes::List(entries) => {
                let mut array = VArray::new();
                for entry in entries {
                    array.push(entry.write());
                }
                Value::from(array)
            }
        }
    }
}

impl CategoryOrWeight {
    fn read(value: &Value) -> Result<Self, ShapeErrorKind> {
        if let Some(name) = value.as_string() {
            return TemperatureCategory::from_name(name.as_str())
                .map(Self::Category)
                .ok_or(ShapeErrorKind::UnknownName);
        }
        weight(value)
            .map(Self::Weight)
            .ok_or(ShapeErrorKind::WrongType {
                expected: "a temperature category or a non-negative integer",
                found: value.json_type(),
            })
    }

    fn write(&self) -> Value {
        match self {
            Self::Category(category) => Value::from(category.name()),
            Self::Weight(weight) => Value::from(u64::from(*weight)),
        }
    }
}

impl TryFrom<Value> for WeightedClimate {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        let [a, b] = pair(&value)?;
        Ok(Self([
            CategoryOrWeight::read(a)?,
            CategoryOrWeight::read(b)?,
        ]))
    }
}

impl From<&WeightedClimate> for Value {
    fn from(climate: &WeightedClimate) -> Self {
        let mut array = VArray::new();
        array.push(climate.0[0].write());
        array.push(climate.0[1].write());
        Value::from(array)
    }
}

/// `minecraft:overworld_height`: the legacy generator's terrain height, as a preset or as
/// `[depth, scale]`, not both.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct OverworldHeight {
    /// A height preset (JSON `noise_type`).
    #[facet(default)]
    pub noise_type: Option<NoiseType>,
    /// `[depth, scale]` (JSON `noise_params`).
    #[facet(default)]
    pub noise_params: Option<[StrictFloat<f32>; 2]>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A terrain-height preset; the game numbers the fifteen presets 0 to 14 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum NoiseType {
    Default,
    DefaultMutated,
    River,
    Ocean,
    DeepOcean,
    Lowlands,
    Taiga,
    Mountains,
    Highlands,
    Extreme,
    LessExtreme,
    Beach,
    StoneBeach,
    Mushroom,
    Swamp,
}

impl NoiseType {
    /// The `(depth, scale)` the preset stands for.
    // The game keeps the depths and the scales in two tables; which table is the depth and
    // which the scale follows the order of the values.
    pub fn depth_and_scale(self) -> (f32, f32) {
        match self {
            Self::Default => (0.1, 0.2),
            Self::DefaultMutated => (0.2, 0.4),
            Self::Taiga => (0.2, 0.2),
            Self::River => (-0.5, 0.0),
            Self::Ocean => (-1.0, 0.1),
            Self::DeepOcean => (-1.8, 0.1),
            Self::Lowlands => (0.125, 0.05),
            Self::Mountains => (0.45, 0.3),
            Self::Highlands => (1.5, 0.025),
            Self::Extreme => (1.0, 0.5),
            Self::LessExtreme => (0.8, 0.4),
            Self::Beach => (0.0, 0.025),
            Self::StoneBeach => (0.1, 0.8),
            Self::Mushroom => (0.2, 0.3),
            Self::Swamp => (-0.2, 0.1),
        }
    }
}

/// `minecraft:replace_biomes`: replaces vanilla biomes, by noise, with this one.
// Required members: no `Default`, or facet would default them.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ReplaceBiomes {
    /// The replacements (JSON `replacements`, required, at least one).
    pub replacements: Vec<BiomeReplacement>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `replace_biomes.replacements`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BiomeReplacement {
    /// The dimension: `minecraft:overworld`, or from format 1.26.0 also `minecraft:nether`
    /// (JSON `dimension`, required).
    pub dimension: StrictString,
    /// The vanilla biomes to replace, by name without namespace (JSON `targets`, required, at
    /// least one).
    pub targets: Vec<StrictString>,
    /// How much of the targets to replace, in `(0, 1]` (JSON `amount`, required).
    pub amount: StrictFloat<f32>,
    /// The scale of the replacement noise, in `(0, 100]` (JSON `noise_frequency_scale`,
    /// required).
    pub noise_frequency_scale: StrictFloat<f32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

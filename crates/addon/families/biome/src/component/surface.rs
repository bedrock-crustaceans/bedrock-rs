//! Surface components: `minecraft:surface_builder`, `minecraft:subsurface_builder` and
//! `minecraft:surface_material_adjustments`.

use facet::Facet;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::shared::{BlockSpecifier, FloatRange};

/// `minecraft:surface_builder`: how the biome's surface is built.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SurfaceBuilder {
    /// The builder, chosen by its `type` (JSON `builder`, required).
    pub builder: Builder,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:subsurface_builder` (format 1.26.30): a second builder of the same shape that
/// shapes the ground below the surface.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SubsurfaceBuilder {
    /// The builder, chosen by its `type` (JSON `builder`, required).
    pub builder: Builder,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A surface builder, chosen by `type`. Every member of every builder is required unless it is
/// an `Option`. The game numbers the builders 0 overworld, 1 swamp, 2 `frozen_ocean`, 3 capped,
/// 4 `the_end` and 5 mesa; `noise_gradient` (7) joins them in format 1.26.30.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(tag = "type")]
#[repr(u8)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "the alternatives are plain data read once per document"
)]
pub enum Builder {
    /// `minecraft:overworld`: the usual layered surface.
    #[facet(rename = "minecraft:overworld")]
    Overworld(OverworldSurface),
    /// `minecraft:swamp`: the overworld surface with puddles below sea level.
    #[facet(rename = "minecraft:swamp")]
    Swamp(SwampSurface),
    /// `minecraft:frozen_ocean`: the overworld surface with icebergs.
    #[facet(rename = "minecraft:frozen_ocean")]
    FrozenOcean(OverworldSurface),
    /// `minecraft:capped`: floors and ceilings of the Nether.
    #[facet(rename = "minecraft:capped")]
    Capped(CappedSurface),
    /// `minecraft:the_end`: the End's surface.
    #[facet(rename = "minecraft:the_end")]
    TheEnd(TheEndSurface),
    /// `minecraft:mesa`: the badlands' terracotta bands.
    #[facet(rename = "minecraft:mesa")]
    Mesa(MesaSurface),
    /// `minecraft:noise_gradient` (format 1.26.30): blocks chosen by a noise value.
    #[facet(rename = "minecraft:noise_gradient")]
    NoiseGradient(NoiseGradientSurface),
}

impl Builder {
    /// The JSON `type` of the builder.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Overworld(_) => "minecraft:overworld",
            Self::Swamp(_) => "minecraft:swamp",
            Self::FrozenOcean(_) => "minecraft:frozen_ocean",
            Self::Capped(_) => "minecraft:capped",
            Self::TheEnd(_) => "minecraft:the_end",
            Self::Mesa(_) => "minecraft:mesa",
            Self::NoiseGradient(_) => "minecraft:noise_gradient",
        }
    }
}

/// The members of the `minecraft:overworld` and `minecraft:frozen_ocean` builders.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct OverworldSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: StrictInt<i32>,
    /// The sea floor (JSON `sea_floor_material`).
    pub sea_floor_material: BlockSpecifier,
    /// The deepest layer (JSON `foundation_material`).
    pub foundation_material: BlockSpecifier,
    /// The layer below the top (JSON `mid_material`).
    pub mid_material: BlockSpecifier,
    /// The top layer (JSON `top_material`).
    pub top_material: BlockSpecifier,
    /// The fluid of the sea (JSON `sea_material`).
    pub sea_material: BlockSpecifier,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The members of the `minecraft:swamp` builder.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SwampSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: StrictInt<i32>,
    /// The sea floor (JSON `sea_floor_material`).
    pub sea_floor_material: BlockSpecifier,
    /// The deepest layer (JSON `foundation_material`).
    pub foundation_material: BlockSpecifier,
    /// The layer below the top (JSON `mid_material`).
    pub mid_material: BlockSpecifier,
    /// The top layer (JSON `top_material`).
    pub top_material: BlockSpecifier,
    /// The fluid of the sea (JSON `sea_material`).
    pub sea_material: BlockSpecifier,
    /// How far below sea level puddles reach, in `[0, 127]` (JSON
    /// `max_puddle_depth_below_sea_level`).
    pub max_puddle_depth_below_sea_level: StrictInt<i32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The members of the `minecraft:capped` builder.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CappedSurface {
    /// The floor blocks, at least one (JSON `floor_materials`).
    pub floor_materials: Vec<BlockSpecifier>,
    /// The ceiling blocks, at least one (JSON `ceiling_materials`).
    pub ceiling_materials: Vec<BlockSpecifier>,
    /// The fluid of the sea (JSON `sea_material`).
    pub sea_material: BlockSpecifier,
    /// The deepest layer (JSON `foundation_material`).
    pub foundation_material: BlockSpecifier,
    /// The beach (JSON `beach_material`).
    #[facet(default)]
    pub beach_material: Option<BlockSpecifier>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The `minecraft:the_end` builder (no members).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TheEndSurface {
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The members of the `minecraft:mesa` builder.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MesaSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: StrictInt<i32>,
    /// The sea floor (JSON `sea_floor_material`).
    pub sea_floor_material: BlockSpecifier,
    /// The deepest layer (JSON `foundation_material`).
    pub foundation_material: BlockSpecifier,
    /// The layer below the top (JSON `mid_material`).
    pub mid_material: BlockSpecifier,
    /// The top layer (JSON `top_material`).
    pub top_material: BlockSpecifier,
    /// The fluid of the sea (JSON `sea_material`).
    pub sea_material: BlockSpecifier,
    /// The clay of the bands (JSON `clay_material`).
    pub clay_material: BlockSpecifier,
    /// The hardened clay (JSON `hard_clay_material`).
    pub hard_clay_material: BlockSpecifier,
    /// Whether bryce pillars generate (JSON `bryce_pillars`).
    pub bryce_pillars: StrictBool,
    /// Whether the plateau has a forest (JSON `has_forest`).
    pub has_forest: StrictBool,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The members of the `minecraft:noise_gradient` builder (format 1.26.30).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct NoiseGradientSurface {
    /// The noise the gradient reads (JSON `noise_descriptor`).
    pub noise_descriptor: NoiseDescriptor,
    /// Blocks the builder never replaces (JSON `non_replaceable_blocks`).
    #[facet(default)]
    pub non_replaceable_blocks: Option<Vec<BlockSpecifier>>,
    /// The block of each noise range, at least one (JSON `noise_block_specifiers`).
    pub noise_block_specifiers: Vec<NoiseBlockSpecifier>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A noise: its name, first octave and octave amplitudes.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct NoiseDescriptor {
    /// The noise's seed name (JSON `name`).
    pub name: StrictString,
    /// The first octave (JSON `first_octave`).
    pub first_octave: StrictInt<i32>,
    /// The amplitude of each octave, 1 to 100 of them (JSON `amplitudes`).
    pub amplitudes: Vec<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One block of a noise gradient and the noise range it covers.
/// `noise`, `threshold` and `range` are optional; `block` is required.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct NoiseBlockSpecifier {
    /// A namespaced noise name (JSON `noise`; `""` when absent).
    #[facet(default)]
    pub noise: Option<StrictString>,
    /// The threshold (JSON `threshold`; `0` when absent).
    #[facet(default)]
    pub threshold: Option<StrictFloat<f32>>,
    /// The closed noise range `{min, max}` the block covers (JSON `range`; `{0, 0}` when
    /// absent).
    #[facet(default)]
    pub range: Option<FloatRange>,
    /// The block (JSON `block`).
    pub block: BlockSpecifier,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl NoiseBlockSpecifier {
    /// The noise name, `""` when absent.
    pub fn noise(&self) -> &str {
        self.noise.as_ref().map_or("", |noise| noise.0.as_str())
    }

    /// The threshold, `0` when absent.
    pub fn threshold(&self) -> f32 {
        self.threshold.map_or(0.0, |t| t.0)
    }

    /// The range's ends as written, `(0, 0)` when absent (an absent end is `0`).
    pub fn range(&self) -> (f32, f32) {
        self.range.as_ref().map_or((0.0, 0.0), |range| {
            // `FloatRange` orders a reversed pair; the game keeps the ends as written.
            if range.swapped {
                (range.max, range.min)
            } else {
                (range.min, range.max)
            }
        })
    }
}

/// `minecraft:surface_material_adjustments`: material changes applied, in order, where noise
/// and height ranges match.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SurfaceMaterialAdjustments {
    /// The adjustments (JSON `adjustments`).
    #[facet(default)]
    pub adjustments: Option<Vec<Adjustment>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `surface_material_adjustments.adjustments`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Adjustment {
    /// The materials it sets (JSON `materials`, required).
    pub materials: AdjustmentMaterials,
    /// The noise range `[min, max]` it applies in; the maximum must exceed the minimum (JSON
    /// `noise_range`).
    #[facet(default)]
    pub noise_range: Option<[StrictFloat<f32>; 2]>,
    /// The height range `[min, max]` it applies in, each a number, bool or Molang expression
    /// compiled at Molang version 1 with the `world_gen` queries (JSON `height_range`).
    #[facet(default)]
    pub height_range: Option<[Molang; 2]>,
    /// The scale of the adjustment noise (JSON `noise_frequency_scale`).
    #[facet(default)]
    pub noise_frequency_scale: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `adjustments[].materials`: every member optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct AdjustmentMaterials {
    /// JSON `top_material`.
    #[facet(default)]
    pub top_material: Option<BlockSpecifier>,
    /// JSON `mid_material`.
    #[facet(default)]
    pub mid_material: Option<BlockSpecifier>,
    /// JSON `sea_floor_material`.
    #[facet(default)]
    pub sea_floor_material: Option<BlockSpecifier>,
    /// JSON `foundation_material`.
    #[facet(default)]
    pub foundation_material: Option<BlockSpecifier>,
    /// JSON `sea_material`.
    #[facet(default)]
    pub sea_material: Option<BlockSpecifier>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

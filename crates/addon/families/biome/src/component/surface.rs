//! Surface components: `minecraft:surface_builder`, `minecraft:subsurface_builder` and
//! `minecraft:surface_material_adjustments`.

use facet::Facet;

use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{BlockSpecifier, FloatRange};

/// `minecraft:surface_builder`: how the biome's surface is built.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SurfaceBuilder {
    /// The builder, chosen by its `type` (JSON `builder`, required).
    pub builder: Builder,
}

/// `minecraft:subsurface_builder` (format 1.26.30): a second builder of the same shape that
/// shapes the ground below the surface.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SubsurfaceBuilder {
    /// The builder, chosen by its `type` (JSON `builder`, required).
    pub builder: Builder,
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OverworldSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: i32,
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
}

/// The members of the `minecraft:swamp` builder.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SwampSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: i32,
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
    pub max_puddle_depth_below_sea_level: i32,
}

/// The members of the `minecraft:capped` builder.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
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
    pub beach_material: Option<BlockSpecifier>,
}

/// The `minecraft:the_end` builder (no members).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct TheEndSurface {}

/// The members of the `minecraft:mesa` builder.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MesaSurface {
    /// How deep the sea floor material goes, in `[0, 127]` (JSON `sea_floor_depth`).
    pub sea_floor_depth: i32,
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
    pub bryce_pillars: bool,
    /// Whether the plateau has a forest (JSON `has_forest`).
    pub has_forest: bool,
}

/// The members of the `minecraft:noise_gradient` builder (format 1.26.30).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct NoiseGradientSurface {
    /// The noise the gradient reads (JSON `noise_descriptor`).
    pub noise_descriptor: NoiseDescriptor,
    /// Blocks the builder never replaces (JSON `non_replaceable_blocks`).
    pub non_replaceable_blocks: Option<Vec<BlockSpecifier>>,
    /// The block of each noise range, at least one (JSON `noise_block_specifiers`).
    pub noise_block_specifiers: Vec<NoiseBlockSpecifier>,
}

/// A noise: its name, first octave and octave amplitudes.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct NoiseDescriptor {
    /// The noise's seed name (JSON `name`).
    pub name: String,
    /// The first octave (JSON `first_octave`).
    pub first_octave: i32,
    /// The amplitude of each octave, 1 to 100 of them (JSON `amplitudes`).
    pub amplitudes: Vec<f32>,
}

/// One block of a noise gradient and the noise range it covers.
/// `noise`, `threshold` and `range` are optional; `block` is required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct NoiseBlockSpecifier {
    /// A namespaced noise name (JSON `noise`; `""` when absent).
    pub noise: Option<String>,
    /// The threshold (JSON `threshold`; `0` when absent: [`NoiseBlockSpecifier::threshold`]).
    #[facet(schema::fallback = 0.0)]
    pub threshold: Option<f32>,
    /// The closed noise range `{min, max}` the block covers (JSON `range`; `{0, 0}` when
    /// absent).
    pub range: Option<FloatRange>,
    /// The block (JSON `block`).
    pub block: BlockSpecifier,
}

impl NoiseBlockSpecifier {
    /// The noise name, `""` when absent.
    pub fn noise(&self) -> &str {
        self.noise.as_deref().unwrap_or_default()
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SurfaceMaterialAdjustments {
    /// The adjustments (JSON `adjustments`).
    pub adjustments: Option<Vec<Adjustment>>,
}

/// One entry of `surface_material_adjustments.adjustments`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Adjustment {
    /// The materials it sets (JSON `materials`, required).
    pub materials: AdjustmentMaterials,
    /// The noise range `[min, max]` it applies in; the maximum must exceed the minimum (JSON
    /// `noise_range`).
    pub noise_range: Option<[f32; 2]>,
    /// The height range `[min, max]` it applies in, each a number, bool or Molang expression
    /// compiled at Molang version 1 with the `world_gen` queries (JSON `height_range`).
    #[facet(schema::molang_with = super::height_range)]
    pub height_range: Option<[Molang; 2]>,
    /// The scale of the adjustment noise (JSON `noise_frequency_scale`).
    pub noise_frequency_scale: Option<f32>,
}

/// `adjustments[].materials`: every member optional.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AdjustmentMaterials {
    /// JSON `top_material`.
    pub top_material: Option<BlockSpecifier>,
    /// JSON `mid_material`.
    pub mid_material: Option<BlockSpecifier>,
    /// JSON `sea_floor_material`.
    pub sea_floor_material: Option<BlockSpecifier>,
    /// JSON `foundation_material`.
    pub foundation_material: Option<BlockSpecifier>,
    /// JSON `sea_material`.
    pub sea_material: Option<BlockSpecifier>,
}

// The builder is a data enum: its alternatives' members are checked by hand (`validate.rs`).
bedrock_addon_core::impl_schema_leaf!(Builder);
bedrock_addon_core::impl_no_molang!(Builder);

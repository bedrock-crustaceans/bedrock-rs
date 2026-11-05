//! `minecraft:jigsaw`: a jigsaw structure.

use super::validate::validate_detached;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{JigsawStructureId, TemplatePoolId};
use bedrock_addon_core::json::{StrictInt, StrictString};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::shared::Filter;

/// A jigsaw structure: a group of structure templates placed together during world generation,
/// starting from one template pool (`minecraft:jigsaw`, in `worldgen/structures/`).
///
/// JSON: `{"format_version": "1.21.20", "minecraft:jigsaw": {"description", "step",
/// "start_pool", "max_depth", "start_height", …}}`. Members with a vanilla default are `None`
/// when the document leaves them out; the accessor of the same name gives the default.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[doc(alias = "minecraft:jigsaw")]
#[doc(alias = "JigsawStructureDefinition")]
pub struct JigsawStructure {
    /// The structure's identifier (JSON `description`).
    pub description: StructureDescription,
    /// The biomes the structure may start in (JSON `biome_filters`); absent admits every biome.
    #[facet(default)]
    pub biome_filters: Option<Filter>,
    /// The world-generation phase the structure is generated in (JSON `step`).
    pub step: GenerationStep,
    /// How the terrain adapts to the structure (JSON `terrain_adaptation`, default `none`).
    #[facet(default)]
    pub terrain_adaptation: Option<TerrainAdaptation>,
    /// The first template pool generation draws from (JSON `start_pool`).
    pub start_pool: TemplatePoolId,
    /// The name of the jigsaw block of the start pool placed first (JSON `start_jigsaw_name`,
    /// default `""`).
    #[facet(default)]
    pub start_jigsaw_name: Option<StrictString>,
    /// The name of the callback the game runs after the structure is placed (JSON `on_placed`,
    /// default `""`: [`JigsawStructure::on_placed`]). Any name loads: the game reserves a
    /// token for it and binds callbacks in code.
    #[facet(default)]
    pub on_placed: Option<StrictString>,
    /// The maximum recursion depth, 0 to 20 (JSON `max_depth`).
    pub max_depth: StrictInt<u8>,
    /// The height the start pool begins at (JSON `start_height`).
    pub start_height: HeightProvider,
    /// The heightmap the start height is relative to (JSON `heightmap_projection`, default
    /// `none`).
    #[facet(default)]
    pub heightmap_projection: Option<HeightmapProjection>,
    /// Keeps the structure from being cut off at the top or bottom of the world (JSON
    /// `dimension_padding`; absent is no padding).
    #[facet(default)]
    pub dimension_padding: Option<DimensionPadding>,
    /// How far pieces may be from the start (JSON `max_distance_from_center`, default
    /// `{"horizontal": 80}` with no vertical limit).
    #[facet(default)]
    pub max_distance_from_center: Option<MaxDistanceFromCenter>,
    /// Which template pools may stand in for others (JSON `pool_aliases`, default `[]`).
    #[facet(default)]
    pub pool_aliases: Option<Vec<PoolAlias>>,
    /// What happens to waterloggable blocks that overlap liquid (JSON `liquid_settings`,
    /// default `apply_waterlogging`).
    #[facet(default)]
    pub liquid_settings: Option<LiquidSettings>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:jigsaw/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct StructureDescription {
    /// The structure's identifier, `namespace:name` (JSON `identifier`).
    pub identifier: JigsawStructureId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The world-generation phase a structure is generated in (`step`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum GenerationStep {
    /// JSON `raw_generation`.
    RawGeneration,
    /// JSON `lakes`.
    Lakes,
    /// JSON `local_modifications`.
    LocalModifications,
    /// JSON `underground_structures`.
    UndergroundStructures,
    /// JSON `surface_structures`.
    SurfaceStructures,
    /// JSON `strongholds`.
    Strongholds,
    /// JSON `underground_ores`.
    UndergroundOres,
    /// JSON `underground_decoration`.
    UndergroundDecoration,
    /// JSON `fluid_springs`.
    FluidSprings,
    /// JSON `vegetal_decoration`.
    VegetalDecoration,
    /// JSON `top_layer_modification`.
    TopLayerModification,
}

/// How the terrain adapts to a jigsaw structure (`terrain_adaptation`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum TerrainAdaptation {
    /// No adaptation (JSON `none`), the default.
    #[default]
    None,
    /// JSON `bury`.
    Bury,
    /// JSON `beard_thin`.
    BeardThin,
    /// JSON `beard_box`.
    BeardBox,
    /// JSON `encapsulate`.
    Encapsulate,
}

/// The heightmap a start height is relative to (`heightmap_projection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum HeightmapProjection {
    /// Absolute (JSON `none`), the default.
    #[default]
    None,
    /// JSON `world_surface`.
    WorldSurface,
    /// JSON `ocean_floor`.
    OceanFloor,
}

/// What happens to waterloggable blocks that overlap existing liquid (`liquid_settings`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LiquidSettings {
    /// They become waterlogged (JSON `apply_waterlogging`), the default.
    #[default]
    ApplyWaterlogging,
    /// They are left as they are (JSON `ignore_waterlogging`).
    IgnoreWaterlogging,
}

/// A height: one anchor, or uniformly between two (`start_height`).
///
/// JSON: `{"type": "constant", "value": anchor}` or `{"type": "uniform", "min": anchor, "max":
/// anchor}`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(tag = "type")]
#[repr(u8)]
pub enum HeightProvider {
    /// Uniformly between `min` and `max` (JSON `uniform`).
    #[facet(rename = "uniform")]
    Uniform {
        /// The lowest height (JSON `min`).
        min: VerticalAnchor,
        /// The highest height (JSON `max`).
        max: VerticalAnchor,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// Always `value` (JSON `constant`).
    #[facet(rename = "constant")]
    Constant {
        /// The height (JSON `value`).
        value: VerticalAnchor,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// A height as an offset from an anchor: an object with exactly one member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[repr(u8)]
pub enum VerticalAnchor {
    /// An absolute height (JSON `{"absolute": n}`).
    #[facet(rename = "absolute")]
    Absolute(StrictInt<i16>),
    /// Blocks above the bottom of the world (JSON `{"above_bottom": n}`).
    #[facet(rename = "above_bottom")]
    AboveBottom(StrictInt<i16>),
    /// Blocks below the top of the world (JSON `{"below_top": n}`).
    #[facet(rename = "below_top")]
    BelowTop(StrictInt<i16>),
    /// Blocks from sea level (JSON `{"from_sea": n}`).
    #[facet(rename = "from_sea")]
    FromSea(StrictInt<i16>),
}

/// Padding at the top and bottom of the world (`dimension_padding`): one number for both, or
/// `{"top", "bottom"}` (each default 0).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum DimensionPadding {
    /// The same padding at the top and the bottom (JSON a number).
    Both(u32),
    /// Separate padding (JSON `{"top"?, "bottom"?}`).
    Range {
        /// Blocks kept free at the top (JSON `top`, default 0).
        #[facet(default, skip_serializing_if = Option::is_none)]
        top: Option<StrictInt<u32>>,
        /// Blocks kept free at the bottom (JSON `bottom`, default 0).
        #[facet(default, skip_serializing_if = Option::is_none)]
        bottom: Option<StrictInt<u32>>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// How far a structure's pieces may be from its start (`max_distance_from_center`).
///
/// JSON: a number (the horizontal distance, 1 to 128) or `{"horizontal", "vertical"?}`
/// (`vertical` 1 or more, default no limit).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum MaxDistanceFromCenter {
    /// The horizontal distance only (JSON a number).
    Horizontal(i32),
    /// Both distances (JSON an object).
    Object {
        /// The distance on the horizontal plane, 1 to 128 (JSON `horizontal`).
        horizontal: StrictInt<i32>,
        /// The distance on the vertical axis (JSON `vertical`, default no limit).
        #[facet(default, skip_serializing_if = Option::is_none)]
        vertical: Option<StrictInt<i32>>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// A pool alias (`pool_aliases[]`): which template pool stands in for an alias name. Vanilla
/// binds no `type` member; the alternative is the first whose required members are present.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum PoolAlias {
    /// One pool (JSON `{"alias", "target"}`).
    Direct {
        /// The alias name (JSON `alias`).
        alias: StrictString,
        /// The pool it stands for (JSON `target`).
        target: TemplatePoolId,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A weighted choice of pools (JSON `{"alias", "targets"}`).
    Random {
        /// The alias name (JSON `alias`).
        alias: StrictString,
        /// The pools and their weights (JSON `targets`).
        targets: Vec<Weighted<TemplatePoolId>>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A weighted choice of alias groups (JSON `{"groups"}`).
    RandomGroup {
        /// The groups and their weights (JSON `groups`).
        groups: Vec<Weighted<Vec<GroupedPoolAlias>>>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// An alias inside a `random_group` alias: direct or random, never a group.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum GroupedPoolAlias {
    /// One pool (JSON `{"alias", "target"}`).
    Direct {
        /// The alias name (JSON `alias`).
        alias: StrictString,
        /// The pool it stands for (JSON `target`).
        target: TemplatePoolId,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A weighted choice of pools (JSON `{"alias", "targets"}`).
    Random {
        /// The alias name (JSON `alias`).
        alias: StrictString,
        /// The pools and their weights (JSON `targets`).
        targets: Vec<Weighted<TemplatePoolId>>,
        /// Unknown members.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
}

/// A weighted item (`{"weight", "data"}`), both members required.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Weighted<T> {
    /// The item's weight (JSON `weight`).
    pub weight: StrictInt<u32>,
    /// The item (JSON `data`).
    pub data: T,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The default horizontal distance from the start (`max_distance_from_center`).
const DEFAULT_HORIZONTAL: i32 = 80;

impl JigsawStructure {
    /// A structure with the required members and every other member left out (its default).
    pub fn new(
        identifier: JigsawStructureId,
        step: GenerationStep,
        start_pool: TemplatePoolId,
        max_depth: u8,
        start_height: HeightProvider,
    ) -> JigsawStructure {
        JigsawStructure {
            description: StructureDescription {
                identifier,
                extra: Extra::default(),
            },
            biome_filters: None,
            step,
            terrain_adaptation: None,
            start_pool,
            start_jigsaw_name: None,
            on_placed: None,
            max_depth: StrictInt(max_depth),
            start_height,
            heightmap_projection: None,
            dimension_padding: None,
            max_distance_from_center: None,
            pool_aliases: None,
            liquid_settings: None,
            extra: Extra::default(),
        }
    }

    /// The biome filter; absent is the empty group, which admits every biome.
    pub fn biome_filters(&self) -> Filter {
        self.biome_filters.clone().unwrap_or_default()
    }

    /// The terrain adaptation; `none` when absent.
    pub fn terrain_adaptation(&self) -> TerrainAdaptation {
        self.terrain_adaptation.unwrap_or_default()
    }

    /// The start jigsaw name; `""` when absent.
    pub fn start_jigsaw_name(&self) -> &str {
        self.start_jigsaw_name
            .as_ref()
            .map_or("", |name| name.0.as_str())
    }

    /// The `on_placed` callback name; `""` when absent (an empty name, never
    /// `minecraft:no_op`, which is a callback the game registers, not a default).
    pub fn on_placed(&self) -> &str {
        self.on_placed.as_ref().map_or("", |name| name.0.as_str())
    }

    /// The heightmap projection; `none` when absent.
    pub fn heightmap_projection(&self) -> HeightmapProjection {
        self.heightmap_projection.unwrap_or_default()
    }

    /// The padding at the top and the bottom of the world, `(top, bottom)`; `(0, 0)` when
    /// absent.
    pub fn dimension_padding(&self) -> (u32, u32) {
        match &self.dimension_padding {
            None => (0, 0),
            Some(DimensionPadding::Both(n)) => (*n, *n),
            Some(DimensionPadding::Range { top, bottom, .. }) => {
                (top.map_or(0, |n| n.0), bottom.map_or(0, |n| n.0))
            }
        }
    }

    /// The maximum distance from the start, `(horizontal, vertical)`; `(80, i32::MAX)` when
    /// absent, `vertical` `i32::MAX` (no limit) when not written.
    pub fn max_distance_from_center(&self) -> (i32, i32) {
        match &self.max_distance_from_center {
            None => (DEFAULT_HORIZONTAL, i32::MAX),
            Some(MaxDistanceFromCenter::Horizontal(h)) => (*h, i32::MAX),
            Some(MaxDistanceFromCenter::Object {
                horizontal,
                vertical,
                ..
            }) => (horizontal.0, vertical.map_or(i32::MAX, |v| v.0)),
        }
    }

    /// The pool aliases; none when absent.
    pub fn pool_aliases(&self) -> &[PoolAlias] {
        self.pool_aliases.as_deref().unwrap_or_default()
    }

    /// The liquid settings; `apply_waterlogging` when absent.
    pub fn liquid_settings(&self) -> LiquidSettings {
        self.liquid_settings.unwrap_or_default()
    }

    /// Runs the family's parse constraints and registration checks as if the structure were
    /// loaded from a behaviour pack with `cx`. The structure is not changed.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        validate_detached(self, cx)
    }
}

impl StructureDescription {
    /// A description with this identifier.
    pub fn new(identifier: JigsawStructureId) -> StructureDescription {
        StructureDescription {
            identifier,
            extra: Extra::default(),
        }
    }
}

//! The family's types: [`Feature`], one variant per feature type (the types live in the modules
//! of their kind), and the `description` they share, with its constructor.

use bedrock_addon_core::id::FeatureId;

use super::{
    AggregateFeature, BeardsAndShaversFeature, CaveCarverFeature, ConditionalListFeature,
    FossilFeature, GeodeFeature, GrowingPlantFeature, HeightDifferenceFilterFeature,
    HorizontalTreeDecorationFeature, MultiBlockFeature, MultifaceFeature,
    MultipartBlockColumnFeature, NetherCaveCarverFeature, OreFeature, PartiallyExposedBlobFeature,
    RectLayoutFeature, ScanSurfaceFeature, ScatterFeature, SculkPatchFeature, SearchFeature,
    SequenceFeature, SingleBlockFeature, SnapToSurfaceFeature, StructureTemplateFeature,
    SurfaceRelativeThresholdFeature, TreeFeature, UnderwaterCaveCarverFeature,
    VegetationPatchFeature, WeightedRandomFeature,
};

/// A feature: one placement rule of world generation, of one of vanilla's feature types, at the
/// newest format version of its type.
///
/// JSON: `{"format_version": "…", "minecraft:<type>_feature": {"description": {"identifier"},
/// …}}`; the payload key names the type. A document of an older format version is upgraded
/// when it loads (`history` has the older shapes); a document that breaks its type's schema in
/// any way is not loaded, as vanilla's strict schema rejects it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[doc(alias = "features")]
#[allow(
    clippy::large_enum_variant,
    reason = "the variants are the types callers match on; the one far larger than the rest is boxed"
)]
pub enum Feature {
    /// `minecraft:aggregate_feature`.
    Aggregate(AggregateFeature),
    /// `minecraft:beards_and_shavers` (base-game packs only).
    BeardsAndShavers(BeardsAndShaversFeature),
    /// `minecraft:cave_carver_feature`.
    CaveCarver(CaveCarverFeature),
    /// `minecraft:conditional_list`.
    ConditionalList(ConditionalListFeature),
    /// `minecraft:fossil_feature`.
    Fossil(FossilFeature),
    /// `minecraft:geode_feature`.
    Geode(GeodeFeature),
    /// `minecraft:growing_plant_feature`.
    GrowingPlant(GrowingPlantFeature),
    /// `minecraft:height_difference_filter_feature` (engine 1.26.30).
    HeightDifferenceFilter(HeightDifferenceFilterFeature),
    /// `minecraft:horizontal_tree_decoration_feature` (engine 1.26.40).
    HorizontalTreeDecoration(HorizontalTreeDecorationFeature),
    /// `minecraft:multi_block_feature` (engine 1.26.40).
    MultiBlock(MultiBlockFeature),
    /// `minecraft:multiface_feature`.
    Multiface(MultifaceFeature),
    /// `minecraft:multipart_block_column_feature` (format 1.26.40).
    MultipartBlockColumn(MultipartBlockColumnFeature),
    /// `minecraft:nether_cave_carver_feature`.
    NetherCaveCarver(NetherCaveCarverFeature),
    /// `minecraft:ore_feature`.
    Ore(OreFeature),
    /// `minecraft:partially_exposed_blob_feature`.
    PartiallyExposedBlob(PartiallyExposedBlobFeature),
    /// `minecraft:rect_layout` (base-game packs only).
    RectLayout(RectLayoutFeature),
    /// `minecraft:scan_surface` (base-game packs only).
    ScanSurface(ScanSurfaceFeature),
    /// `minecraft:scatter_feature`.
    Scatter(ScatterFeature),
    /// `minecraft:sculk_patch_feature` (base-game packs only).
    SculkPatch(SculkPatchFeature),
    /// `minecraft:search_feature`.
    Search(SearchFeature),
    /// `minecraft:sequence_feature`.
    Sequence(SequenceFeature),
    /// `minecraft:single_block_feature`.
    SingleBlock(SingleBlockFeature),
    /// `minecraft:snap_to_surface_feature`.
    SnapToSurface(SnapToSurfaceFeature),
    /// `minecraft:structure_template_feature`.
    StructureTemplate(StructureTemplateFeature),
    /// `minecraft:surface_relative_threshold_feature`.
    SurfaceRelativeThreshold(SurfaceRelativeThresholdFeature),
    /// `minecraft:tree_feature`. Boxed: a `TreeFeature` holds every trunk and canopy shape
    /// inline (some 17 KB, four times the next type), and an unoptimised build keeps a copy of
    /// a `Feature` in each frame that moves one; held inline it took a 1 MiB thread past its
    /// stack on the base game's load.
    Tree(Box<TreeFeature>),
    /// `minecraft:underwater_cave_carver_feature`.
    UnderwaterCaveCarver(UnderwaterCaveCarverFeature),
    /// `minecraft:vegetation_patch_feature`.
    VegetationPatch(VegetationPatchFeature),
    /// `minecraft:weighted_random_feature`.
    WeightedRandom(WeightedRandomFeature),
}

/// A feature's `description`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, facet::Facet)]
pub struct Description {
    /// The feature's identifier, `namespace:name` (JSON `identifier`). References to the
    /// feature use it; the file's name and folder do not matter.
    pub identifier: FeatureId,
}

impl Description {
    /// The description of the feature `identifier`.
    pub fn new(identifier: FeatureId) -> Self {
        Self { identifier }
    }
}

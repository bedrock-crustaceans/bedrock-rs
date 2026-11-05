//! Features: `features/**/*.json` of a behaviour pack and the app's `definitions/features`
//! (one struct per feature type, gathered in [`Feature`]). A document is `{"format_version",
//! "minecraft:<type>": {…}}`; each payload key is a feature type with its own schema and chain.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use blocks::{
    AttachConditions, ExposedFace, FossilFeature, GeodeFeature, GrowingPlantFeature,
    GrowthDirection, HeightWeight, HorizontalTreeDecorationFeature, MultiBlockFeature,
    MultifaceFeature, MultipartBlockColumnFeature, OreFeature, PartiallyExposedBlobFeature,
    PlacesBlock, ReplaceRule, SculkPatchFeature, SingleBlockFeature, VegetationPatchFeature,
    VegetationSurface, WeightedHeight, WeightedSpecifier,
};
pub use carver::{CaveCarverFeature, NetherCaveCarverFeature, UnderwaterCaveCarverFeature};
pub use placement::{
    AggregateFeature, ConditionalFeature, ConditionalListFeature, EarlyOut, EarlyOutScheme,
    FeatureArea, HeightDifferenceFilterFeature, RectLayoutFeature, ScanSurfaceFeature, SearchAxis,
    SearchFeature, SearchVolume, SequenceFeature, SurfaceRelativeThresholdFeature, WeightedFeature,
    WeightedRandomFeature,
};
pub use scatter::ScatterFeature;
pub use structure::{
    BeardsAndShaversFeature, BlockIntersection, EmptyConstraint, FacingDirection,
    LeveledConstraint, SnapToSurfaceFeature, StructureConstraints, StructureTemplateFeature,
    Surface,
};
pub use tree::{
    AboveRoot, AcaciaBranches, AcaciaCanopy, AcaciaTrunk, AcaciaTrunkHeight, AltitudeFactor,
    BaseCluster, BranchCanopy, CanopyOffset, CanopySlope, CherryBranches, CherryCanopy,
    CherryTrunk, CherryTrunkHeight, Decoration, DecorationBlock, FallenTrunk, FancyBranches,
    FancyCanopy, FancyTrunk, FancyTrunkHeight, MangroveBranches, MangroveCanopy, MangroveRoots,
    MangroveTrunk, MangroveTrunkHeight, MegaBranches, MegaCanopy, MegaPineCanopy, MegaTrunk,
    MegaTrunkHeight, PineCanopy, PoplarCanopy, PoplarTrunk, RandomSpreadCanopy, RoofedCanopy,
    SimpleCanopy, SimpleTrunk, SpruceCanopy, StepDirection, Submerged, TreeFeature,
    TreeTypeWeights, TrunkLean,
};
pub use values::{BlockList, CarverRange, Chance, ChanceList, IntToken, JsonInt, WeightedBlock};
#[doc(hidden)]
pub use walk::{Reference, references};

mod blocks;
mod carver;
mod load;
mod new;
mod parse;
mod placement;
mod scatter;
mod structure;
mod tree;
mod validate;
mod values;
mod walk;

#[cfg(test)]
mod tests;

pub use load::LOADER;

use bedrock_addon_core::id::FeatureId;

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

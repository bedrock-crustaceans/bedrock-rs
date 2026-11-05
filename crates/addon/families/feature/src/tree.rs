//! `minecraft:tree_feature`: a tree assembled from a trunk, a canopy and roots, each picked by
//! the member it is written under.

use facet::Facet;
use facet_value::VObject;

use super::Description;
use super::blocks::WeightedHeight;
use super::load::{Rules, V1_13_0, strict_struct};
use super::validate::Check;
use super::values::{BlockList, Chance, ChanceList, JsonInt, WeightedBlock};
use super::walk::walk;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, VanillaText};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{JsonType, StrictBool, StrictError, StrictFloat, Value};
use bedrock_addon_core::shared::{BlockDescriptor, IntRange};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::ValidateCx;

/// The engine that adds `decoration_blocks_sequence`.
const ENGINE_1_26_10: FormatVersion = known::V1_26_10;
/// The engine that adds the poplar parts. The gate is the crate's own: the client's builder
/// has none.
const ENGINE_1_26_40: FormatVersion = known::V1_26_40;

/// `minecraft:tree_feature`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TreeFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block(s) placed below the trunk (JSON `base_block`).
    #[facet(default)]
    pub base_block: Option<BlockList>,
    /// Clusters of the base block around the trunk (JSON `base_cluster`).
    #[facet(default)]
    pub base_cluster: Option<BaseCluster>,
    /// The blocks the tree may grow on (JSON `may_grow_on`).
    #[facet(default)]
    pub may_grow_on: Option<Vec<BlockDescriptor>>,
    /// The blocks the tree may replace (JSON `may_replace`).
    #[facet(default)]
    pub may_replace: Option<Vec<BlockDescriptor>>,
    /// The blocks the tree may grow through (JSON `may_grow_through`).
    #[facet(default)]
    pub may_grow_through: Option<Vec<BlockDescriptor>>,
    /// JSON `trunk`.
    #[facet(default)]
    pub trunk: Option<SimpleTrunk>,
    /// JSON `acacia_trunk`.
    #[facet(default)]
    pub acacia_trunk: Option<AcaciaTrunk>,
    /// JSON `cherry_trunk`.
    #[facet(default)]
    pub cherry_trunk: Option<CherryTrunk>,
    /// JSON `fallen_trunk`.
    #[facet(default)]
    pub fallen_trunk: Option<FallenTrunk>,
    /// JSON `fancy_trunk`.
    #[facet(default)]
    pub fancy_trunk: Option<FancyTrunk>,
    /// JSON `mangrove_trunk`.
    #[facet(default)]
    pub mangrove_trunk: Option<MangroveTrunk>,
    /// JSON `mega_trunk`.
    #[facet(default)]
    pub mega_trunk: Option<MegaTrunk>,
    /// JSON `poplar_trunk` (engine 1.26.40).
    #[facet(default)]
    pub poplar_trunk: Option<PoplarTrunk>,
    /// JSON `canopy`.
    #[facet(default)]
    pub canopy: Option<SimpleCanopy>,
    /// JSON `acacia_canopy`.
    #[facet(default)]
    pub acacia_canopy: Option<AcaciaCanopy>,
    /// JSON `cherry_canopy`.
    #[facet(default)]
    pub cherry_canopy: Option<CherryCanopy>,
    /// JSON `fancy_canopy`.
    #[facet(default)]
    pub fancy_canopy: Option<FancyCanopy>,
    /// JSON `mangrove_canopy`.
    #[facet(default)]
    pub mangrove_canopy: Option<MangroveCanopy>,
    /// JSON `mega_canopy`.
    #[facet(default)]
    pub mega_canopy: Option<MegaCanopy>,
    /// JSON `mega_pine_canopy`.
    #[facet(default)]
    pub mega_pine_canopy: Option<MegaPineCanopy>,
    /// JSON `pine_canopy`.
    #[facet(default)]
    pub pine_canopy: Option<PineCanopy>,
    /// JSON `roofed_canopy`.
    #[facet(default)]
    pub roofed_canopy: Option<RoofedCanopy>,
    /// JSON `spruce_canopy`.
    #[facet(default)]
    pub spruce_canopy: Option<SpruceCanopy>,
    /// JSON `random_spread_canopy`.
    #[facet(default)]
    pub random_spread_canopy: Option<RandomSpreadCanopy>,
    /// JSON `poplar_canopy` (engine 1.26.40).
    #[facet(default)]
    pub poplar_canopy: Option<PoplarCanopy>,
    /// JSON `mangrove_roots`.
    #[facet(default)]
    pub mangrove_roots: Option<MangroveRoots>,
}

/// `tree_feature.base_cluster`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BaseCluster {
    /// The blocks a cluster may replace (JSON `may_replace`).
    pub may_replace: Vec<BlockDescriptor>,
    /// How many clusters, at least 1 (JSON `num_clusters`).
    pub num_clusters: JsonInt,
    /// Their radius, at least 0 (JSON `cluster_radius`).
    pub cluster_radius: JsonInt,
}

/// A decoration attached to a trunk, canopy or roots (`trunk_decoration`, `canopy_decoration`,
/// `root_decoration`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Decoration {
    /// The chance of a decoration (JSON `decoration_chance`).
    pub decoration_chance: Chance,
    /// The blocks placed in sequence (JSON `decoration_blocks_sequence`, engine 1.26.10); when
    /// both are written it takes priority over `decoration_block`.
    #[facet(default)]
    pub decoration_blocks_sequence: Option<Vec<DecorationBlock>>,
    /// The block placed (JSON `decoration_block`).
    #[facet(default)]
    pub decoration_block: Option<BlockDescriptor>,
    /// How many blocks (JSON `num_steps`).
    #[facet(default)]
    pub num_steps: Option<JsonInt>,
    /// Which way the steps go (JSON `step_direction`).
    #[facet(default)]
    pub step_direction: Option<StepDirection>,
}

/// One entry of `decoration_blocks_sequence`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DecorationBlock {
    /// JSON `block`.
    pub block: BlockDescriptor,
    /// How many of it (JSON `count`).
    #[facet(default)]
    pub count: Option<IntRange>,
}

/// `step_direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum StepDirection {
    /// `down`.
    Down,
    /// `up`.
    Up,
    /// `out`.
    Out,
    /// `away`: the same direction as `out`, kept as written.
    Away,
}

// ---- trunks ------------------------------------------------------------------------------------

/// `trunk`: a straight trunk.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SimpleTrunk {
    /// At least 1 (JSON `trunk_height`).
    pub trunk_height: IntRange,
    /// JSON `height_modifier`.
    #[facet(default)]
    pub height_modifier: Option<IntRange>,
    /// Whether, or how deep, the trunk may be under water (JSON `can_be_submerged`).
    #[facet(default)]
    pub can_be_submerged: Option<Submerged>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
}

/// `trunk.can_be_submerged`.
///
/// JSON: a boolean, or `{"max_depth": int}` (at least 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum Submerged {
    /// A boolean.
    Allowed(bool),
    /// `{"max_depth"}`: under water down to this depth.
    MaxDepth(i32),
}

impl TryFrom<Value> for Submerged {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        if let Some(b) = value.as_bool() {
            return Ok(Submerged::Allowed(b));
        }
        let depth = value
            .as_object()
            .filter(|object| object.len() == 1)
            .and_then(|object| object.get("max_depth"))
            .and_then(Value::as_number)
            .and_then(facet_value::VNumber::to_i64)
            .and_then(|depth| i32::try_from(depth).ok());
        depth
            .map(Submerged::MaxDepth)
            .ok_or(StrictError::WrongType {
                expected: "a boolean or {\"max_depth\"}",
                found: JsonType::Null,
            })
    }
}

impl From<&Submerged> for Value {
    fn from(submerged: &Submerged) -> Self {
        match *submerged {
            Submerged::Allowed(b) => Value::from(b),
            Submerged::MaxDepth(depth) => {
                let mut object = VObject::new();
                object.insert("max_depth", Value::from(i64::from(depth)));
                Value::from(object)
            }
        }
    }
}

/// `fallen_trunk`: a log lying beside a stump.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FallenTrunk {
    /// At least 1 (JSON `log_length`).
    pub log_length: IntRange,
    /// At least 1 (JSON `stump_height`).
    #[facet(default)]
    pub stump_height: Option<IntRange>,
    /// JSON `height_modifier`.
    #[facet(default)]
    pub height_modifier: Option<IntRange>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// The feature placed on the log (JSON `log_decoration_feature`).
    #[facet(default)]
    pub log_decoration_feature: Option<FeatureId>,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
}

/// `fancy_trunk`: a large oak trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FancyTrunk {
    /// JSON `trunk_height` (its members required when present).
    #[facet(default)]
    pub trunk_height: Option<FancyTrunkHeight>,
    /// At least 1 (JSON `trunk_width`).
    pub trunk_width: JsonInt,
    /// JSON `branches` (its members required when present).
    #[facet(default)]
    pub branches: Option<FancyBranches>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// At least 0 (JSON `width_scale`).
    pub width_scale: StrictFloat<f32>,
    /// In `[0, 1]` (JSON `foliage_altitude_factor`).
    pub foliage_altitude_factor: StrictFloat<f32>,
}

/// `fancy_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FancyTrunkHeight {
    /// At least 1 (JSON `base`).
    pub base: JsonInt,
    /// At least 1 (JSON `variance`).
    pub variance: JsonInt,
    /// JSON `scale`.
    pub scale: StrictFloat<f32>,
}

/// `fancy_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FancyBranches {
    /// JSON `slope`.
    pub slope: StrictFloat<f32>,
    /// JSON `density`.
    pub density: StrictFloat<f32>,
    /// In `[0, 1]` (JSON `min_altitude_factor`).
    pub min_altitude_factor: StrictFloat<f32>,
}

/// `mega_trunk`: a wide trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MegaTrunk {
    /// JSON `trunk_width`.
    pub trunk_width: JsonInt,
    /// JSON `trunk_height`.
    pub trunk_height: MegaTrunkHeight,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
    /// JSON `branches`.
    #[facet(default)]
    pub branches: Option<MegaBranches>,
}

/// `mega_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MegaTrunkHeight {
    /// At least 1 (JSON `base`).
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(default)]
    pub intervals: Option<Vec<JsonInt>>,
}

/// `mega_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MegaBranches {
    /// At least 1 (JSON `branch_length`).
    pub branch_length: JsonInt,
    /// JSON `branch_slope`.
    pub branch_slope: StrictFloat<f32>,
    /// At least 0 (JSON `branch_interval`).
    pub branch_interval: IntRange,
    /// JSON `branch_altitude_factor`.
    pub branch_altitude_factor: AltitudeFactor,
    /// JSON `branch_canopy`.
    #[facet(default)]
    pub branch_canopy: Option<BranchCanopy>,
}

/// `mega_trunk.branches.branch_altitude_factor`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AltitudeFactor {
    /// In `[0, 1]` (JSON `min`).
    pub min: StrictFloat<f32>,
    /// In `[0, 1]` (JSON `max`).
    pub max: StrictFloat<f32>,
}

/// `acacia_trunk`: a leaning trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AcaciaTrunk {
    /// JSON `trunk_width`.
    pub trunk_width: JsonInt,
    /// JSON `trunk_height`.
    pub trunk_height: AcaciaTrunkHeight,
    /// JSON `trunk_lean`.
    pub trunk_lean: TrunkLean,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `branches`.
    #[facet(default)]
    pub branches: Option<AcaciaBranches>,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
}

/// `acacia_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AcaciaTrunkHeight {
    /// At least 1 (JSON `base`).
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(default)]
    pub intervals: Option<Vec<JsonInt>>,
    /// At least 1 (JSON `min_height_for_canopy`).
    #[facet(default)]
    pub min_height_for_canopy: Option<JsonInt>,
}

/// `acacia_trunk.trunk_lean`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TrunkLean {
    /// JSON `allow_diagonal_growth`.
    pub allow_diagonal_growth: StrictBool,
    /// At least 0 (JSON `lean_height`).
    pub lean_height: IntRange,
    /// At least 0 (JSON `lean_steps`).
    pub lean_steps: IntRange,
    /// At least 0 (JSON `lean_length`).
    #[facet(default)]
    pub lean_length: Option<IntRange>,
}

/// `acacia_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AcaciaBranches {
    /// At least 1 (JSON `branch_length`).
    pub branch_length: IntRange,
    /// At least 1 (JSON `branch_position`).
    pub branch_position: IntRange,
    /// JSON `branch_chance`.
    pub branch_chance: Chance,
    /// JSON `branch_canopy`.
    #[facet(default)]
    pub branch_canopy: Option<BranchCanopy>,
}

/// `cherry_trunk`: a trunk that splits into branches.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CherryTrunk {
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_height`.
    pub trunk_height: CherryTrunkHeight,
    /// JSON `branches`.
    pub branches: CherryBranches,
}

/// `cherry_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CherryTrunkHeight {
    /// At least 2 (JSON `base`).
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(default)]
    pub intervals: Option<Vec<JsonInt>>,
}

/// `cherry_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CherryBranches {
    /// JSON `tree_type_weights`.
    #[facet(default)]
    pub tree_type_weights: Option<TreeTypeWeights>,
    /// At least 2 (JSON `branch_horizontal_length`).
    pub branch_horizontal_length: IntRange,
    /// At most 0, with at least two values (JSON `branch_start_offset_from_top`).
    pub branch_start_offset_from_top: IntRange,
    /// JSON `branch_end_offset_from_top`.
    pub branch_end_offset_from_top: IntRange,
    /// JSON `branch_canopy`.
    #[facet(default)]
    pub branch_canopy: Option<BranchCanopy>,
}

/// `cherry_trunk.branches.tree_type_weights`: each at least 0.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TreeTypeWeights {
    /// JSON `one_branch`.
    pub one_branch: JsonInt,
    /// JSON `two_branches`.
    pub two_branches: JsonInt,
    /// JSON `two_branches_and_trunk`.
    pub two_branches_and_trunk: JsonInt,
}

/// `mangrove_trunk`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MangroveTrunk {
    /// JSON `trunk_width`.
    pub trunk_width: JsonInt,
    /// JSON `trunk_height` (its members required when present).
    #[facet(default)]
    pub trunk_height: Option<MangroveTrunkHeight>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `branches`.
    #[facet(default)]
    pub branches: Option<MangroveBranches>,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
}

/// `mangrove_trunk.trunk_height`: each at least 1.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MangroveTrunkHeight {
    /// JSON `base`.
    pub base: JsonInt,
    /// JSON `height_rand_a`.
    pub height_rand_a: JsonInt,
    /// JSON `height_rand_b`.
    pub height_rand_b: JsonInt,
}

/// `mangrove_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MangroveBranches {
    /// At least 0 (JSON `branch_length`).
    pub branch_length: IntRange,
    /// At least 0 (JSON `branch_steps`).
    pub branch_steps: IntRange,
    /// JSON `branch_chance`.
    pub branch_chance: Chance,
}

/// `poplar_trunk` (engine 1.26.40).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PoplarTrunk {
    /// At least 1 (JSON `trunk_height`).
    pub trunk_height: IntRange,
    /// JSON `remaining_trunk_height_above_branches`.
    #[facet(default)]
    pub remaining_trunk_height_above_branches: Option<IntRange>,
    /// JSON `amount_of_foliage_support_branches`.
    #[facet(default)]
    pub amount_of_foliage_support_branches: Option<IntRange>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
    #[facet(default)]
    pub trunk_decoration: Option<Decoration>,
    /// JSON `log_decoration_feature`.
    #[facet(default)]
    pub log_decoration_feature: Option<FeatureId>,
}

// ---- canopies ----------------------------------------------------------------------------------

/// The canopy a branch ends in (`branches.branch_canopy`): one of the canopies, except the
/// random-spread and poplar ones.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct BranchCanopy {
    /// JSON `canopy`.
    #[facet(default)]
    pub canopy: Option<SimpleCanopy>,
    /// JSON `acacia_canopy`.
    #[facet(default)]
    pub acacia_canopy: Option<AcaciaCanopy>,
    /// JSON `cherry_canopy`.
    #[facet(default)]
    pub cherry_canopy: Option<CherryCanopy>,
    /// JSON `fancy_canopy`.
    #[facet(default)]
    pub fancy_canopy: Option<FancyCanopy>,
    /// JSON `mangrove_canopy`.
    #[facet(default)]
    pub mangrove_canopy: Option<MangroveCanopy>,
    /// JSON `mega_canopy`.
    #[facet(default)]
    pub mega_canopy: Option<MegaCanopy>,
    /// JSON `mega_pine_canopy`.
    #[facet(default)]
    pub mega_pine_canopy: Option<MegaPineCanopy>,
    /// JSON `pine_canopy`.
    #[facet(default)]
    pub pine_canopy: Option<PineCanopy>,
    /// JSON `roofed_canopy`.
    #[facet(default)]
    pub roofed_canopy: Option<RoofedCanopy>,
    /// JSON `spruce_canopy`.
    #[facet(default)]
    pub spruce_canopy: Option<SpruceCanopy>,
}

/// `canopy`: a layered canopy.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SimpleCanopy {
    /// JSON `canopy_offset`.
    pub canopy_offset: CanopyOffset,
    /// At least 0 (JSON `min_width`).
    #[facet(default)]
    pub min_width: Option<JsonInt>,
    /// JSON `canopy_slope`.
    #[facet(default)]
    pub canopy_slope: Option<CanopySlope>,
    /// JSON `variation_chance`.
    #[facet(default)]
    pub variation_chance: Option<ChanceList>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `canopy_decoration`.
    #[facet(default)]
    pub canopy_decoration: Option<Decoration>,
}

/// `canopy.canopy_offset`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CanopyOffset {
    /// JSON `min`, at most `max`.
    pub min: JsonInt,
    /// JSON `max`.
    pub max: JsonInt,
}

/// `canopy.canopy_slope`: each at least 1.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CanopySlope {
    /// JSON `rise`.
    #[facet(default)]
    pub rise: Option<JsonInt>,
    /// JSON `run`.
    #[facet(default)]
    pub run: Option<JsonInt>,
}

/// `acacia_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AcaciaCanopy {
    /// At least 1 (JSON `canopy_size`).
    pub canopy_size: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `simplify_canopy`.
    #[facet(default)]
    pub simplify_canopy: Option<StrictBool>,
}

/// `cherry_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CherryCanopy {
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// At least 4 (JSON `height`).
    pub height: IntRange,
    /// At least 3 (JSON `radius`).
    pub radius: IntRange,
    /// At least 1 (JSON `trunk_width`).
    #[facet(default)]
    pub trunk_width: Option<JsonInt>,
    /// JSON `wide_bottom_layer_hole_chance`.
    pub wide_bottom_layer_hole_chance: Chance,
    /// JSON `corner_hole_chance`.
    pub corner_hole_chance: Chance,
    /// JSON `hanging_leaves_chance`.
    pub hanging_leaves_chance: Chance,
    /// JSON `hanging_leaves_extension_chance`.
    pub hanging_leaves_extension_chance: Chance,
}

/// `fancy_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FancyCanopy {
    /// At least 1 (JSON `height`).
    pub height: JsonInt,
    /// At least 0 (JSON `radius`).
    pub radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `mangrove_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MangroveCanopy {
    /// At least 1 (JSON `canopy_height`).
    pub canopy_height: IntRange,
    /// At least 1 (JSON `canopy_radius`).
    pub canopy_radius: IntRange,
    /// At least 1 (JSON `leaf_placement_attempts`).
    pub leaf_placement_attempts: JsonInt,
    /// JSON `leaf_blocks`.
    pub leaf_blocks: Vec<WeightedBlock>,
    /// JSON `canopy_decoration`.
    #[facet(default)]
    pub canopy_decoration: Option<Decoration>,
    /// JSON `hanging_block`.
    pub hanging_block: BlockDescriptor,
    /// JSON `hanging_block_placement_chance`.
    pub hanging_block_placement_chance: Chance,
}

/// `mega_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MegaCanopy {
    /// At least 1 (JSON `canopy_height`).
    pub canopy_height: IntRange,
    /// At least 0 (JSON `base_radius`).
    pub base_radius: JsonInt,
    /// At least 1 (JSON `core_width`).
    #[facet(default)]
    pub core_width: Option<JsonInt>,
    /// JSON `simplify_canopy`.
    #[facet(default)]
    pub simplify_canopy: Option<StrictBool>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `mega_pine_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MegaPineCanopy {
    /// At least 1 (JSON `canopy_height`).
    pub canopy_height: IntRange,
    /// At least 0 (JSON `base_radius`).
    pub base_radius: JsonInt,
    /// At least 0 (JSON `radius_step_modifier`).
    pub radius_step_modifier: StrictFloat<f32>,
    /// At least 1 (JSON `core_width`).
    #[facet(default)]
    pub core_width: Option<JsonInt>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `pine_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PineCanopy {
    /// At least 1 (JSON `canopy_height`).
    pub canopy_height: IntRange,
    /// At least 1 (JSON `base_radius`).
    pub base_radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `roofed_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RoofedCanopy {
    /// At least 3 (JSON `canopy_height`).
    pub canopy_height: JsonInt,
    /// At least 1 (JSON `core_width`).
    pub core_width: JsonInt,
    /// At least 0 (JSON `outer_radius`).
    pub outer_radius: JsonInt,
    /// At least 0 (JSON `inner_radius`).
    pub inner_radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `spruce_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SpruceCanopy {
    /// At least 1 (JSON `lower_offset`).
    pub lower_offset: IntRange,
    /// At least 0 (JSON `upper_offset`).
    pub upper_offset: IntRange,
    /// At least 1 (JSON `max_radius`).
    pub max_radius: IntRange,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `random_spread_canopy`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RandomSpreadCanopy {
    /// At least 1 (JSON `canopy_height`).
    pub canopy_height: IntRange,
    /// At least 1 (JSON `canopy_radius`).
    pub canopy_radius: IntRange,
    /// At least 1 (JSON `leaf_placement_attempts`).
    pub leaf_placement_attempts: JsonInt,
    /// JSON `leaf_blocks`.
    pub leaf_blocks: Vec<WeightedBlock>,
}

/// `poplar_canopy` (engine 1.26.40).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PoplarCanopy {
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `branch_block`.
    pub branch_block: BlockDescriptor,
    /// Radii and their weights, at least one (JSON `radius`).
    pub radius: Vec<WeightedHeight>,
    /// At least 4 (JSON `height`).
    pub height: IntRange,
    /// In `[0, 1]` (JSON `side_hole_chance`).
    #[facet(default)]
    pub side_hole_chance: Option<StrictFloat<f32>>,
    /// At least 1 (JSON `trunk_width`).
    #[facet(default)]
    pub trunk_width: Option<JsonInt>,
}

// ---- roots -------------------------------------------------------------------------------------

/// `mangrove_roots`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MangroveRoots {
    /// At least 1 (JSON `max_root_width`).
    pub max_root_width: JsonInt,
    /// At least 1 (JSON `max_root_length`).
    pub max_root_length: JsonInt,
    /// JSON `root_block`.
    pub root_block: BlockDescriptor,
    /// JSON `above_root`.
    #[facet(default)]
    pub above_root: Option<AboveRoot>,
    /// JSON `muddy_root_block`.
    pub muddy_root_block: BlockDescriptor,
    /// JSON `mud_block`.
    pub mud_block: BlockDescriptor,
    /// At least 1 (JSON `y_offset`).
    pub y_offset: IntRange,
    /// JSON `roots_may_grow_through`.
    pub roots_may_grow_through: Vec<BlockDescriptor>,
    /// JSON `root_decoration`.
    #[facet(default)]
    pub root_decoration: Option<Decoration>,
}

/// `mangrove_roots.above_root`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AboveRoot {
    /// JSON `above_root_chance`.
    #[facet(default)]
    pub above_root_chance: Option<Chance>,
    /// JSON `above_root_block`.
    #[facet(default)]
    pub above_root_block: Option<BlockDescriptor>,
}

// ---- version struct, Molang walk ---------------------------------------------------------------

strict_struct!(TreeFeature, V1_13_0);

walk!(fields TreeFeature {
    "base_block" => base_block,
    "base_cluster" => base_cluster,
    "may_grow_on" => may_grow_on,
    "may_replace" => may_replace,
    "may_grow_through" => may_grow_through,
    "trunk" => trunk,
    "acacia_trunk" => acacia_trunk,
    "cherry_trunk" => cherry_trunk,
    "fallen_trunk" => fallen_trunk,
    "fancy_trunk" => fancy_trunk,
    "mangrove_trunk" => mangrove_trunk,
    "mega_trunk" => mega_trunk,
    "poplar_trunk" => poplar_trunk,
    "canopy" => canopy,
    "acacia_canopy" => acacia_canopy,
    "cherry_canopy" => cherry_canopy,
    "fancy_canopy" => fancy_canopy,
    "mangrove_canopy" => mangrove_canopy,
    "mega_canopy" => mega_canopy,
    "mega_pine_canopy" => mega_pine_canopy,
    "pine_canopy" => pine_canopy,
    "roofed_canopy" => roofed_canopy,
    "spruce_canopy" => spruce_canopy,
    "random_spread_canopy" => random_spread_canopy,
    "poplar_canopy" => poplar_canopy,
    "mangrove_roots" => mangrove_roots,
});
walk!(BaseCluster { "may_replace" => may_replace });
walk!(Decoration {
    "decoration_blocks_sequence" => decoration_blocks_sequence,
    "decoration_block" => decoration_block,
});
walk!(DecorationBlock { "block" => block });
walk!(SimpleTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration });
// The list after `+`: the member that names a feature.
walk!(FallenTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration } + {
    "log_decoration_feature" => log_decoration_feature,
});
walk!(FancyTrunk { "trunk_block" => trunk_block });
walk!(MegaTrunk {
    "trunk_block" => trunk_block,
    "trunk_decoration" => trunk_decoration,
    "branches" => branches,
});
walk!(MegaBranches { "branch_canopy" => branch_canopy });
walk!(AcaciaTrunk {
    "trunk_block" => trunk_block,
    "branches" => branches,
    "trunk_decoration" => trunk_decoration,
});
walk!(AcaciaBranches { "branch_canopy" => branch_canopy });
walk!(CherryTrunk { "trunk_block" => trunk_block, "branches" => branches });
walk!(CherryBranches { "branch_canopy" => branch_canopy });
walk!(MangroveTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration });
// The list after `+`: the member that names a feature.
walk!(PoplarTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration } + {
    "log_decoration_feature" => log_decoration_feature,
});
walk!(BranchCanopy {
    "canopy" => canopy,
    "acacia_canopy" => acacia_canopy,
    "cherry_canopy" => cherry_canopy,
    "fancy_canopy" => fancy_canopy,
    "mangrove_canopy" => mangrove_canopy,
    "mega_canopy" => mega_canopy,
    "mega_pine_canopy" => mega_pine_canopy,
    "pine_canopy" => pine_canopy,
    "roofed_canopy" => roofed_canopy,
    "spruce_canopy" => spruce_canopy,
});
walk!(SimpleCanopy { "leaf_block" => leaf_block, "canopy_decoration" => canopy_decoration });
walk!(AcaciaCanopy { "leaf_block" => leaf_block });
walk!(CherryCanopy { "leaf_block" => leaf_block });
walk!(FancyCanopy { "leaf_block" => leaf_block });
walk!(MangroveCanopy {
    "leaf_blocks" => leaf_blocks,
    "canopy_decoration" => canopy_decoration,
    "hanging_block" => hanging_block,
});
walk!(MegaCanopy { "leaf_block" => leaf_block });
walk!(MegaPineCanopy { "leaf_block" => leaf_block });
walk!(PineCanopy { "leaf_block" => leaf_block });
walk!(RoofedCanopy { "leaf_block" => leaf_block });
walk!(SpruceCanopy { "leaf_block" => leaf_block });
walk!(RandomSpreadCanopy { "leaf_blocks" => leaf_blocks });
walk!(PoplarCanopy { "leaf_block" => leaf_block, "branch_block" => branch_block });
walk!(MangroveRoots {
    "root_block" => root_block,
    "above_root" => above_root,
    "muddy_root_block" => muddy_root_block,
    "mud_block" => mud_block,
    "roots_may_grow_through" => roots_may_grow_through,
    "root_decoration" => root_decoration,
});
walk!(AboveRoot { "above_root_block" => above_root_block });

// ---- rules -------------------------------------------------------------------------------------

/// The trunk setter's text for a negative `height_modifier` larger than the trunk height.
const INVERTED_TRUNK: &str = "A negative 'height_modifier' cannot have greater magnitude than 'trunk_height:range_min' (trunk would invert!)";
/// The cherry trunk setter's text for a start offset of one value.
const CHERRY_START_OFFSET: &str = "branch_start_offset_from_top range must contain at least 2 values because branches must start at different heights.";
/// The simple canopy setter's text for `canopy_offset.min > max`.
const CANOPY_OFFSET: &str = "'min' must be <= 'max'";

/// A setter's message as an `InvalidValue` kept error: the client logs it at level 3 and keeps
/// the value (for the trunk, the cherry start offset and each canopy offset).
fn setter(check: &mut ValidateCx<'_>, key: &str, text: &'static str) {
    check.kept(
        key,
        DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Fixed(text)),
        },
    );
}

fn decoration(check: &mut ValidateCx<'_>, decoration: &Decoration) {
    check.chance("decoration_chance", Some(&decoration.decoration_chance));
    check.since_engine(
        "decoration_blocks_sequence",
        decoration.decoration_blocks_sequence.is_some(),
        ENGINE_1_26_10,
    );
    if let Some(sequence) = &decoration.decoration_blocks_sequence {
        check.each("decoration_blocks_sequence", sequence, |check, entry| {
            check.descriptor("block", Some(&entry.block));
            check.range("count", entry.count.as_ref(), i32::MIN, i32::MAX);
        });
    }
    check.descriptor("decoration_block", decoration.decoration_block.as_ref());
}

fn height_modifier(check: &mut ValidateCx<'_>, height: &IntRange, modifier: Option<&IntRange>) {
    check.range("height_modifier", modifier, i32::MIN, i32::MAX);
    if let Some(modifier) = modifier
        && modifier.min < 0
        && modifier.min.unsigned_abs() > height.min.unsigned_abs()
    {
        setter(check, "height_modifier", INVERTED_TRUNK);
    }
}

fn intervals(check: &mut ValidateCx<'_>, values: Option<&Vec<JsonInt>>) {
    if let Some(values) = values {
        check.each("intervals", values, |check, value| {
            // The element's own path: `check.at` ends at its index.
            if value.0 < 1 {
                let at = check.at.clone();
                check.diagnostics.push(
                    Diagnostic::error(DiagnosticCode::InvalidValue {
                        text: Some(super::parse::formatted(
                            super::parse::INT_OUT_OF_RANGE,
                            &[&value.0, &1, &i32::MAX],
                        )),
                    })
                    .at(at)
                    .phase(bedrock_addon_core::diagnostic::Phase::Validate),
                );
            }
        });
    }
}

fn simple_trunk(check: &mut ValidateCx<'_>, trunk: &SimpleTrunk) {
    check.range_min("trunk_height", Some(&trunk.trunk_height), 1);
    height_modifier(check, &trunk.trunk_height, trunk.height_modifier.as_ref());
    if let Some(Submerged::MaxDepth(depth)) = trunk.can_be_submerged {
        check.nested("can_be_submerged", |check| {
            check.int_min("max_depth", Some(JsonInt::new(depth)), 1);
        });
    }
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
}

fn fallen_trunk(check: &mut ValidateCx<'_>, trunk: &FallenTrunk) {
    check.range_min("log_length", Some(&trunk.log_length), 1);
    check.range_min("stump_height", trunk.stump_height.as_ref(), 1);
    height_modifier(check, &trunk.log_length, trunk.height_modifier.as_ref());
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
}

fn fancy_trunk(check: &mut ValidateCx<'_>, trunk: &FancyTrunk) {
    check.opt(
        "trunk_height",
        trunk.trunk_height.as_ref(),
        |check, height| {
            check.int_min("base", Some(height.base), 1);
            check.int_min("variance", Some(height.variance), 1);
        },
    );
    check.int_min("trunk_width", Some(trunk.trunk_width), 1);
    check.opt("branches", trunk.branches.as_ref(), |check, branches| {
        check.check_float(
            "min_altitude_factor",
            Some(branches.min_altitude_factor),
            0.0,
            1.0,
        );
    });
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.check_float("width_scale", Some(trunk.width_scale), 0.0, f32::MAX);
    check.check_float(
        "foliage_altitude_factor",
        Some(trunk.foliage_altitude_factor),
        0.0,
        1.0,
    );
}

fn mega_trunk(check: &mut ValidateCx<'_>, trunk: &MegaTrunk) {
    check.nested("trunk_height", |check| {
        check.int_min("base", Some(trunk.trunk_height.base), 1);
        intervals(check, trunk.trunk_height.intervals.as_ref());
    });
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
    check.opt("branches", trunk.branches.as_ref(), |check, branches| {
        check.int_min("branch_length", Some(branches.branch_length), 1);
        check.range_min("branch_interval", Some(&branches.branch_interval), 0);
        check.nested("branch_altitude_factor", |check| {
            let factor = &branches.branch_altitude_factor;
            check.check_float("min", Some(factor.min), 0.0, 1.0);
            check.check_float("max", Some(factor.max), 0.0, 1.0);
        });
        check.opt(
            "branch_canopy",
            branches.branch_canopy.as_ref(),
            branch_canopy,
        );
    });
}

fn acacia_trunk(check: &mut ValidateCx<'_>, trunk: &AcaciaTrunk) {
    check.nested("trunk_height", |check| {
        let height = &trunk.trunk_height;
        check.int_min("base", Some(height.base), 1);
        intervals(check, height.intervals.as_ref());
        check.int_min("min_height_for_canopy", height.min_height_for_canopy, 1);
    });
    check.nested("trunk_lean", |check| {
        let lean = &trunk.trunk_lean;
        check.range_min("lean_height", Some(&lean.lean_height), 0);
        check.range_min("lean_steps", Some(&lean.lean_steps), 0);
        check.range_min("lean_length", lean.lean_length.as_ref(), 0);
    });
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt("branches", trunk.branches.as_ref(), |check, branches| {
        check.range_min("branch_length", Some(&branches.branch_length), 1);
        check.range_min("branch_position", Some(&branches.branch_position), 1);
        check.chance("branch_chance", Some(&branches.branch_chance));
        check.opt(
            "branch_canopy",
            branches.branch_canopy.as_ref(),
            branch_canopy,
        );
    });
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
}

fn cherry_trunk(check: &mut ValidateCx<'_>, trunk: &CherryTrunk) {
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.nested("trunk_height", |check| {
        check.int_min("base", Some(trunk.trunk_height.base), 2);
        intervals(check, trunk.trunk_height.intervals.as_ref());
    });
    check.nested("branches", |check| {
        let branches = &trunk.branches;
        check.opt(
            "tree_type_weights",
            branches.tree_type_weights.as_ref(),
            |check, weights| {
                check.int_min("one_branch", Some(weights.one_branch), 0);
                check.int_min("two_branches", Some(weights.two_branches), 0);
                check.int_min(
                    "two_branches_and_trunk",
                    Some(weights.two_branches_and_trunk),
                    0,
                );
            },
        );
        check.range_min(
            "branch_horizontal_length",
            Some(&branches.branch_horizontal_length),
            2,
        );
        let start = &branches.branch_start_offset_from_top;
        check.range("branch_start_offset_from_top", Some(start), i32::MIN, 0);
        if start.min == start.max {
            setter(check, "branch_start_offset_from_top", CHERRY_START_OFFSET);
        }
        check.range(
            "branch_end_offset_from_top",
            Some(&branches.branch_end_offset_from_top),
            i32::MIN,
            i32::MAX,
        );
        check.opt(
            "branch_canopy",
            branches.branch_canopy.as_ref(),
            branch_canopy,
        );
    });
}

fn mangrove_trunk(check: &mut ValidateCx<'_>, trunk: &MangroveTrunk) {
    check.opt(
        "trunk_height",
        trunk.trunk_height.as_ref(),
        |check, height| {
            check.int_min("base", Some(height.base), 1);
            check.int_min("height_rand_a", Some(height.height_rand_a), 1);
            check.int_min("height_rand_b", Some(height.height_rand_b), 1);
        },
    );
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt("branches", trunk.branches.as_ref(), |check, branches| {
        check.range_min("branch_length", Some(&branches.branch_length), 0);
        check.range_min("branch_steps", Some(&branches.branch_steps), 0);
        check.chance("branch_chance", Some(&branches.branch_chance));
    });
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
}

fn poplar_trunk(check: &mut ValidateCx<'_>, trunk: &PoplarTrunk) {
    check.range_min("trunk_height", Some(&trunk.trunk_height), 1);
    for (key, range) in [
        (
            "remaining_trunk_height_above_branches",
            &trunk.remaining_trunk_height_above_branches,
        ),
        (
            "amount_of_foliage_support_branches",
            &trunk.amount_of_foliage_support_branches,
        ),
    ] {
        check.range(key, range.as_ref(), i32::MIN, i32::MAX);
    }
    check.descriptor("trunk_block", Some(&trunk.trunk_block));
    check.opt(
        "trunk_decoration",
        trunk.trunk_decoration.as_ref(),
        decoration,
    );
}

fn branch_canopy(check: &mut ValidateCx<'_>, canopy: &BranchCanopy) {
    check.opt("canopy", canopy.canopy.as_ref(), simple_canopy);
    check.opt(
        "acacia_canopy",
        canopy.acacia_canopy.as_ref(),
        acacia_canopy,
    );
    check.opt(
        "cherry_canopy",
        canopy.cherry_canopy.as_ref(),
        cherry_canopy,
    );
    check.opt("fancy_canopy", canopy.fancy_canopy.as_ref(), fancy_canopy);
    check.opt(
        "mangrove_canopy",
        canopy.mangrove_canopy.as_ref(),
        mangrove_canopy,
    );
    check.opt("mega_canopy", canopy.mega_canopy.as_ref(), mega_canopy);
    check.opt(
        "mega_pine_canopy",
        canopy.mega_pine_canopy.as_ref(),
        mega_pine_canopy,
    );
    check.opt("pine_canopy", canopy.pine_canopy.as_ref(), pine_canopy);
    check.opt(
        "roofed_canopy",
        canopy.roofed_canopy.as_ref(),
        roofed_canopy,
    );
    check.opt(
        "spruce_canopy",
        canopy.spruce_canopy.as_ref(),
        spruce_canopy,
    );
}

fn simple_canopy(check: &mut ValidateCx<'_>, canopy: &SimpleCanopy) {
    if canopy.canopy_offset.min.0 > canopy.canopy_offset.max.0 {
        setter(check, "canopy_offset", CANOPY_OFFSET);
    }
    check.int_min("min_width", canopy.min_width, 0);
    check.opt(
        "canopy_slope",
        canopy.canopy_slope.as_ref(),
        |check, slope| {
            check.int_min("rise", slope.rise, 1);
            check.int_min("run", slope.run, 1);
        },
    );
    check.chances("variation_chance", canopy.variation_chance.as_ref());
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
    check.opt(
        "canopy_decoration",
        canopy.canopy_decoration.as_ref(),
        decoration,
    );
}

fn acacia_canopy(check: &mut ValidateCx<'_>, canopy: &AcaciaCanopy) {
    check.int_min("canopy_size", Some(canopy.canopy_size), 1);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn cherry_canopy(check: &mut ValidateCx<'_>, canopy: &CherryCanopy) {
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
    check.range_min("height", Some(&canopy.height), 4);
    check.range_min("radius", Some(&canopy.radius), 3);
    check.int_min("trunk_width", canopy.trunk_width, 1);
    for (key, chance) in [
        (
            "wide_bottom_layer_hole_chance",
            &canopy.wide_bottom_layer_hole_chance,
        ),
        ("corner_hole_chance", &canopy.corner_hole_chance),
        ("hanging_leaves_chance", &canopy.hanging_leaves_chance),
        (
            "hanging_leaves_extension_chance",
            &canopy.hanging_leaves_extension_chance,
        ),
    ] {
        check.chance(key, Some(chance));
    }
}

fn fancy_canopy(check: &mut ValidateCx<'_>, canopy: &FancyCanopy) {
    check.int_min("height", Some(canopy.height), 1);
    check.int_min("radius", Some(canopy.radius), 0);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn weighted_blocks(check: &mut ValidateCx<'_>, key: &str, blocks: &[WeightedBlock]) {
    check.each(key, blocks, |check, entry| {
        check.descriptor("0", Some(&entry.block));
    });
}

fn mangrove_canopy(check: &mut ValidateCx<'_>, canopy: &MangroveCanopy) {
    check.range_min("canopy_height", Some(&canopy.canopy_height), 1);
    check.range_min("canopy_radius", Some(&canopy.canopy_radius), 1);
    check.int_min(
        "leaf_placement_attempts",
        Some(canopy.leaf_placement_attempts),
        1,
    );
    weighted_blocks(check, "leaf_blocks", &canopy.leaf_blocks);
    check.opt(
        "canopy_decoration",
        canopy.canopy_decoration.as_ref(),
        decoration,
    );
    check.descriptor("hanging_block", Some(&canopy.hanging_block));
    check.chance(
        "hanging_block_placement_chance",
        Some(&canopy.hanging_block_placement_chance),
    );
}

fn mega_canopy(check: &mut ValidateCx<'_>, canopy: &MegaCanopy) {
    check.range_min("canopy_height", Some(&canopy.canopy_height), 1);
    check.int_min("base_radius", Some(canopy.base_radius), 0);
    check.int_min("core_width", canopy.core_width, 1);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn mega_pine_canopy(check: &mut ValidateCx<'_>, canopy: &MegaPineCanopy) {
    check.range_min("canopy_height", Some(&canopy.canopy_height), 1);
    check.int_min("base_radius", Some(canopy.base_radius), 0);
    check.check_float(
        "radius_step_modifier",
        Some(canopy.radius_step_modifier),
        0.0,
        f32::MAX,
    );
    check.int_min("core_width", canopy.core_width, 1);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn pine_canopy(check: &mut ValidateCx<'_>, canopy: &PineCanopy) {
    check.range_min("canopy_height", Some(&canopy.canopy_height), 1);
    check.int_min("base_radius", Some(canopy.base_radius), 1);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn roofed_canopy(check: &mut ValidateCx<'_>, canopy: &RoofedCanopy) {
    check.int_min("canopy_height", Some(canopy.canopy_height), 3);
    check.int_min("core_width", Some(canopy.core_width), 1);
    check.int_min("outer_radius", Some(canopy.outer_radius), 0);
    check.int_min("inner_radius", Some(canopy.inner_radius), 0);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn spruce_canopy(check: &mut ValidateCx<'_>, canopy: &SpruceCanopy) {
    check.range_min("lower_offset", Some(&canopy.lower_offset), 1);
    check.range_min("upper_offset", Some(&canopy.upper_offset), 0);
    check.range_min("max_radius", Some(&canopy.max_radius), 1);
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
}

fn random_spread_canopy(check: &mut ValidateCx<'_>, canopy: &RandomSpreadCanopy) {
    check.range_min("canopy_height", Some(&canopy.canopy_height), 1);
    check.range_min("canopy_radius", Some(&canopy.canopy_radius), 1);
    check.int_min(
        "leaf_placement_attempts",
        Some(canopy.leaf_placement_attempts),
        1,
    );
    weighted_blocks(check, "leaf_blocks", &canopy.leaf_blocks);
}

fn poplar_canopy(check: &mut ValidateCx<'_>, canopy: &PoplarCanopy) {
    check.descriptor("leaf_block", Some(&canopy.leaf_block));
    check.descriptor("branch_block", Some(&canopy.branch_block));
    // The array's minimum length is one.
    check.check_len("radius", Some(canopy.radius.len()), 1, None);
    check.range_min("height", Some(&canopy.height), 4);
    check.check_float("side_hole_chance", canopy.side_hole_chance, 0.0, 1.0);
    check.int_min("trunk_width", canopy.trunk_width, 1);
}

fn mangrove_roots(check: &mut ValidateCx<'_>, roots: &MangroveRoots) {
    check.int_min("max_root_width", Some(roots.max_root_width), 1);
    check.int_min("max_root_length", Some(roots.max_root_length), 1);
    check.descriptor("root_block", Some(&roots.root_block));
    check.opt("above_root", roots.above_root.as_ref(), |check, above| {
        check.chance("above_root_chance", above.above_root_chance.as_ref());
        check.descriptor("above_root_block", above.above_root_block.as_ref());
    });
    check.descriptor("muddy_root_block", Some(&roots.muddy_root_block));
    check.descriptor("mud_block", Some(&roots.mud_block));
    check.range_min("y_offset", Some(&roots.y_offset), 1);
    check.descriptors(
        "roots_may_grow_through",
        Some(&roots.roots_may_grow_through),
    );
    check.opt(
        "root_decoration",
        roots.root_decoration.as_ref(),
        decoration,
    );
}

impl Rules for TreeFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.list("base_block", self.base_block.as_ref());
        check.opt(
            "base_cluster",
            self.base_cluster.as_ref(),
            |check, cluster| {
                check.descriptors("may_replace", Some(&cluster.may_replace));
                check.int_min("num_clusters", Some(cluster.num_clusters), 1);
                check.int_min("cluster_radius", Some(cluster.cluster_radius), 0);
            },
        );
        check.descriptors("may_grow_on", self.may_grow_on.as_deref());
        check.descriptors("may_replace", self.may_replace.as_deref());
        check.descriptors("may_grow_through", self.may_grow_through.as_deref());
        check.since_engine("poplar_trunk", self.poplar_trunk.is_some(), ENGINE_1_26_40);
        check.since_engine(
            "poplar_canopy",
            self.poplar_canopy.is_some(),
            ENGINE_1_26_40,
        );
        check.opt("trunk", self.trunk.as_ref(), simple_trunk);
        check.opt("acacia_trunk", self.acacia_trunk.as_ref(), acacia_trunk);
        check.opt("cherry_trunk", self.cherry_trunk.as_ref(), cherry_trunk);
        check.opt("fallen_trunk", self.fallen_trunk.as_ref(), fallen_trunk);
        check.opt("fancy_trunk", self.fancy_trunk.as_ref(), fancy_trunk);
        check.opt(
            "mangrove_trunk",
            self.mangrove_trunk.as_ref(),
            mangrove_trunk,
        );
        check.opt("mega_trunk", self.mega_trunk.as_ref(), mega_trunk);
        check.opt("poplar_trunk", self.poplar_trunk.as_ref(), poplar_trunk);
        check.opt("canopy", self.canopy.as_ref(), simple_canopy);
        check.opt("acacia_canopy", self.acacia_canopy.as_ref(), acacia_canopy);
        check.opt("cherry_canopy", self.cherry_canopy.as_ref(), cherry_canopy);
        check.opt("fancy_canopy", self.fancy_canopy.as_ref(), fancy_canopy);
        check.opt(
            "mangrove_canopy",
            self.mangrove_canopy.as_ref(),
            mangrove_canopy,
        );
        check.opt("mega_canopy", self.mega_canopy.as_ref(), mega_canopy);
        check.opt(
            "mega_pine_canopy",
            self.mega_pine_canopy.as_ref(),
            mega_pine_canopy,
        );
        check.opt("pine_canopy", self.pine_canopy.as_ref(), pine_canopy);
        check.opt("roofed_canopy", self.roofed_canopy.as_ref(), roofed_canopy);
        check.opt("spruce_canopy", self.spruce_canopy.as_ref(), spruce_canopy);
        check.opt(
            "random_spread_canopy",
            self.random_spread_canopy.as_ref(),
            random_spread_canopy,
        );
        check.opt("poplar_canopy", self.poplar_canopy.as_ref(), poplar_canopy);
        check.opt(
            "mangrove_roots",
            self.mangrove_roots.as_ref(),
            mangrove_roots,
        );
    }
}

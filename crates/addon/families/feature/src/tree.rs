//! `minecraft:tree_feature`: a tree assembled from a trunk, a canopy and roots, each picked by
//! the member it is written under.

use facet::Facet;
use facet_value::VObject;

use super::Description;
use super::blocks::WeightedHeight;
use super::codes;
use super::molang::{DESCRIPTOR_TAGS, refers};
use super::values::{BlockList, Chance, ChanceList, JsonInt, WeightedBlock};
use bedrock_addon_core::diagnostic::{Code, CodeDef};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonType, StrictError, Value};
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx, SchemaFields, from_value};
use bedrock_addon_core::shared::IntRange;
use bedrock_addon_shared_shapes::BlockDescriptor;

/// `minecraft:tree_feature`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct TreeFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block(s) placed below the trunk (JSON `base_block`).
    pub base_block: Option<BlockList>,
    /// Clusters of the base block around the trunk (JSON `base_cluster`).
    pub base_cluster: Option<BaseCluster>,
    /// The blocks the tree may grow on (JSON `may_grow_on`).
    pub may_grow_on: Option<Vec<BlockDescriptor>>,
    /// The blocks the tree may replace (JSON `may_replace`).
    pub may_replace: Option<Vec<BlockDescriptor>>,
    /// The blocks the tree may grow through (JSON `may_grow_through`).
    pub may_grow_through: Option<Vec<BlockDescriptor>>,
    /// JSON `trunk`.
    pub trunk: Option<SimpleTrunk>,
    /// JSON `acacia_trunk`.
    pub acacia_trunk: Option<AcaciaTrunk>,
    /// JSON `cherry_trunk`.
    pub cherry_trunk: Option<CherryTrunk>,
    /// JSON `fallen_trunk`.
    pub fallen_trunk: Option<FallenTrunk>,
    /// JSON `fancy_trunk`.
    pub fancy_trunk: Option<FancyTrunk>,
    /// JSON `mangrove_trunk`.
    pub mangrove_trunk: Option<MangroveTrunk>,
    /// JSON `mega_trunk`.
    pub mega_trunk: Option<MegaTrunk>,
    /// JSON `poplar_trunk` (engine 1.26.40).
    #[facet(schema::since_engine = "1.26.40")]
    pub poplar_trunk: Option<PoplarTrunk>,
    /// JSON `canopy`.
    pub canopy: Option<SimpleCanopy>,
    /// JSON `acacia_canopy`.
    pub acacia_canopy: Option<AcaciaCanopy>,
    /// JSON `cherry_canopy`.
    pub cherry_canopy: Option<CherryCanopy>,
    /// JSON `fancy_canopy`.
    pub fancy_canopy: Option<FancyCanopy>,
    /// JSON `mangrove_canopy`.
    pub mangrove_canopy: Option<MangroveCanopy>,
    /// JSON `mega_canopy`.
    pub mega_canopy: Option<MegaCanopy>,
    /// JSON `mega_pine_canopy`.
    pub mega_pine_canopy: Option<MegaPineCanopy>,
    /// JSON `pine_canopy`.
    pub pine_canopy: Option<PineCanopy>,
    /// JSON `roofed_canopy`.
    pub roofed_canopy: Option<RoofedCanopy>,
    /// JSON `spruce_canopy`.
    pub spruce_canopy: Option<SpruceCanopy>,
    /// JSON `random_spread_canopy`.
    pub random_spread_canopy: Option<RandomSpreadCanopy>,
    /// JSON `poplar_canopy` (engine 1.26.40).
    #[facet(schema::since_engine = "1.26.40")]
    pub poplar_canopy: Option<PoplarCanopy>,
    /// JSON `mangrove_roots`.
    pub mangrove_roots: Option<MangroveRoots>,
}

/// `tree_feature.base_cluster`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct BaseCluster {
    /// The blocks a cluster may replace (JSON `may_replace`).
    pub may_replace: Vec<BlockDescriptor>,
    /// How many clusters, at least 1 (JSON `num_clusters`).
    #[facet(schema::min = 1)]
    pub num_clusters: JsonInt,
    /// Their radius, at least 0 (JSON `cluster_radius`).
    #[facet(schema::min = 0)]
    pub cluster_radius: JsonInt,
}

/// A decoration attached to a trunk, canopy or roots (`trunk_decoration`, `canopy_decoration`,
/// `root_decoration`).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Decoration {
    /// The chance of a decoration (JSON `decoration_chance`).
    pub decoration_chance: Chance,
    /// The blocks placed in sequence (JSON `decoration_blocks_sequence`, engine 1.26.10); when
    /// both are written it takes priority over `decoration_block`.
    #[facet(schema::since_engine = "1.26.10")]
    pub decoration_blocks_sequence: Option<Vec<DecorationBlock>>,
    /// The block placed (JSON `decoration_block`).
    pub decoration_block: Option<BlockDescriptor>,
    /// How many blocks (JSON `num_steps`).
    pub num_steps: Option<JsonInt>,
    /// Which way the steps go (JSON `step_direction`).
    pub step_direction: Option<StepDirection>,
}

/// One entry of `decoration_blocks_sequence`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct DecorationBlock {
    /// JSON `block`.
    pub block: BlockDescriptor,
    /// How many of it (JSON `count`).
    pub count: Option<IntRange>,
}

/// `step_direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = simple_trunk)]
pub struct SimpleTrunk {
    /// At least 1 (JSON `trunk_height`).
    #[facet(schema::min = 1)]
    pub trunk_height: IntRange,
    /// JSON `height_modifier`.
    pub height_modifier: Option<IntRange>,
    /// Whether, or how deep, the trunk may be under water (JSON `can_be_submerged`).
    pub can_be_submerged: Option<Submerged>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = fallen_trunk)]
pub struct FallenTrunk {
    /// At least 1 (JSON `log_length`).
    #[facet(schema::min = 1)]
    pub log_length: IntRange,
    /// At least 1 (JSON `stump_height`).
    #[facet(schema::min = 1)]
    pub stump_height: Option<IntRange>,
    /// JSON `height_modifier`.
    pub height_modifier: Option<IntRange>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// The feature placed on the log (JSON `log_decoration_feature`).
    pub log_decoration_feature: Option<FeatureId>,
    /// JSON `trunk_decoration`.
    pub trunk_decoration: Option<Decoration>,
}

/// `fancy_trunk`: a large oak trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct FancyTrunk {
    /// JSON `trunk_height` (its members required when present).
    pub trunk_height: Option<FancyTrunkHeight>,
    /// At least 1 (JSON `trunk_width`).
    #[facet(schema::min = 1)]
    pub trunk_width: JsonInt,
    /// JSON `branches` (its members required when present).
    pub branches: Option<FancyBranches>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// At least 0 (JSON `width_scale`).
    #[facet(schema::min = 0.0)]
    pub width_scale: f32,
    /// In `[0, 1]` (JSON `foliage_altitude_factor`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub foliage_altitude_factor: f32,
}

/// `fancy_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct FancyTrunkHeight {
    /// At least 1 (JSON `base`).
    #[facet(schema::min = 1)]
    pub base: JsonInt,
    /// At least 1 (JSON `variance`).
    #[facet(schema::min = 1)]
    pub variance: JsonInt,
    /// JSON `scale`.
    pub scale: f32,
}

/// `fancy_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct FancyBranches {
    /// JSON `slope`.
    pub slope: f32,
    /// JSON `density`.
    pub density: f32,
    /// In `[0, 1]` (JSON `min_altitude_factor`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub min_altitude_factor: f32,
}

/// `mega_trunk`: a wide trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MegaTrunk {
    /// JSON `trunk_width`.
    pub trunk_width: JsonInt,
    /// JSON `trunk_height`.
    pub trunk_height: MegaTrunkHeight,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
    pub trunk_decoration: Option<Decoration>,
    /// JSON `branches`.
    pub branches: Option<MegaBranches>,
}

/// `mega_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MegaTrunkHeight {
    /// At least 1 (JSON `base`).
    #[facet(schema::min = 1)]
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(schema::min = 1)]
    pub intervals: Option<Vec<JsonInt>>,
}

/// `mega_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MegaBranches {
    /// At least 1 (JSON `branch_length`).
    #[facet(schema::min = 1)]
    pub branch_length: JsonInt,
    /// JSON `branch_slope`.
    pub branch_slope: f32,
    /// At least 0 (JSON `branch_interval`).
    #[facet(schema::min = 0)]
    pub branch_interval: IntRange,
    /// JSON `branch_altitude_factor`.
    pub branch_altitude_factor: AltitudeFactor,
    /// JSON `branch_canopy`.
    pub branch_canopy: Option<BranchCanopy>,
}

/// `mega_trunk.branches.branch_altitude_factor`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AltitudeFactor {
    /// In `[0, 1]` (JSON `min`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub min: f32,
    /// In `[0, 1]` (JSON `max`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub max: f32,
}

/// `acacia_trunk`: a leaning trunk with branches.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
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
    pub branches: Option<AcaciaBranches>,
    /// JSON `trunk_decoration`.
    pub trunk_decoration: Option<Decoration>,
}

/// `acacia_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AcaciaTrunkHeight {
    /// At least 1 (JSON `base`).
    #[facet(schema::min = 1)]
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(schema::min = 1)]
    pub intervals: Option<Vec<JsonInt>>,
    /// At least 1 (JSON `min_height_for_canopy`).
    #[facet(schema::min = 1)]
    pub min_height_for_canopy: Option<JsonInt>,
}

/// `acacia_trunk.trunk_lean`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct TrunkLean {
    /// JSON `allow_diagonal_growth`.
    pub allow_diagonal_growth: bool,
    /// At least 0 (JSON `lean_height`).
    #[facet(schema::min = 0)]
    pub lean_height: IntRange,
    /// At least 0 (JSON `lean_steps`).
    #[facet(schema::min = 0)]
    pub lean_steps: IntRange,
    /// At least 0 (JSON `lean_length`).
    #[facet(schema::min = 0)]
    pub lean_length: Option<IntRange>,
}

/// `acacia_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AcaciaBranches {
    /// At least 1 (JSON `branch_length`).
    #[facet(schema::min = 1)]
    pub branch_length: IntRange,
    /// At least 1 (JSON `branch_position`).
    #[facet(schema::min = 1)]
    pub branch_position: IntRange,
    /// JSON `branch_chance`.
    pub branch_chance: Chance,
    /// JSON `branch_canopy`.
    pub branch_canopy: Option<BranchCanopy>,
}

/// `cherry_trunk`: a trunk that splits into branches.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CherryTrunk {
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_height`.
    pub trunk_height: CherryTrunkHeight,
    /// JSON `branches`.
    pub branches: CherryBranches,
}

/// `cherry_trunk.trunk_height`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CherryTrunkHeight {
    /// At least 2 (JSON `base`).
    #[facet(schema::min = 2)]
    pub base: JsonInt,
    /// Each at least 1 (JSON `intervals`).
    #[facet(schema::min = 1)]
    pub intervals: Option<Vec<JsonInt>>,
}

/// `cherry_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CherryBranches {
    /// JSON `tree_type_weights`.
    pub tree_type_weights: Option<TreeTypeWeights>,
    /// At least 2 (JSON `branch_horizontal_length`).
    #[facet(schema::min = 2)]
    pub branch_horizontal_length: IntRange,
    /// At most 0, with at least two values (JSON `branch_start_offset_from_top`).
    #[facet(schema::max = 0, schema::check_with = start_offset)]
    pub branch_start_offset_from_top: IntRange,
    /// JSON `branch_end_offset_from_top`.
    pub branch_end_offset_from_top: IntRange,
    /// JSON `branch_canopy`.
    pub branch_canopy: Option<BranchCanopy>,
}

/// `cherry_trunk.branches.tree_type_weights`: each at least 0.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct TreeTypeWeights {
    /// JSON `one_branch`.
    #[facet(schema::min = 0)]
    pub one_branch: JsonInt,
    /// JSON `two_branches`.
    #[facet(schema::min = 0)]
    pub two_branches: JsonInt,
    /// JSON `two_branches_and_trunk`.
    #[facet(schema::min = 0)]
    pub two_branches_and_trunk: JsonInt,
}

/// `mangrove_trunk`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MangroveTrunk {
    /// JSON `trunk_width`.
    pub trunk_width: JsonInt,
    /// JSON `trunk_height` (its members required when present).
    pub trunk_height: Option<MangroveTrunkHeight>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `branches`.
    pub branches: Option<MangroveBranches>,
    /// JSON `trunk_decoration`.
    pub trunk_decoration: Option<Decoration>,
}

/// `mangrove_trunk.trunk_height`: each at least 1.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MangroveTrunkHeight {
    /// JSON `base`.
    #[facet(schema::min = 1)]
    pub base: JsonInt,
    /// JSON `height_rand_a`.
    #[facet(schema::min = 1)]
    pub height_rand_a: JsonInt,
    /// JSON `height_rand_b`.
    #[facet(schema::min = 1)]
    pub height_rand_b: JsonInt,
}

/// `mangrove_trunk.branches`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MangroveBranches {
    /// At least 0 (JSON `branch_length`).
    #[facet(schema::min = 0)]
    pub branch_length: IntRange,
    /// At least 0 (JSON `branch_steps`).
    #[facet(schema::min = 0)]
    pub branch_steps: IntRange,
    /// JSON `branch_chance`.
    pub branch_chance: Chance,
}

/// `poplar_trunk` (engine 1.26.40).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct PoplarTrunk {
    /// At least 1 (JSON `trunk_height`).
    #[facet(schema::min = 1)]
    pub trunk_height: IntRange,
    /// JSON `remaining_trunk_height_above_branches`.
    pub remaining_trunk_height_above_branches: Option<IntRange>,
    /// JSON `amount_of_foliage_support_branches`.
    pub amount_of_foliage_support_branches: Option<IntRange>,
    /// JSON `trunk_block`.
    pub trunk_block: BlockDescriptor,
    /// JSON `trunk_decoration`.
    pub trunk_decoration: Option<Decoration>,
    /// JSON `log_decoration_feature`.
    pub log_decoration_feature: Option<FeatureId>,
}

// ---- canopies ----------------------------------------------------------------------------------

/// The canopy a branch ends in (`branches.branch_canopy`): one of the canopies, except the
/// random-spread and poplar ones.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct BranchCanopy {
    /// JSON `canopy`.
    pub canopy: Option<SimpleCanopy>,
    /// JSON `acacia_canopy`.
    pub acacia_canopy: Option<AcaciaCanopy>,
    /// JSON `cherry_canopy`.
    pub cherry_canopy: Option<CherryCanopy>,
    /// JSON `fancy_canopy`.
    pub fancy_canopy: Option<FancyCanopy>,
    /// JSON `mangrove_canopy`.
    pub mangrove_canopy: Option<MangroveCanopy>,
    /// JSON `mega_canopy`.
    pub mega_canopy: Option<MegaCanopy>,
    /// JSON `mega_pine_canopy`.
    pub mega_pine_canopy: Option<MegaPineCanopy>,
    /// JSON `pine_canopy`.
    pub pine_canopy: Option<PineCanopy>,
    /// JSON `roofed_canopy`.
    pub roofed_canopy: Option<RoofedCanopy>,
    /// JSON `spruce_canopy`.
    pub spruce_canopy: Option<SpruceCanopy>,
}

/// `canopy`: a layered canopy.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SimpleCanopy {
    /// JSON `canopy_offset`.
    pub canopy_offset: CanopyOffset,
    /// At least 0 (JSON `min_width`).
    #[facet(schema::min = 0)]
    pub min_width: Option<JsonInt>,
    /// JSON `canopy_slope`.
    pub canopy_slope: Option<CanopySlope>,
    /// JSON `variation_chance`.
    pub variation_chance: Option<ChanceList>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `canopy_decoration`.
    pub canopy_decoration: Option<Decoration>,
}

/// `canopy.canopy_offset`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = canopy_offset)]
pub struct CanopyOffset {
    /// JSON `min`, at most `max`.
    pub min: JsonInt,
    /// JSON `max`.
    pub max: JsonInt,
}

/// `canopy.canopy_slope`: each at least 1.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CanopySlope {
    /// JSON `rise`.
    #[facet(schema::min = 1)]
    pub rise: Option<JsonInt>,
    /// JSON `run`.
    #[facet(schema::min = 1)]
    pub run: Option<JsonInt>,
}

/// `acacia_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AcaciaCanopy {
    /// At least 1 (JSON `canopy_size`).
    #[facet(schema::min = 1)]
    pub canopy_size: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `simplify_canopy`.
    pub simplify_canopy: Option<bool>,
}

/// `cherry_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CherryCanopy {
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// At least 4 (JSON `height`).
    #[facet(schema::min = 4)]
    pub height: IntRange,
    /// At least 3 (JSON `radius`).
    #[facet(schema::min = 3)]
    pub radius: IntRange,
    /// At least 1 (JSON `trunk_width`).
    #[facet(schema::min = 1)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct FancyCanopy {
    /// At least 1 (JSON `height`).
    #[facet(schema::min = 1)]
    pub height: JsonInt,
    /// At least 0 (JSON `radius`).
    #[facet(schema::min = 0)]
    pub radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `mangrove_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MangroveCanopy {
    /// At least 1 (JSON `canopy_height`).
    #[facet(schema::min = 1)]
    pub canopy_height: IntRange,
    /// At least 1 (JSON `canopy_radius`).
    #[facet(schema::min = 1)]
    pub canopy_radius: IntRange,
    /// At least 1 (JSON `leaf_placement_attempts`).
    #[facet(schema::min = 1)]
    pub leaf_placement_attempts: JsonInt,
    /// JSON `leaf_blocks`.
    pub leaf_blocks: Vec<WeightedBlock>,
    /// JSON `canopy_decoration`.
    pub canopy_decoration: Option<Decoration>,
    /// JSON `hanging_block`.
    pub hanging_block: BlockDescriptor,
    /// JSON `hanging_block_placement_chance`.
    pub hanging_block_placement_chance: Chance,
}

/// `mega_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MegaCanopy {
    /// At least 1 (JSON `canopy_height`).
    #[facet(schema::min = 1)]
    pub canopy_height: IntRange,
    /// At least 0 (JSON `base_radius`).
    #[facet(schema::min = 0)]
    pub base_radius: JsonInt,
    /// At least 1 (JSON `core_width`).
    #[facet(schema::min = 1)]
    pub core_width: Option<JsonInt>,
    /// JSON `simplify_canopy`.
    pub simplify_canopy: Option<bool>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `mega_pine_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MegaPineCanopy {
    /// At least 1 (JSON `canopy_height`).
    #[facet(schema::min = 1)]
    pub canopy_height: IntRange,
    /// At least 0 (JSON `base_radius`).
    #[facet(schema::min = 0)]
    pub base_radius: JsonInt,
    /// At least 0 (JSON `radius_step_modifier`).
    #[facet(schema::min = 0.0)]
    pub radius_step_modifier: f32,
    /// At least 1 (JSON `core_width`).
    #[facet(schema::min = 1)]
    pub core_width: Option<JsonInt>,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `pine_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct PineCanopy {
    /// At least 1 (JSON `canopy_height`).
    #[facet(schema::min = 1)]
    pub canopy_height: IntRange,
    /// At least 1 (JSON `base_radius`).
    #[facet(schema::min = 1)]
    pub base_radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `roofed_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct RoofedCanopy {
    /// At least 3 (JSON `canopy_height`).
    #[facet(schema::min = 3)]
    pub canopy_height: JsonInt,
    /// At least 1 (JSON `core_width`).
    #[facet(schema::min = 1)]
    pub core_width: JsonInt,
    /// At least 0 (JSON `outer_radius`).
    #[facet(schema::min = 0)]
    pub outer_radius: JsonInt,
    /// At least 0 (JSON `inner_radius`).
    #[facet(schema::min = 0)]
    pub inner_radius: JsonInt,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `spruce_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SpruceCanopy {
    /// At least 1 (JSON `lower_offset`).
    #[facet(schema::min = 1)]
    pub lower_offset: IntRange,
    /// At least 0 (JSON `upper_offset`).
    #[facet(schema::min = 0)]
    pub upper_offset: IntRange,
    /// At least 1 (JSON `max_radius`).
    #[facet(schema::min = 1)]
    pub max_radius: IntRange,
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
}

/// `random_spread_canopy`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct RandomSpreadCanopy {
    /// At least 1 (JSON `canopy_height`).
    #[facet(schema::min = 1)]
    pub canopy_height: IntRange,
    /// At least 1 (JSON `canopy_radius`).
    #[facet(schema::min = 1)]
    pub canopy_radius: IntRange,
    /// At least 1 (JSON `leaf_placement_attempts`).
    #[facet(schema::min = 1)]
    pub leaf_placement_attempts: JsonInt,
    /// JSON `leaf_blocks`.
    pub leaf_blocks: Vec<WeightedBlock>,
}

/// `poplar_canopy` (engine 1.26.40).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct PoplarCanopy {
    /// JSON `leaf_block`.
    pub leaf_block: BlockDescriptor,
    /// JSON `branch_block`.
    pub branch_block: BlockDescriptor,
    /// Radii and their weights, at least one (JSON `radius`).
    #[facet(schema::min_len = 1)]
    pub radius: Vec<WeightedHeight>,
    /// At least 4 (JSON `height`).
    #[facet(schema::min = 4)]
    pub height: IntRange,
    /// In `[0, 1]` (JSON `side_hole_chance`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub side_hole_chance: Option<f32>,
    /// At least 1 (JSON `trunk_width`).
    #[facet(schema::min = 1)]
    pub trunk_width: Option<JsonInt>,
}

// ---- roots -------------------------------------------------------------------------------------

/// `mangrove_roots`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct MangroveRoots {
    /// At least 1 (JSON `max_root_width`).
    #[facet(schema::min = 1)]
    pub max_root_width: JsonInt,
    /// At least 1 (JSON `max_root_length`).
    #[facet(schema::min = 1)]
    pub max_root_length: JsonInt,
    /// JSON `root_block`.
    pub root_block: BlockDescriptor,
    /// JSON `above_root`.
    pub above_root: Option<AboveRoot>,
    /// JSON `muddy_root_block`.
    pub muddy_root_block: BlockDescriptor,
    /// JSON `mud_block`.
    pub mud_block: BlockDescriptor,
    /// At least 1 (JSON `y_offset`).
    #[facet(schema::min = 1)]
    pub y_offset: IntRange,
    /// JSON `roots_may_grow_through`.
    pub roots_may_grow_through: Vec<BlockDescriptor>,
    /// JSON `root_decoration`.
    pub root_decoration: Option<Decoration>,
}

/// `mangrove_roots.above_root`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AboveRoot {
    /// JSON `above_root_chance`.
    pub above_root_chance: Option<Chance>,
    /// JSON `above_root_block`.
    pub above_root_block: Option<BlockDescriptor>,
}

// ---- version struct, references, rules ----------------------------------------------------------

refers!(TreeFeature {
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
refers!(BaseCluster { "may_replace" => may_replace });
refers!(Decoration {
    "decoration_blocks_sequence" => decoration_blocks_sequence,
    "decoration_block" => decoration_block,
});
refers!(DecorationBlock { "block" => block });
refers!(SimpleTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration });
refers!(FallenTrunk {
    "trunk_block" => trunk_block,
    "trunk_decoration" => trunk_decoration,
    "log_decoration_feature" => log_decoration_feature,
});
refers!(FancyTrunk { "trunk_block" => trunk_block });
refers!(MegaTrunk {
    "trunk_block" => trunk_block,
    "trunk_decoration" => trunk_decoration,
    "branches" => branches,
});
refers!(MegaBranches { "branch_canopy" => branch_canopy });
refers!(AcaciaTrunk {
    "trunk_block" => trunk_block,
    "branches" => branches,
    "trunk_decoration" => trunk_decoration,
});
refers!(AcaciaBranches { "branch_canopy" => branch_canopy });
refers!(CherryTrunk { "trunk_block" => trunk_block, "branches" => branches });
refers!(CherryBranches { "branch_canopy" => branch_canopy });
refers!(MangroveTrunk { "trunk_block" => trunk_block, "trunk_decoration" => trunk_decoration });
refers!(PoplarTrunk {
    "trunk_block" => trunk_block,
    "trunk_decoration" => trunk_decoration,
    "log_decoration_feature" => log_decoration_feature,
});
refers!(BranchCanopy {
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
refers!(SimpleCanopy { "leaf_block" => leaf_block, "canopy_decoration" => canopy_decoration });
refers!(AcaciaCanopy { "leaf_block" => leaf_block });
refers!(CherryCanopy { "leaf_block" => leaf_block });
refers!(FancyCanopy { "leaf_block" => leaf_block });
refers!(MangroveCanopy {
    "leaf_blocks" => leaf_blocks,
    "canopy_decoration" => canopy_decoration,
    "hanging_block" => hanging_block,
});
refers!(MegaCanopy { "leaf_block" => leaf_block });
refers!(MegaPineCanopy { "leaf_block" => leaf_block });
refers!(PineCanopy { "leaf_block" => leaf_block });
refers!(RoofedCanopy { "leaf_block" => leaf_block });
refers!(SpruceCanopy { "leaf_block" => leaf_block });
refers!(RandomSpreadCanopy { "leaf_blocks" => leaf_blocks });
refers!(PoplarCanopy { "leaf_block" => leaf_block, "branch_block" => branch_block });
refers!(MangroveRoots {
    "root_block" => root_block,
    "above_root" => above_root,
    "muddy_root_block" => muddy_root_block,
    "mud_block" => mud_block,
    "roots_may_grow_through" => roots_may_grow_through,
    "root_decoration" => root_decoration,
});
refers!(AboveRoot { "above_root_block" => above_root_block });

// Read as the game's reader of the shape reads it (`TryFrom<Value>`), not by the slice's
// scalar rules; a refusal is one wrong type at the member.
impl FromJson for Submerged {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        from_value(node, read, Self::try_from)
    }
}
bedrock_addon_core::impl_no_molang!(Submerged);

/// A depth is at least 1.
impl SchemaFields for Submerged {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        if let Self::MaxDepth(depth) = *self {
            cx.enter("max_depth");
            cx.within("", depth, Some(1), None);
            cx.leave();
        }
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

/// A setter's message as an `InvalidValue` kept error: the client logs it at level 3 and keeps
/// the value (for the trunk, the cherry start offset and each canopy offset).
fn setter(cx: &mut SchemaCx<'_>, key: &str, code: &'static CodeDef<0>) {
    cx.kept(key, Code::from(code));
}

/// A negative `height_modifier` that outweighs the height it modifies.
fn height_modifier(cx: &mut SchemaCx<'_>, height: &IntRange, modifier: Option<&IntRange>) {
    if let Some(modifier) = modifier
        && modifier.min < 0
        && modifier.min.unsigned_abs() > height.min.unsigned_abs()
    {
        setter(cx, "height_modifier", &codes::INVERTED_TRUNK);
    }
}

fn simple_trunk(trunk: &mut SimpleTrunk, cx: &mut SchemaCx<'_>) {
    height_modifier(cx, &trunk.trunk_height, trunk.height_modifier.as_ref());
}

fn fallen_trunk(trunk: &mut FallenTrunk, cx: &mut SchemaCx<'_>) {
    height_modifier(cx, &trunk.log_length, trunk.height_modifier.as_ref());
}

/// The range's own finding, then the client's setter for a start offset that does not vary.
fn start_offset(start: &mut IntRange, cx: &mut SchemaCx<'_>) {
    start.check_fields(cx);
    if start.min == start.max {
        setter(cx, "", &codes::CHERRY_START_OFFSET);
    }
}

fn canopy_offset(offset: &mut CanopyOffset, cx: &mut SchemaCx<'_>) {
    if offset.min.0 > offset.max.0 {
        setter(cx, "", &codes::CANOPY_OFFSET);
    }
}

impl TreeFeature {
    /// A tree with this identifier and no parts yet.
    pub fn new(identifier: FeatureId) -> Self {
        Self {
            description: super::Description::new(identifier),
            base_block: None,
            base_cluster: None,
            may_grow_on: None,
            may_replace: None,
            may_grow_through: None,
            trunk: None,
            acacia_trunk: None,
            cherry_trunk: None,
            fallen_trunk: None,
            fancy_trunk: None,
            mangrove_trunk: None,
            mega_trunk: None,
            poplar_trunk: None,
            canopy: None,
            acacia_canopy: None,
            cherry_canopy: None,
            fancy_canopy: None,
            mangrove_canopy: None,
            mega_canopy: None,
            mega_pine_canopy: None,
            pine_canopy: None,
            roofed_canopy: None,
            spruce_canopy: None,
            random_spread_canopy: None,
            poplar_canopy: None,
            mangrove_roots: None,
        }
    }
}

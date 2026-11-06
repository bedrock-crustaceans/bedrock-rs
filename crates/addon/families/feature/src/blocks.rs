//! Feature types that place blocks: ores, single and multi-part blocks, blobs, geodes, plants,
//! patches and columns.

use facet::Facet;
use facet_value::{VArray, VObject};

use super::Description;
use super::molang::{Walk, walk};
use super::read::parse_strict_with;
use super::validate::Check;
use super::values::{BlockList, JsonInt, WeightedBlock, pair, weight_of, weight_value};
use super::{Rules, V1_13_0};
use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, JsonType, StrictError, Value};
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::shared::IntRange;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_shared_shapes::{BlockDescriptor, BlockSpecifier};

// ---- ore --------------------------------------------------------------------------------------

/// `minecraft:ore_feature`: a vein of blocks that replace what they may replace.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct OreFeature {
    /// JSON `description`.
    pub description: Description,
    /// How many blocks the vein has, at least one (JSON `count`).
    pub count: JsonInt,
    /// The chance, in `[0, 1]`, that a block exposed to air is not placed (JSON
    /// `discard_chance_on_air_exposure`); absent is 0.
    pub discard_chance_on_air_exposure: Option<f32>,
    /// Which block replaces which; at least one rule (JSON `replace_rules`).
    pub replace_rules: Option<Vec<ReplaceRule>>,
}

/// One entry of `ore_feature.replace_rules`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ReplaceRule {
    /// The block placed (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// The blocks it may replace, at least one; absent replaces any block (JSON
    /// `may_replace`).
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

// ---- single block -----------------------------------------------------------------------------

/// `minecraft:single_block_feature`: one block, placed if the attach and replace rules allow it.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SingleBlockFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block, or blocks with weights, to place (JSON `places_block`).
    pub places_block: Option<PlacesBlock>,
    /// Whether the block's own placement rules apply (JSON `enforce_placement_rules`).
    pub enforce_placement_rules: bool,
    /// Whether the block's survival rules apply (JSON `enforce_survivability_rules`).
    pub enforce_survivability_rules: bool,
    /// Whether the block is placed in a random rotation (JSON `randomize_rotation`); absent is
    /// `false`.
    pub randomize_rotation: Option<bool>,
    /// The blocks it may attach to (JSON `may_attach_to`).
    pub may_attach_to: Option<AttachConditions>,
    /// The blocks it may not attach to (JSON `may_not_attach_to`).
    pub may_not_attach_to: Option<AttachConditions>,
    /// The blocks it may replace (JSON `may_replace`).
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

/// `single_block_feature.places_block`.
///
/// JSON: a block specifier, or an array of `{"block": specifier, "weight": number}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum PlacesBlock {
    /// One block.
    Specifier(BlockSpecifier),
    /// Blocks picked by weight.
    Weighted(Vec<WeightedSpecifier>),
    /// A descriptor of a document below format 1.21.40, carried by the upgrade unchanged: one
    /// block (or the blocks its tags match) with weight 1.
    Descriptor(BlockDescriptor),
}

/// One entry of a weighted `places_block`.
#[derive(Clone, Debug, PartialEq)]
pub struct WeightedSpecifier {
    /// The block (JSON `block`).
    pub block: BlockSpecifier,
    /// Its weight; a higher one is picked more often (JSON `weight`).
    pub weight: f32,
}

impl TryFrom<Value> for PlacesBlock {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let Some(array) = value.as_array() else {
            return BlockSpecifier::try_from(value)
                .map(PlacesBlock::Specifier)
                .map_err(|_| StrictError::WrongType {
                    expected: "a block specifier",
                    found: JsonType::Null,
                });
        };
        let entry = |element: &Value| -> Result<WeightedSpecifier, StrictError> {
            let wrong = StrictError::WrongType {
                expected: "{\"block\", \"weight\"}",
                found: JsonType::Null,
            };
            let object = element.as_object().ok_or(wrong)?;
            if object
                .keys()
                .any(|key| !matches!(key.as_str(), "block" | "weight"))
            {
                return Err(wrong);
            }
            let block = object
                .get("block")
                .cloned()
                .and_then(|block| BlockSpecifier::try_from(block).ok())
                .ok_or(wrong)?;
            let weight = weight_of(object.get("weight").ok_or(wrong)?)?;
            Ok(WeightedSpecifier { block, weight })
        };
        array
            .iter()
            .map(entry)
            .collect::<Result<_, _>>()
            .map(PlacesBlock::Weighted)
    }
}

impl From<&PlacesBlock> for Value {
    fn from(places: &PlacesBlock) -> Self {
        match places {
            PlacesBlock::Specifier(specifier) => Value::from(specifier),
            PlacesBlock::Descriptor(descriptor) => Value::from(descriptor),
            PlacesBlock::Weighted(entries) => Value::from(
                entries
                    .iter()
                    .map(|entry| {
                        let mut object = VObject::new();
                        object.insert("block", Value::from(&entry.block));
                        object.insert("weight", weight_value(entry.weight));
                        Value::from(object)
                    })
                    .collect::<VArray>(),
            ),
        }
    }
}

/// `may_attach_to` / `may_not_attach_to`: per side, the blocks the placed block may (not) touch.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct AttachConditions {
    /// How many sides must attach, 1 to 4 (JSON `min_sides_must_attach`); absent is 4.
    pub min_sides_must_attach: Option<JsonInt>,
    /// Whether the block turns to face what it attaches to (JSON `auto_rotate`); absent is
    /// `true`.
    pub auto_rotate: Option<bool>,
    /// JSON `top`.
    pub top: Option<BlockList>,
    /// JSON `bottom`.
    pub bottom: Option<BlockList>,
    /// JSON `north`.
    pub north: Option<BlockList>,
    /// JSON `east`.
    pub east: Option<BlockList>,
    /// JSON `south`.
    pub south: Option<BlockList>,
    /// JSON `west`.
    pub west: Option<BlockList>,
    /// Every side (JSON `all`).
    pub all: Option<BlockList>,
    /// The four horizontal sides (JSON `sides`).
    pub sides: Option<BlockList>,
    /// The diagonal neighbours (JSON `diagonal`, format 1.21.40); absent is none.
    pub diagonal: Option<BlockList>,
}

impl AttachConditions {
    /// The sides that must attach: `min_sides_must_attach`, or 4.
    pub fn min_sides(&self) -> i32 {
        self.min_sides_must_attach.map_or(4, |n| n.0)
    }

    /// Whether the block auto-rotates: `auto_rotate`, or `true`.
    pub fn rotates(&self) -> bool {
        self.auto_rotate.unwrap_or(true)
    }
}

impl SingleBlockFeature {
    /// Whether the block is placed in a random rotation: `randomize_rotation`, or `false`.
    pub fn randomizes_rotation(&self) -> bool {
        self.randomize_rotation.unwrap_or(false)
    }
}

// ---- multi block -------------------------------------------------------------------------------

/// `minecraft:multi_block_feature` (engine 1.26.40, format 1.26.40): a multi-block, placed from
/// its starting part.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MultiBlockFeature {
    /// JSON `description`.
    pub description: Description,
    /// The multi-block's starting part (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// Whether the block's placement rules apply (JSON `enforce_placement_rules`).
    pub enforce_placement_rules: Option<bool>,
    /// Whether the block is placed in a random rotation, which needs its
    /// `minecraft:cardinal_direction` state (JSON `randomize_rotation`).
    pub randomize_rotation: Option<bool>,
    /// The blocks it may replace; absent is any block but another multi-block's part (JSON
    /// `may_replace`).
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

// ---- multiface, fossil, geode ------------------------------------------------------------------

/// `minecraft:multiface_feature`: a block that covers faces of neighbouring blocks and spreads.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MultifaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// How far it searches, 1 to 64 (JSON `search_range`).
    pub search_range: JsonInt,
    /// JSON `can_place_on_floor`.
    pub can_place_on_floor: bool,
    /// JSON `can_place_on_ceiling`.
    pub can_place_on_ceiling: bool,
    /// JSON `can_place_on_wall`.
    pub can_place_on_wall: bool,
    /// The chance, in `[0, 1]`, that it spreads (JSON `chance_of_spreading`).
    pub chance_of_spreading: f32,
    /// The blocks it may be placed on, at least one (JSON `can_place_on`).
    pub can_place_on: Option<Vec<BlockDescriptor>>,
}

/// `minecraft:fossil_feature`: a fossil structure with ore blocks.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FossilFeature {
    /// JSON `description`.
    pub description: Description,
    /// The ore block (JSON `ore_block`).
    pub ore_block: BlockDescriptor,
    /// How many corners may be empty (JSON `max_empty_corners`).
    pub max_empty_corners: JsonInt,
}

/// `minecraft:geode_feature`: a layered geode.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GeodeFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block inside (JSON `filler`).
    pub filler: BlockDescriptor,
    /// JSON `inner_layer`.
    pub inner_layer: BlockDescriptor,
    /// JSON `alternate_inner_layer`.
    pub alternate_inner_layer: BlockDescriptor,
    /// JSON `middle_layer`.
    pub middle_layer: BlockDescriptor,
    /// JSON `outer_layer`.
    pub outer_layer: BlockDescriptor,
    /// The blocks placed on the inner layer, at least one (JSON `inner_placements`).
    pub inner_placements: Option<Vec<BlockDescriptor>>,
    /// 1 to 10 (JSON `min_outer_wall_distance`).
    pub min_outer_wall_distance: JsonInt,
    /// 1 to 20 (JSON `max_outer_wall_distance`).
    pub max_outer_wall_distance: JsonInt,
    /// 1 to 10 (JSON `min_distribution_points`).
    pub min_distribution_points: JsonInt,
    /// 1 to 20 (JSON `max_distribution_points`).
    pub max_distribution_points: JsonInt,
    /// 0 to 10 (JSON `min_point_offset`).
    pub min_point_offset: JsonInt,
    /// 0 to 10 (JSON `max_point_offset`).
    pub max_point_offset: JsonInt,
    /// JSON `max_radius`.
    pub max_radius: JsonInt,
    /// 0 to 10 (JSON `crack_point_offset`).
    pub crack_point_offset: JsonInt,
    /// In `[0, 1]` (JSON `generate_crack_chance`).
    pub generate_crack_chance: f32,
    /// In `[0, 5]` (JSON `base_crack_size`).
    pub base_crack_size: f32,
    /// JSON `noise_multiplier`.
    pub noise_multiplier: f32,
    /// In `[0, 1]` (JSON `use_potential_placements_chance`).
    pub use_potential_placements_chance: f32,
    /// In `[0, 1]` (JSON `use_alternate_layer0_chance`).
    pub use_alternate_layer0_chance: f32,
    /// JSON `placements_require_layer0_alternate`.
    pub placements_require_layer0_alternate: bool,
    /// JSON `invalid_blocks_threshold`.
    pub invalid_blocks_threshold: JsonInt,
}

// ---- growing plant -----------------------------------------------------------------------------

/// `minecraft:growing_plant_feature`: a plant column of body blocks ending in a head block.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GrowingPlantFeature {
    /// JSON `description`.
    pub description: Description,
    /// Heights and their weights, at least one (JSON `height_distribution`).
    pub height_distribution: Vec<HeightWeight>,
    /// Which way it grows (JSON `growth_direction`).
    pub growth_direction: GrowthDirection,
    /// The age state of the head block (JSON `age`).
    pub age: Option<IntRange>,
    /// The body blocks and their weights, at least one (JSON `body_blocks`).
    pub body_blocks: Vec<WeightedBlock>,
    /// The head blocks and their weights, at least one (JSON `head_blocks`).
    pub head_blocks: Vec<WeightedBlock>,
    /// Whether it may grow in water (JSON `allow_water`); absent is `false`.
    pub allow_water: Option<bool>,
}

/// A height range with a weight: `[range, weight]`.
///
/// JSON: a two-element array, an int range and a number.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct HeightWeight {
    /// Position 0: the heights.
    pub height: IntRange,
    /// Position 1: the weight.
    pub weight: f32,
}

impl TryFrom<Value> for HeightWeight {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let (first, second) = pair(&value, "a [range, weight] array")?;
        let height = IntRange::try_from(first.clone()).map_err(|_| StrictError::WrongType {
            expected: "an int range",
            found: JsonType::Null,
        })?;
        Ok(HeightWeight {
            height,
            weight: weight_of(second)?,
        })
    }
}

impl TryFrom<&HeightWeight> for Value {
    type Error = bedrock_addon_core::shared::ShapeError;

    fn try_from(entry: &HeightWeight) -> Result<Self, Self::Error> {
        let mut array = VArray::new();
        array.push(Value::try_from(&entry.height)?);
        array.push(weight_value(entry.weight));
        Ok(Value::from(array))
    }
}

/// `growing_plant_feature.growth_direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "UPPERCASE")]
#[repr(u8)]
#[non_exhaustive]
pub enum GrowthDirection {
    /// `UP`.
    Up,
    /// `DOWN`.
    Down,
}

// ---- partially exposed blob, sculk patch, vegetation patch -------------------------------------

/// `minecraft:partially_exposed_blob_feature`: a blob of blocks around the input position, with
/// one face exposed.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PartiallyExposedBlobFeature {
    /// JSON `description`.
    pub description: Description,
    /// 1 to 8 (JSON `placement_radius_around_floor`).
    pub placement_radius_around_floor: JsonInt,
    /// In `[0, 1]` (JSON `placement_probability_per_valid_position`).
    pub placement_probability_per_valid_position: f32,
    /// The exposed face (JSON `exposed_face`); absent is `up`.
    pub exposed_face: Option<ExposedFace>,
    /// The block (JSON `places_block`).
    pub places_block: BlockDescriptor,
}

/// `partially_exposed_blob_feature.exposed_face`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ExposedFace {
    /// `up` (the default).
    #[default]
    Up,
    /// `down`.
    Down,
}

/// `minecraft:sculk_patch_feature` (base-game packs only): a sculk patch spread by charge
/// cursors.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SculkPatchFeature {
    /// JSON `description`.
    pub description: Description,
    /// The blocks it may be placed on (JSON `can_place_sculk_patch_on`).
    pub can_place_sculk_patch_on: Vec<BlockDescriptor>,
    /// The block at the centre (JSON `central_block`).
    pub central_block: Option<BlockDescriptor>,
    /// In `[0, 1]` (JSON `central_block_placement_chance`).
    pub central_block_placement_chance: Option<f32>,
    /// 0 to 32 (JSON `cursor_count`).
    pub cursor_count: JsonInt,
    /// 1 to 1000 (JSON `charge_amount`).
    pub charge_amount: JsonInt,
    /// 1 to 64 (JSON `spread_attempts`).
    pub spread_attempts: JsonInt,
    /// 0 to 8 (JSON `growth_rounds`).
    pub growth_rounds: JsonInt,
    /// 0 to 8 (JSON `spread_rounds`).
    pub spread_rounds: JsonInt,
    /// JSON `extra_growth_chance`.
    pub extra_growth_chance: Option<IntRange>,
}

/// `minecraft:vegetation_patch_feature`: a patch of ground block with vegetation on it.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct VegetationPatchFeature {
    /// JSON `description`.
    pub description: Description,
    /// The blocks the ground block replaces (JSON `replaceable_blocks`).
    pub replaceable_blocks: Vec<BlockDescriptor>,
    /// JSON `ground_block`.
    pub ground_block: BlockDescriptor,
    /// The vegetation placed on the patch (JSON `vegetation_feature`).
    pub vegetation_feature: FeatureId,
    /// Floor or ceiling (JSON `surface`); absent is `floor`.
    pub surface: Option<VegetationSurface>,
    /// JSON `depth`.
    pub depth: IntRange,
    /// JSON `extra_deep_block_chance`.
    pub extra_deep_block_chance: Option<f32>,
    /// JSON `vertical_range`.
    pub vertical_range: JsonInt,
    /// JSON `vegetation_chance`.
    pub vegetation_chance: Option<f32>,
    /// JSON `horizontal_radius`.
    pub horizontal_radius: IntRange,
    /// At least 0 (JSON `extra_edge_column_chance`).
    pub extra_edge_column_chance: Option<f32>,
    /// JSON `waterlogged`.
    pub waterlogged: Option<bool>,
}

/// `vegetation_patch_feature.surface`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum VegetationSurface {
    /// `ceiling`.
    Ceiling,
    /// `floor` (the default).
    #[default]
    Floor,
}

// ---- horizontal tree decoration, multipart block column ----------------------------------------

/// `minecraft:horizontal_tree_decoration_feature` (engine 1.26.40): a block on the side of a
/// trunk (vanilla's shelf mushrooms).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct HorizontalTreeDecorationFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// JSON `allow_adjacent`.
    pub allow_adjacent: Option<bool>,
    /// JSON `bark_side_only`.
    pub bark_side_only: Option<bool>,
}

/// `minecraft:multipart_block_column_feature` (format 1.26.40): a column of tip, frustum, middle
/// and base blocks.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MultipartBlockColumnFeature {
    /// JSON `description`.
    pub description: Description,
    /// JSON `tip_block`.
    pub tip_block: BlockDescriptor,
    /// JSON `frustum_block`.
    pub frustum_block: BlockDescriptor,
    /// JSON `middle_block`.
    pub middle_block: BlockDescriptor,
    /// JSON `base_block`.
    pub base_block: BlockDescriptor,
    /// JSON `height_range`.
    pub height_range: Option<IntRange>,
    /// Heights and their weights (JSON `weighted_heights`).
    pub weighted_heights: Option<Vec<WeightedHeight>>,
    /// The direction the column grows (JSON `direction`); absent is `UP`.
    pub direction: Option<String>,
    /// The blocks it may be placed on (JSON `may_place_on`).
    pub may_place_on: Option<Vec<BlockDescriptor>>,
    /// The blocks it may replace; absent is any block (JSON `may_replace`).
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

/// A value with a weight: `{"value", "weight"}` (`multipart_block_column_feature.weighted_heights`,
/// `poplar_canopy.radius`).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct WeightedHeight {
    /// JSON `value`.
    pub value: Option<JsonInt>,
    /// JSON `weight`.
    pub weight: Option<JsonInt>,
}

// ---- version structs, Molang walk, rules -------------------------------------------------------

/// The format of `single_block_feature`'s newest schema.
pub(crate) const SINGLE_BLOCK_FORMAT: FormatVersion = FormatVersion::new(1, 21, 40);

/// The format floor of `multi_block_feature` and `multipart_block_column_feature`, which are
/// registered at this format.
pub(crate) const MULTI_BLOCK_FORMAT: FormatVersion = known::V1_26_40;

/// The setter texts of the named members of the block feature types.
const GROWTH_DIRECTION: (&str, &str) = (
    "growth_direction",
    "Bad value for growth_direction - should be 'UP' or 'DOWN'",
);
const EXPOSED_FACE: (&str, &str) = (
    "exposed_face",
    "Bad value for exposed_face - should be 'up' or 'down'",
);
const VEGETATION_SURFACE: (&str, &str) = (
    "surface",
    "Bad value for surface - should be 'ceiling' or 'floor'",
);

/// `impl Versioned + VersionStruct` with the setter texts of named members.
macro_rules! named_struct {
    ($ty:ty, $($text:expr),+) => {
        impl Versioned for $ty {
            const VERSION: FormatVersion = V1_13_0;
        }

        impl VersionStruct for $ty {
            fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
                parse_strict_with(payload, cx, d, &[$($text),+])
            }

            fn write(&self) -> Result<Value, EncodeError> {
                write_payload(self)
            }
        }
    };
}

named_struct!(GrowingPlantFeature, GROWTH_DIRECTION);
named_struct!(PartiallyExposedBlobFeature, EXPOSED_FACE);
named_struct!(VegetationPatchFeature, VEGETATION_SURFACE);

impl Versioned for SingleBlockFeature {
    const VERSION: FormatVersion = SINGLE_BLOCK_FORMAT;
}

impl VersionStruct for SingleBlockFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        super::read::parse_strict(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

walk!(fields OreFeature { "replace_rules" => replace_rules });
walk!(ReplaceRule { "places_block" => places_block, "may_replace" => may_replace });
walk!(fields SingleBlockFeature {
    "places_block" => places_block,
    "may_attach_to" => may_attach_to,
    "may_not_attach_to" => may_not_attach_to,
    "may_replace" => may_replace,
});
walk!(AttachConditions {
    "top" => top,
    "bottom" => bottom,
    "north" => north,
    "east" => east,
    "south" => south,
    "west" => west,
    "all" => all,
    "sides" => sides,
    "diagonal" => diagonal,
});
walk!(fields MultiBlockFeature { "places_block" => places_block, "may_replace" => may_replace });
walk!(fields MultifaceFeature { "places_block" => places_block, "can_place_on" => can_place_on });
walk!(fields FossilFeature { "ore_block" => ore_block });
walk!(fields GeodeFeature {
    "filler" => filler,
    "inner_layer" => inner_layer,
    "alternate_inner_layer" => alternate_inner_layer,
    "middle_layer" => middle_layer,
    "outer_layer" => outer_layer,
    "inner_placements" => inner_placements,
});
walk!(fields GrowingPlantFeature { "body_blocks" => body_blocks, "head_blocks" => head_blocks });
walk!(fields PartiallyExposedBlobFeature { "places_block" => places_block });
walk!(fields SculkPatchFeature {
    "can_place_sculk_patch_on" => can_place_sculk_patch_on,
    "central_block" => central_block,
});
// The list after `+`: the member that names a feature.
walk!(fields VegetationPatchFeature {
    "replaceable_blocks" => replaceable_blocks,
    "ground_block" => ground_block,
} + { "vegetation_feature" => vegetation_feature });
walk!(fields HorizontalTreeDecorationFeature { "places_block" => places_block });
walk!(fields MultipartBlockColumnFeature {
    "tip_block" => tip_block,
    "frustum_block" => frustum_block,
    "middle_block" => middle_block,
    "base_block" => base_block,
    "may_place_on" => may_place_on,
    "may_replace" => may_replace,
});

impl Walk for PlacesBlock {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        if let PlacesBlock::Descriptor(descriptor) = self {
            descriptor.walk(v);
        }
    }
}

impl super::molang::Refer for PlacesBlock {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        match self {
            PlacesBlock::Specifier(specifier) => specifier.refer(out),
            PlacesBlock::Descriptor(descriptor) => descriptor.refer(out),
            PlacesBlock::Weighted(entries) => {
                for (index, entry) in entries.iter().enumerate() {
                    out.element(index, entry);
                }
            }
        }
    }
}

impl super::molang::Refer for WeightedSpecifier {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        out.member("block", &self.block);
    }
}

impl Rules for OreFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.int_min("count", Some(self.count), 1);
        check.check_float(
            "discard_chance_on_air_exposure",
            self.discard_chance_on_air_exposure,
            0.0,
            1.0,
        );
        if let Some(rules) = &self.replace_rules {
            check.check_len("replace_rules", Some(rules.len()), 1, None);
            check.each("replace_rules", rules, |check, rule| {
                check.descriptor("places_block", Some(&rule.places_block));
                // The array's minimum length is one.
                check.check_len(
                    "may_replace",
                    rule.may_replace.as_ref().map(Vec::len),
                    1,
                    None,
                );
                check.descriptors("may_replace", rule.may_replace.as_deref());
            });
        }
    }
}

/// The attach-condition rules, at the current path.
pub(super) fn attach_rules(check: &mut ValidateCx<'_>, conditions: &AttachConditions) {
    check.check_int(
        "min_sides_must_attach",
        conditions.min_sides_must_attach,
        1,
        4,
    );
    for (key, list) in [
        ("top", &conditions.top),
        ("bottom", &conditions.bottom),
        ("north", &conditions.north),
        ("east", &conditions.east),
        ("south", &conditions.south),
        ("west", &conditions.west),
        ("all", &conditions.all),
        ("sides", &conditions.sides),
        ("diagonal", &conditions.diagonal),
    ] {
        check.list(key, list.as_ref());
    }
}

impl Rules for SingleBlockFeature {
    /// A descriptor `places_block` (an upgraded 1.13.0 document) has no 1.21.40 form: written
    /// at 1.13.0 when nothing of 1.21.40 is set, so it reads back unchanged.
    fn older_form(&self) -> Option<Result<(FormatVersion, Value), EncodeError>> {
        use crate::history::v1_13_0;
        let Some(PlacesBlock::Descriptor(descriptor)) = &self.places_block else {
            return None;
        };
        let newer = self.randomize_rotation.is_some()
            || self.may_not_attach_to.is_some()
            || self
                .may_attach_to
                .as_ref()
                .is_some_and(|attach| attach.diagonal.is_some());
        if newer {
            return None;
        }
        let older = v1_13_0::SingleBlockFeature {
            description: self.description.clone(),
            places_block: Some(descriptor.clone()),
            enforce_placement_rules: self.enforce_placement_rules,
            enforce_survivability_rules: self.enforce_survivability_rules,
            may_attach_to: self
                .may_attach_to
                .clone()
                .map(|attach| v1_13_0::AttachConditions {
                    min_sides_must_attach: attach.min_sides_must_attach,
                    auto_rotate: attach.auto_rotate,
                    top: attach.top,
                    bottom: attach.bottom,
                    north: attach.north,
                    east: attach.east,
                    south: attach.south,
                    west: attach.west,
                    all: attach.all,
                    sides: attach.sides,
                }),
            may_replace: self.may_replace.clone(),
        };
        Some(older.write().map(|payload| (v1_13_0::VERSION, payload)))
    }

    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        match &self.places_block {
            Some(PlacesBlock::Specifier(specifier)) => {
                let at = check.path("places_block");
                check
                    .diagnostics
                    .extend(specifier.check(&at, Severity::Warning));
            }
            Some(PlacesBlock::Descriptor(descriptor)) => {
                check.descriptor("places_block", Some(descriptor));
            }
            Some(PlacesBlock::Weighted(entries)) => {
                check.each("places_block", entries, |check, entry| {
                    let at = check.path("block");
                    check
                        .diagnostics
                        .extend(entry.block.check(&at, Severity::Warning));
                });
            }
            None => {}
        }
        check.opt("may_attach_to", self.may_attach_to.as_ref(), attach_rules);
        check.opt(
            "may_not_attach_to",
            self.may_not_attach_to.as_ref(),
            attach_rules,
        );
        check.descriptors("may_replace", self.may_replace.as_deref());
    }
}

impl Rules for MultiBlockFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptor("places_block", Some(&self.places_block));
        check.descriptors("may_replace", self.may_replace.as_deref());
    }
}

impl Rules for MultifaceFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptor("places_block", Some(&self.places_block));
        check.check_int("search_range", Some(self.search_range), 1, 64);
        check.check_float(
            "chance_of_spreading",
            Some(self.chance_of_spreading),
            0.0,
            1.0,
        );
        check.check_len(
            "can_place_on",
            self.can_place_on.as_ref().map(Vec::len),
            1,
            None,
        );
        check.descriptors("can_place_on", self.can_place_on.as_deref());
    }
}

impl Rules for FossilFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptor("ore_block", Some(&self.ore_block));
    }
}

impl Rules for GeodeFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        for (key, descriptor) in [
            ("filler", &self.filler),
            ("inner_layer", &self.inner_layer),
            ("alternate_inner_layer", &self.alternate_inner_layer),
            ("middle_layer", &self.middle_layer),
            ("outer_layer", &self.outer_layer),
        ] {
            check.descriptor(key, Some(descriptor));
        }
        let placements = self.inner_placements.as_deref();
        check.check_len("inner_placements", placements.map(<[_]>::len), 1, None);
        check.descriptors("inner_placements", placements);
        for (key, value, min, max) in [
            (
                "min_outer_wall_distance",
                self.min_outer_wall_distance,
                1,
                10,
            ),
            (
                "max_outer_wall_distance",
                self.max_outer_wall_distance,
                1,
                20,
            ),
            (
                "min_distribution_points",
                self.min_distribution_points,
                1,
                10,
            ),
            (
                "max_distribution_points",
                self.max_distribution_points,
                1,
                20,
            ),
            ("min_point_offset", self.min_point_offset, 0, 10),
            ("max_point_offset", self.max_point_offset, 0, 10),
            ("crack_point_offset", self.crack_point_offset, 0, 10),
        ] {
            check.check_int(key, Some(value), min, max);
        }
        for (key, value, max) in [
            ("generate_crack_chance", self.generate_crack_chance, 1.0),
            ("base_crack_size", self.base_crack_size, 5.0),
            (
                "use_potential_placements_chance",
                self.use_potential_placements_chance,
                1.0,
            ),
            (
                "use_alternate_layer0_chance",
                self.use_alternate_layer0_chance,
                1.0,
            ),
        ] {
            check.check_float(key, Some(value), 0.0, max);
        }
    }
}

impl Rules for GrowingPlantFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.check_len(
            "height_distribution",
            Some(self.height_distribution.len()),
            1,
            None,
        );
        check.each(
            "height_distribution",
            &self.height_distribution,
            |check, entry| {
                let at = check.path("0");
                check.diagnostics.extend(entry.height.check(&at));
            },
        );
        check.range("age", self.age.as_ref(), i32::MIN, i32::MAX);
        for (key, blocks) in [
            ("body_blocks", &self.body_blocks),
            ("head_blocks", &self.head_blocks),
        ] {
            check.check_len(key, Some(blocks.len()), 1, None);
            check.each(key, blocks, |check, entry| {
                check.descriptor("0", Some(&entry.block));
            });
        }
    }
}

impl Rules for PartiallyExposedBlobFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.check_int(
            "placement_radius_around_floor",
            Some(self.placement_radius_around_floor),
            1,
            8,
        );
        check.check_float(
            "placement_probability_per_valid_position",
            Some(self.placement_probability_per_valid_position),
            0.0,
            1.0,
        );
        check.descriptor("places_block", Some(&self.places_block));
    }
}

impl Rules for SculkPatchFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptors(
            "can_place_sculk_patch_on",
            Some(&self.can_place_sculk_patch_on),
        );
        check.descriptor("central_block", self.central_block.as_ref());
        check.check_float(
            "central_block_placement_chance",
            self.central_block_placement_chance,
            0.0,
            1.0,
        );
        for (key, value, min, max) in [
            ("cursor_count", self.cursor_count, 0, 32),
            ("charge_amount", self.charge_amount, 1, 1000),
            ("spread_attempts", self.spread_attempts, 1, 64),
            ("growth_rounds", self.growth_rounds, 0, 8),
            ("spread_rounds", self.spread_rounds, 0, 8),
        ] {
            check.check_int(key, Some(value), min, max);
        }
        check.range(
            "extra_growth_chance",
            self.extra_growth_chance.as_ref(),
            i32::MIN,
            i32::MAX,
        );
    }
}

impl Rules for VegetationPatchFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptors("replaceable_blocks", Some(&self.replaceable_blocks));
        check.descriptor("ground_block", Some(&self.ground_block));
        check.range("depth", Some(&self.depth), i32::MIN, i32::MAX);
        check.range(
            "horizontal_radius",
            Some(&self.horizontal_radius),
            i32::MIN,
            i32::MAX,
        );
        check.check_float(
            "extra_edge_column_chance",
            self.extra_edge_column_chance,
            0.0,
            f32::MAX,
        );
    }
}

/// The engine that adds `horizontal_tree_decoration_feature`.
pub(crate) const HORIZONTAL_TREE_DECORATION_ENGINE: FormatVersion = known::V1_26_40;

impl Rules for HorizontalTreeDecorationFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.descriptor("places_block", Some(&self.places_block));
    }
}

// Registered together with `multi_block_feature`: format floor and engine 1.26.40.
impl Rules for MultipartBlockColumnFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        for (key, descriptor) in [
            ("tip_block", &self.tip_block),
            ("frustum_block", &self.frustum_block),
            ("middle_block", &self.middle_block),
            ("base_block", &self.base_block),
        ] {
            check.descriptor(key, Some(descriptor));
        }
        check.range(
            "height_range",
            self.height_range.as_ref(),
            i32::MIN,
            i32::MAX,
        );
        check.descriptors("may_place_on", self.may_place_on.as_deref());
        check.descriptors("may_replace", self.may_replace.as_deref());
    }
}

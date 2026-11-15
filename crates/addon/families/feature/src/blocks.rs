//! Feature types that place blocks: ores, single and multi-part blocks, blobs, geodes, plants,
//! patches and columns.

use facet::Facet;
use facet_value::{VArray, VObject};

use super::Description;
use super::V1_13_0;
use super::codes;
use super::molang::{DESCRIPTOR_TAGS, refers};
use super::strict_struct;
use super::values::{BlockList, JsonInt, WeightedBlock, pair, weight_of, weight_value};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonType, StrictError, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx, SchemaFields, from_value};
use bedrock_addon_core::shared::IntRange;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::VersionStruct;
use bedrock_addon_shared_shapes::{BlockDescriptor, BlockSpecifier};

// ---- ore --------------------------------------------------------------------------------------

/// `minecraft:ore_feature`: a vein of blocks that replace what they may replace.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct OreFeature {
    /// JSON `description`.
    pub description: Description,
    /// How many blocks the vein has, at least one (JSON `count`).
    #[facet(schema::min = 1)]
    pub count: JsonInt,
    /// The chance, in `[0, 1]`, that a block exposed to air is not placed (JSON
    /// `discard_chance_on_air_exposure`); absent is 0.
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub discard_chance_on_air_exposure: Option<f32>,
    /// Which block replaces which; at least one rule (JSON `replace_rules`).
    #[facet(schema::min_len = 1)]
    pub replace_rules: Option<Vec<ReplaceRule>>,
}

/// One entry of `ore_feature.replace_rules`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ReplaceRule {
    /// The block placed (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// The blocks it may replace, at least one; absent replaces any block (JSON
    /// `may_replace`).
    #[facet(schema::min_len = 1)]
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

// ---- single block -----------------------------------------------------------------------------

/// `minecraft:single_block_feature`: one block, placed if the attach and replace rules allow it.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AttachConditions {
    /// How many sides must attach, 1 to 4 (JSON `min_sides_must_attach`); absent is 4.
    #[facet(schema::min = 1, schema::max = 4)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct MultifaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block (JSON `places_block`).
    pub places_block: BlockDescriptor,
    /// How far it searches, 1 to 64 (JSON `search_range`).
    #[facet(schema::min = 1, schema::max = 64)]
    pub search_range: JsonInt,
    /// JSON `can_place_on_floor`.
    pub can_place_on_floor: bool,
    /// JSON `can_place_on_ceiling`.
    pub can_place_on_ceiling: bool,
    /// JSON `can_place_on_wall`.
    pub can_place_on_wall: bool,
    /// The chance, in `[0, 1]`, that it spreads (JSON `chance_of_spreading`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub chance_of_spreading: f32,
    /// The blocks it may be placed on, at least one (JSON `can_place_on`).
    #[facet(schema::min_len = 1)]
    pub can_place_on: Option<Vec<BlockDescriptor>>,
}

/// `minecraft:fossil_feature`: a fossil structure with ore blocks.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct FossilFeature {
    /// JSON `description`.
    pub description: Description,
    /// The ore block (JSON `ore_block`).
    pub ore_block: BlockDescriptor,
    /// How many corners may be empty (JSON `max_empty_corners`).
    pub max_empty_corners: JsonInt,
}

/// `minecraft:geode_feature`: a layered geode.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
    #[facet(schema::min_len = 1)]
    pub inner_placements: Option<Vec<BlockDescriptor>>,
    /// 1 to 10 (JSON `min_outer_wall_distance`).
    #[facet(schema::min = 1, schema::max = 10)]
    pub min_outer_wall_distance: JsonInt,
    /// 1 to 20 (JSON `max_outer_wall_distance`).
    #[facet(schema::min = 1, schema::max = 20)]
    pub max_outer_wall_distance: JsonInt,
    /// 1 to 10 (JSON `min_distribution_points`).
    #[facet(schema::min = 1, schema::max = 10)]
    pub min_distribution_points: JsonInt,
    /// 1 to 20 (JSON `max_distribution_points`).
    #[facet(schema::min = 1, schema::max = 20)]
    pub max_distribution_points: JsonInt,
    /// 0 to 10 (JSON `min_point_offset`).
    #[facet(schema::min = 0, schema::max = 10)]
    pub min_point_offset: JsonInt,
    /// 0 to 10 (JSON `max_point_offset`).
    #[facet(schema::min = 0, schema::max = 10)]
    pub max_point_offset: JsonInt,
    /// JSON `max_radius`.
    pub max_radius: JsonInt,
    /// 0 to 10 (JSON `crack_point_offset`).
    #[facet(schema::min = 0, schema::max = 10)]
    pub crack_point_offset: JsonInt,
    /// In `[0, 1]` (JSON `generate_crack_chance`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub generate_crack_chance: f32,
    /// In `[0, 5]` (JSON `base_crack_size`).
    #[facet(schema::min = 0.0, schema::max = 5.0)]
    pub base_crack_size: f32,
    /// JSON `noise_multiplier`.
    pub noise_multiplier: f32,
    /// In `[0, 1]` (JSON `use_potential_placements_chance`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub use_potential_placements_chance: f32,
    /// In `[0, 1]` (JSON `use_alternate_layer0_chance`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub use_alternate_layer0_chance: f32,
    /// JSON `placements_require_layer0_alternate`.
    pub placements_require_layer0_alternate: bool,
    /// JSON `invalid_blocks_threshold`.
    pub invalid_blocks_threshold: JsonInt,
}

// ---- growing plant -----------------------------------------------------------------------------

/// `minecraft:growing_plant_feature`: a plant column of body blocks ending in a head block.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct GrowingPlantFeature {
    /// JSON `description`.
    pub description: Description,
    /// Heights and their weights, at least one (JSON `height_distribution`).
    #[facet(schema::min_len = 1)]
    pub height_distribution: Vec<HeightWeight>,
    /// Which way it grows (JSON `growth_direction`).
    #[facet(schema::invalid = codes::GROWTH_DIRECTION)]
    pub growth_direction: GrowthDirection,
    /// The age state of the head block (JSON `age`).
    pub age: Option<IntRange>,
    /// The body blocks and their weights, at least one (JSON `body_blocks`).
    #[facet(schema::min_len = 1)]
    pub body_blocks: Vec<WeightedBlock>,
    /// The head blocks and their weights, at least one (JSON `head_blocks`).
    #[facet(schema::min_len = 1)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct PartiallyExposedBlobFeature {
    /// JSON `description`.
    pub description: Description,
    /// 1 to 8 (JSON `placement_radius_around_floor`).
    #[facet(schema::min = 1, schema::max = 8)]
    pub placement_radius_around_floor: JsonInt,
    /// In `[0, 1]` (JSON `placement_probability_per_valid_position`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub placement_probability_per_valid_position: f32,
    /// The exposed face (JSON `exposed_face`); absent is `up`.
    #[facet(schema::invalid = codes::EXPOSED_FACE)]
    pub exposed_face: Option<ExposedFace>,
    /// The block (JSON `places_block`).
    pub places_block: BlockDescriptor,
}

/// `partially_exposed_blob_feature.exposed_face`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct SculkPatchFeature {
    /// JSON `description`.
    pub description: Description,
    /// The blocks it may be placed on (JSON `can_place_sculk_patch_on`).
    pub can_place_sculk_patch_on: Vec<BlockDescriptor>,
    /// The block at the centre (JSON `central_block`).
    pub central_block: Option<BlockDescriptor>,
    /// In `[0, 1]` (JSON `central_block_placement_chance`).
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub central_block_placement_chance: Option<f32>,
    /// 0 to 32 (JSON `cursor_count`).
    #[facet(schema::min = 0, schema::max = 32)]
    pub cursor_count: JsonInt,
    /// 1 to 1000 (JSON `charge_amount`).
    #[facet(schema::min = 1, schema::max = 1000)]
    pub charge_amount: JsonInt,
    /// 1 to 64 (JSON `spread_attempts`).
    #[facet(schema::min = 1, schema::max = 64)]
    pub spread_attempts: JsonInt,
    /// 0 to 8 (JSON `growth_rounds`).
    #[facet(schema::min = 0, schema::max = 8)]
    pub growth_rounds: JsonInt,
    /// 0 to 8 (JSON `spread_rounds`).
    #[facet(schema::min = 0, schema::max = 8)]
    pub spread_rounds: JsonInt,
    /// JSON `extra_growth_chance`.
    pub extra_growth_chance: Option<IntRange>,
}

/// `minecraft:vegetation_patch_feature`: a patch of ground block with vegetation on it.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
    #[facet(schema::invalid = codes::VEGETATION_SURFACE)]
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
    #[facet(schema::min = 0.0)]
    pub extra_edge_column_chance: Option<f32>,
    /// JSON `waterlogged`.
    pub waterlogged: Option<bool>,
}

/// `vegetation_patch_feature.surface`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct WeightedHeight {
    /// JSON `value`.
    pub value: Option<JsonInt>,
    /// JSON `weight`.
    pub weight: Option<JsonInt>,
}

// ---- version structs, references, the older form ----------------------------------------------

/// The format of `single_block_feature`'s newest schema.
pub(crate) const SINGLE_BLOCK_FORMAT: FormatVersion = FormatVersion::new(1, 21, 40);

/// The format floor of `multi_block_feature` and `multipart_block_column_feature`, which are
/// registered at this format.
pub(crate) const MULTI_BLOCK_FORMAT: FormatVersion = known::V1_26_40;

/// The engine that adds `horizontal_tree_decoration_feature`.
pub(crate) const HORIZONTAL_TREE_DECORATION_ENGINE: FormatVersion = known::V1_26_40;

strict_struct!(GrowingPlantFeature, V1_13_0);
strict_struct!(PartiallyExposedBlobFeature, V1_13_0);
strict_struct!(VegetationPatchFeature, V1_13_0);
strict_struct!(SingleBlockFeature, SINGLE_BLOCK_FORMAT);

refers!(OreFeature { "replace_rules" => replace_rules });
refers!(ReplaceRule { "places_block" => places_block, "may_replace" => may_replace });
refers!(SingleBlockFeature {
    "places_block" => places_block,
    "may_attach_to" => may_attach_to,
    "may_not_attach_to" => may_not_attach_to,
    "may_replace" => may_replace,
});
refers!(AttachConditions {
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
refers!(MultiBlockFeature { "places_block" => places_block, "may_replace" => may_replace });
refers!(MultifaceFeature { "places_block" => places_block, "can_place_on" => can_place_on });
refers!(FossilFeature { "ore_block" => ore_block });
refers!(GeodeFeature {
    "filler" => filler,
    "inner_layer" => inner_layer,
    "alternate_inner_layer" => alternate_inner_layer,
    "middle_layer" => middle_layer,
    "outer_layer" => outer_layer,
    "inner_placements" => inner_placements,
});
refers!(GrowingPlantFeature { "body_blocks" => body_blocks, "head_blocks" => head_blocks });
refers!(PartiallyExposedBlobFeature { "places_block" => places_block });
refers!(SculkPatchFeature {
    "can_place_sculk_patch_on" => can_place_sculk_patch_on,
    "central_block" => central_block,
});
// The member after the blocks: the one that names a feature.
refers!(VegetationPatchFeature {
    "replaceable_blocks" => replaceable_blocks,
    "ground_block" => ground_block,
    "vegetation_feature" => vegetation_feature,
});
refers!(HorizontalTreeDecorationFeature { "places_block" => places_block });
refers!(MultipartBlockColumnFeature {
    "tip_block" => tip_block,
    "frustum_block" => frustum_block,
    "middle_block" => middle_block,
    "base_block" => base_block,
    "may_place_on" => may_place_on,
    "may_replace" => may_replace,
});

// Read as the game's reader of the shape reads them (`TryFrom<Value>`), not by the slice's
// scalar rules; a refusal is one wrong type at the member.
impl FromJson for PlacesBlock {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        from_value(node, read, Self::try_from)
    }
}

impl FromJson for HeightWeight {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        from_value(node, read, Self::try_from)
    }
}
bedrock_addon_core::impl_no_molang!(HeightWeight);

/// A specifier or descriptor is walked at the member; a weighted list's blocks at
/// `places_block/<index>/block`.
impl SchemaFields for PlacesBlock {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        match self {
            Self::Specifier(specifier) => specifier.check_fields(cx),
            Self::Descriptor(descriptor) => descriptor.check_fields(cx),
            Self::Weighted(entries) => {
                for (index, entry) in entries.iter_mut().enumerate() {
                    cx.enter_index(index);
                    cx.enter("block");
                    entry.block.check_fields(cx);
                    cx.leave();
                    cx.leave();
                }
            }
        }
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

impl MolangFields for PlacesBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Descriptor(descriptor) = self {
            descriptor.visit_molang(visitor);
        }
    }
}

/// The range is the element at position 0.
impl SchemaFields for HeightWeight {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        cx.enter_index(0);
        self.height.check_fields(cx);
        cx.leave();
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
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

/// A descriptor `places_block` (an upgraded 1.13.0 document) has no 1.21.40 form: written at
/// 1.13.0 when nothing of 1.21.40 is set, so it reads back unchanged.
pub(crate) fn older_form(
    feature: &SingleBlockFeature,
) -> Option<Result<(FormatVersion, Value), EncodeError>> {
    use crate::history::v1_13_0;
    let Some(PlacesBlock::Descriptor(descriptor)) = &feature.places_block else {
        return None;
    };
    let newer = feature.randomize_rotation.is_some()
        || feature.may_not_attach_to.is_some()
        || feature
            .may_attach_to
            .as_ref()
            .is_some_and(|attach| attach.diagonal.is_some());
    if newer {
        return None;
    }
    let older = v1_13_0::SingleBlockFeature {
        description: feature.description.clone(),
        places_block: Some(descriptor.clone()),
        enforce_placement_rules: feature.enforce_placement_rules,
        enforce_survivability_rules: feature.enforce_survivability_rules,
        may_attach_to: feature
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
        may_replace: feature.may_replace.clone(),
    };
    Some(older.write().map(|payload| (v1_13_0::VERSION, payload)))
}

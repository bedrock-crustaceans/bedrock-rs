//! Feature types placed against the terrain: structure templates, beards and shavers, and
//! snapping to a surface.

use facet::Facet;

use super::Description;
use super::codes;
use super::molang::{DESCRIPTOR_TAGS, refers};
use super::read::required_child;
use super::strict_struct;
use super::values::JsonInt;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_shared_shapes::BlockDescriptor;

/// The engine whose schema bounds `leveled.max_steepness` to `[0, 8]`.
const ENGINE_1_26_50: FormatVersion = known::V1_26_50;

/// The format of `snap_to_surface_feature`'s newest schema, which runs from 1.26.50 to the
/// current version.
pub(crate) const SNAP_FORMAT: FormatVersion = known::V1_26_50;

// ---- structure template ------------------------------------------------------------------------

/// `minecraft:structure_template_feature`: places a structure file where its constraints hold.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct StructureTemplateFeature {
    /// JSON `description`.
    pub description: Description,
    /// The structure, by its name (JSON `structure_name`).
    pub structure_name: String,
    /// How far the position may move to satisfy the constraints, 0 to 16 (JSON
    /// `adjustment_radius`); absent is 0.
    #[facet(schema::min = 0, schema::max = 16)]
    pub adjustment_radius: Option<JsonInt>,
    /// The structure's rotation (JSON `facing_direction`); absent is `random`.
    pub facing_direction: Option<FacingDirection>,
    /// Whether the structure rotates around its centre (JSON `rotate_around_center`, engine
    /// 1.26.20); absent is `false`.
    #[facet(schema::since_engine = "1.26.20")]
    pub rotate_around_center: Option<bool>,
    /// The structure's layer that sits at the input position, at least 0 (JSON
    /// `ground_level`, engine 1.26.30); absent is 0.
    #[facet(schema::since_engine = "1.26.30", schema::min = 0)]
    pub ground_level: Option<JsonInt>,
    /// What must hold where the structure is placed (JSON `constraints`).
    pub constraints: StructureConstraints,
}

/// `structure_template_feature.facing_direction`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum FacingDirection {
    /// `north`.
    North,
    /// `south`.
    South,
    /// `east`.
    East,
    /// `west`.
    West,
    /// `random` (the default).
    #[default]
    Random,
}

/// `structure_template_feature.constraints`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct StructureConstraints {
    /// The structure must rest on the ground (JSON `grounded`, an empty object).
    pub grounded: Option<EmptyConstraint>,
    /// The structure must not be buried (JSON `unburied`, an empty object).
    pub unburied: Option<EmptyConstraint>,
    /// The structure may intersect only the listed blocks (JSON `block_intersection`).
    pub block_intersection: Option<BlockIntersection>,
    /// The ground below must be level enough (JSON `leveled`, engine 1.26.30).
    #[facet(schema::since_engine = "1.26.30")]
    pub leveled: Option<LeveledConstraint>,
}

/// A constraint without members: JSON `{}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
pub struct EmptyConstraint {}

/// `constraints.block_intersection`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = one_list_is_required)]
pub struct BlockIntersection {
    /// The blocks the structure may intersect (JSON `block_allowlist`). One of this and
    /// `block_whitelist` is required.
    pub block_allowlist: Option<Vec<BlockDescriptor>>,
    /// The older name of `block_allowlist`, kept under the key the author wrote (JSON
    /// `block_whitelist`).
    pub block_whitelist: Option<Vec<BlockDescriptor>>,
    /// Whether only motion-blocking blocks count as intersecting (JSON
    /// `only_check_intersection_for_motion_blocking_blocks`, engine 1.26.30); absent is `true`.
    #[facet(schema::since_engine = "1.26.30")]
    pub only_check_intersection_for_motion_blocking_blocks: Option<bool>,
}

/// `constraints.leveled` (engine 1.26.30).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct LeveledConstraint {
    /// How steep the ground may be, 0 to 8 from engine 1.26.50 (JSON `max_steepness`); absent
    /// is 2.
    #[facet(schema::check_with = max_steepness)]
    pub max_steepness: Option<JsonInt>,
}

// ---- beards and shavers ------------------------------------------------------------------------

/// `minecraft:beards_and_shavers` (base-game packs only): builds terrain up below a feature's
/// box ("beard") and cuts it away above ("shaver").
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
#[facet(schema::rule = beard_setters)]
pub struct BeardsAndShaversFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// `[x, y, z]` (JSON `bounding_box_min`).
    #[facet(schema::min_len = 3, schema::max_len = 3)]
    pub bounding_box_min: Vec<f32>,
    /// `[x, y, z]`, above the minimum (JSON `bounding_box_max`).
    #[facet(schema::min_len = 3, schema::max_len = 3)]
    pub bounding_box_max: Vec<f32>,
    /// JSON `y_delta`.
    pub y_delta: f32,
    /// JSON `surface_block_type`.
    pub surface_block_type: BlockDescriptor,
    /// JSON `subsurface_block_type`.
    pub subsurface_block_type: BlockDescriptor,
    /// In `[0, 1]`, below the maximum (JSON `beard_raggedness_min`).
    pub beard_raggedness_min: Option<f32>,
    /// In `[0, 1]` (JSON `beard_raggedness_max`).
    pub beard_raggedness_max: Option<f32>,
}

// ---- snap to surface ---------------------------------------------------------------------------

/// `minecraft:snap_to_surface_feature`: moves the input position to a surface and places a
/// feature there. Format 1.26.50 renamed `vertical_search_range` to `search_range` (an older
/// document's member is renamed when it loads); every other member, `allow_non_air_placement`
/// and the `wall` surface included, is read at every format.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct SnapToSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed (JSON `feature_to_snap`).
    pub feature_to_snap: FeatureId,
    /// How far it searches for the surface (JSON `search_range`).
    pub search_range: JsonInt,
    /// Which surface (JSON `surface`); absent is `floor`.
    #[facet(schema::invalid = codes::SNAP_SURFACE)]
    pub surface: Option<Surface>,
    /// Whether it may snap to air (JSON `allow_air_placement`); absent is `true`.
    pub allow_air_placement: Option<bool>,
    /// Whether a snap started in a block other than air or water may place (JSON
    /// `allow_non_air_placement`); absent is `false`.
    pub allow_non_air_placement: Option<bool>,
    /// Whether it may snap under water (JSON `allow_underwater_placement`); absent is `false`.
    pub allow_underwater_placement: Option<bool>,
    /// The blocks it may snap to; absent or empty is any block that supports the face (JSON
    /// `allowed_surface_blocks`).
    pub allowed_surface_blocks: Option<Vec<BlockDescriptor>>,
    /// Whether the feature is placed in the surface instead of on it (JSON `embed_in_surface`,
    /// engine 1.26.20); absent is `false`.
    #[facet(schema::since_engine = "1.26.20")]
    pub embed_in_surface: Option<bool>,
}

/// `snap_to_surface_feature.surface`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Surface {
    /// `ceiling`.
    Ceiling,
    /// `floor` (the default).
    #[default]
    Floor,
    /// `random_horizontal`.
    RandomHorizontal,
    /// `wall`.
    Wall,
}

// ---- version structs, references, rules --------------------------------------------------------

strict_struct!(SnapToSurfaceFeature, SNAP_FORMAT);

impl BlockIntersection {
    /// The blocks the structure may intersect, under either name (`block_allowlist` first).
    pub fn blocks(&self) -> &[BlockDescriptor] {
        self.block_allowlist
            .as_deref()
            .or(self.block_whitelist.as_deref())
            .unwrap_or_default()
    }
}

refers!(StructureTemplateFeature { "constraints" => constraints });
refers!(StructureConstraints { "block_intersection" => block_intersection });
refers!(BlockIntersection {
    "block_allowlist" => block_allowlist,
    "block_whitelist" => block_whitelist,
});
refers!(BeardsAndShaversFeature {
    "surface_block_type" => surface_block_type,
    "subsurface_block_type" => subsurface_block_type,
    "places_feature" => places_feature,
});
refers!(SnapToSurfaceFeature {
    "allowed_surface_blocks" => allowed_surface_blocks,
    "feature_to_snap" => feature_to_snap,
});

/// The bound of `leveled.max_steepness` is the schema of engine 1.26.50 on.
fn max_steepness(steepness: &mut JsonInt, cx: &mut SchemaCx<'_>) {
    if cx
        .engine_version()
        .is_some_and(|engine| engine >= ENGINE_1_26_50)
    {
        cx.within("", steepness.0, Some(0), Some(8));
    }
}

/// The node `block_allowlist|block_whitelist` is required.
fn one_list_is_required(intersection: &mut BlockIntersection, cx: &mut SchemaCx<'_>) {
    if intersection.block_allowlist.is_none() && intersection.block_whitelist.is_none() {
        let at = cx.path().child("block_allowlist");
        cx.push(required_child(at));
    }
}

/// The client logs these at level 3 and keeps the values: kept errors.
fn beard_setters(beard: &mut BeardsAndShaversFeature, cx: &mut SchemaCx<'_>) {
    let below = beard
        .bounding_box_min
        .iter()
        .zip(&beard.bounding_box_max)
        .any(|(min, max)| max <= min);
    if below {
        cx.kept("bounding_box_max", Code::from(&codes::BOUNDING_BOX));
    }
    let unit = |v: Option<f32>| v.is_none_or(|v| (0.0..=1.0).contains(&v));
    if !unit(beard.beard_raggedness_min) || !unit(beard.beard_raggedness_max) {
        cx.kept("beard_raggedness_min", Code::from(&codes::RAGGED_RANGE));
    } else if let (Some(min), Some(max)) = (beard.beard_raggedness_min, beard.beard_raggedness_max)
        && min >= max
    {
        cx.kept("beard_raggedness_min", Code::from(&codes::RAGGED_ORDER));
    }
}

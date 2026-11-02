//! Feature types placed against the terrain: structure templates, beards and shavers, and
//! snapping to a surface.

use facet::Facet;

use super::Description;
use super::load::{Rules, V1_13_0, strict_struct};
use super::parse::parse_strict_with;
use super::validate::Check;
use super::values::JsonInt;
use super::walk::walk;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, VanillaText};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, StrictBool, StrictFloat, StrictString, Value,
};
use bedrock_addon_core::shared::BlockDescriptor;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

/// The engine that adds `rotate_around_center` and `embed_in_surface`.
pub(crate) const ENGINE_1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);
/// The engine that adds `ground_level`, `leveled` and
/// `only_check_intersection_for_motion_blocking_blocks`.
const ENGINE_1_26_30: FormatVersion = FormatVersion::new(1, 26, 30);
/// The engine whose schema bounds `leveled.max_steepness` to `[0, 8]`.
const ENGINE_1_26_50: FormatVersion = FormatVersion::new(1, 26, 50);

/// The format of `snap_to_surface_feature`'s newest schema, which runs from 1.26.50 to the
/// current version.
pub(crate) const SNAP_FORMAT: FormatVersion = FormatVersion::new(1, 26, 50);

// ---- structure template ------------------------------------------------------------------------

/// `minecraft:structure_template_feature`: places a structure file where its constraints hold.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct StructureTemplateFeature {
    /// JSON `description`.
    pub description: Description,
    /// The structure, by its name (JSON `structure_name`).
    pub structure_name: StrictString,
    /// How far the position may move to satisfy the constraints, 0 to 16 (JSON
    /// `adjustment_radius`); absent is 0.
    #[facet(default)]
    pub adjustment_radius: Option<JsonInt>,
    /// The structure's rotation (JSON `facing_direction`); absent is `random`.
    #[facet(default)]
    pub facing_direction: Option<FacingDirection>,
    /// Whether the structure rotates around its centre (JSON `rotate_around_center`, engine
    /// 1.26.20); absent is `false`.
    #[facet(default)]
    pub rotate_around_center: Option<StrictBool>,
    /// The structure's layer that sits at the input position, at least 0 (JSON
    /// `ground_level`, engine 1.26.30); absent is 0.
    #[facet(default)]
    pub ground_level: Option<JsonInt>,
    /// What must hold where the structure is placed (JSON `constraints`).
    pub constraints: StructureConstraints,
}

/// `structure_template_feature.facing_direction`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct StructureConstraints {
    /// The structure must rest on the ground (JSON `grounded`, an empty object).
    #[facet(default)]
    pub grounded: Option<EmptyConstraint>,
    /// The structure must not be buried (JSON `unburied`, an empty object).
    #[facet(default)]
    pub unburied: Option<EmptyConstraint>,
    /// The structure may intersect only the listed blocks (JSON `block_intersection`).
    #[facet(default)]
    pub block_intersection: Option<BlockIntersection>,
    /// The ground below must be level enough (JSON `leveled`, engine 1.26.30).
    #[facet(default)]
    pub leveled: Option<LeveledConstraint>,
}

/// A constraint without members: JSON `{}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
pub struct EmptyConstraint {}

/// `constraints.block_intersection`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BlockIntersection {
    /// The blocks the structure may intersect (JSON `block_allowlist`). One of this and
    /// `block_whitelist` is required.
    #[facet(default)]
    pub block_allowlist: Option<Vec<BlockDescriptor>>,
    /// The older name of `block_allowlist`, kept under the key the author wrote (JSON
    /// `block_whitelist`).
    #[facet(default)]
    pub block_whitelist: Option<Vec<BlockDescriptor>>,
    /// Whether only motion-blocking blocks count as intersecting (JSON
    /// `only_check_intersection_for_motion_blocking_blocks`, engine 1.26.30); absent is `true`.
    #[facet(default)]
    pub only_check_intersection_for_motion_blocking_blocks: Option<StrictBool>,
}

/// `constraints.leveled` (engine 1.26.30).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LeveledConstraint {
    /// How steep the ground may be, 0 to 8 from engine 1.26.50 (JSON `max_steepness`); absent
    /// is 2.
    #[facet(default)]
    pub max_steepness: Option<JsonInt>,
}

// ---- beards and shavers ------------------------------------------------------------------------

/// `minecraft:beards_and_shavers` (base-game packs only): builds terrain up below a feature's
/// box ("beard") and cuts it away above ("shaver").
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BeardsAndShaversFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// `[x, y, z]` (JSON `bounding_box_min`).
    pub bounding_box_min: Vec<StrictFloat<f32>>,
    /// `[x, y, z]`, above the minimum (JSON `bounding_box_max`).
    pub bounding_box_max: Vec<StrictFloat<f32>>,
    /// JSON `y_delta`.
    pub y_delta: StrictFloat<f32>,
    /// JSON `surface_block_type`.
    pub surface_block_type: BlockDescriptor,
    /// JSON `subsurface_block_type`.
    pub subsurface_block_type: BlockDescriptor,
    /// In `[0, 1]`, below the maximum (JSON `beard_raggedness_min`).
    #[facet(default)]
    pub beard_raggedness_min: Option<StrictFloat<f32>>,
    /// In `[0, 1]` (JSON `beard_raggedness_max`).
    #[facet(default)]
    pub beard_raggedness_max: Option<StrictFloat<f32>>,
}

// ---- snap to surface ---------------------------------------------------------------------------

/// `minecraft:snap_to_surface_feature`: moves the input position to a surface and places a
/// feature there. Format 1.26.50 renamed `vertical_search_range` to `search_range` (an older
/// document's member is renamed when it loads); every other member, `allow_non_air_placement`
/// and the `wall` surface included, is read at every format.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SnapToSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed (JSON `feature_to_snap`).
    pub feature_to_snap: FeatureId,
    /// How far it searches for the surface (JSON `search_range`).
    pub search_range: JsonInt,
    /// Which surface (JSON `surface`); absent is `floor`.
    #[facet(default)]
    pub surface: Option<Surface>,
    /// Whether it may snap to air (JSON `allow_air_placement`); absent is `true`.
    #[facet(default)]
    pub allow_air_placement: Option<StrictBool>,
    /// Whether a snap started in a block other than air or water may place (JSON
    /// `allow_non_air_placement`); absent is `false`.
    #[facet(default)]
    pub allow_non_air_placement: Option<StrictBool>,
    /// Whether it may snap under water (JSON `allow_underwater_placement`); absent is `false`.
    #[facet(default)]
    pub allow_underwater_placement: Option<StrictBool>,
    /// The blocks it may snap to; absent or empty is any block that supports the face (JSON
    /// `allowed_surface_blocks`).
    #[facet(default)]
    pub allowed_surface_blocks: Option<Vec<BlockDescriptor>>,
    /// Whether the feature is placed in the surface instead of on it (JSON `embed_in_surface`,
    /// engine 1.26.20); absent is `false`.
    #[facet(default)]
    pub embed_in_surface: Option<StrictBool>,
}

/// `snap_to_surface_feature.surface`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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

/// The client's text for an unknown `surface`, at every format (the backticks around `wall`
/// are the game's).
pub(crate) const SNAP_SURFACE: (&str, &str) = (
    "surface",
    "Bad value for surface - should be 'ceiling', 'floor', 'random_horizontal', or `wall`",
);

// ---- version structs, Molang walk, rules -------------------------------------------------------

strict_struct!(StructureTemplateFeature, V1_13_0);
strict_struct!(BeardsAndShaversFeature, V1_13_0);

impl Versioned for SnapToSurfaceFeature {
    const VERSION: FormatVersion = SNAP_FORMAT;
}

impl VersionStruct for SnapToSurfaceFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_strict_with(payload, cx, d, &[SNAP_SURFACE])
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

walk!(fields StructureTemplateFeature { "constraints" => constraints });
walk!(StructureConstraints { "block_intersection" => block_intersection });
walk!(BlockIntersection {
    "block_allowlist" => block_allowlist,
    "block_whitelist" => block_whitelist,
});

impl BlockIntersection {
    /// The blocks the structure may intersect, under either name (`block_allowlist` first).
    #[must_use]
    pub fn blocks(&self) -> &[BlockDescriptor] {
        self.block_allowlist
            .as_deref()
            .or(self.block_whitelist.as_deref())
            .unwrap_or_default()
    }
}
// The lists after `+`: the members that name a feature.
walk!(fields BeardsAndShaversFeature {
    "surface_block_type" => surface_block_type,
    "subsurface_block_type" => subsurface_block_type,
} + { "places_feature" => places_feature });
walk!(fields SnapToSurfaceFeature { "allowed_surface_blocks" => allowed_surface_blocks } + {
    "feature_to_snap" => feature_to_snap,
});

impl Rules for StructureTemplateFeature {
    fn rules(&mut self, check: &mut Check<'_>) {
        check.check_int("adjustment_radius", self.adjustment_radius, 0, 16);
        check.since_engine(
            "rotate_around_center",
            self.rotate_around_center.is_some(),
            ENGINE_1_26_20,
        );
        check.since_engine("ground_level", self.ground_level.is_some(), ENGINE_1_26_30);
        check.int_min("ground_level", self.ground_level, 0);
        let engine = check.cx.engine_version;
        check.nested("constraints", |check| {
            let constraints = &self.constraints;
            check.since_engine("leveled", constraints.leveled.is_some(), ENGINE_1_26_30);
            if let Some(leveled) = &constraints.leveled
                && engine >= ENGINE_1_26_50
            {
                check.nested("leveled", |check| {
                    check.check_int("max_steepness", leveled.max_steepness, 0, 8);
                });
            }
            if let Some(intersection) = &constraints.block_intersection {
                check.nested("block_intersection", |check| {
                    check.since_engine(
                        "only_check_intersection_for_motion_blocking_blocks",
                        intersection
                            .only_check_intersection_for_motion_blocking_blocks
                            .is_some(),
                        ENGINE_1_26_30,
                    );
                    // The node `block_allowlist|block_whitelist` is required.
                    if intersection.block_allowlist.is_none()
                        && intersection.block_whitelist.is_none()
                    {
                        let at = check.path("block_allowlist");
                        check.d.push(super::parse::required_child(at));
                    }
                    check.descriptors("block_allowlist", intersection.block_allowlist.as_deref());
                    check.descriptors("block_whitelist", intersection.block_whitelist.as_deref());
                });
            }
        });
    }
}

/// "The max values of the Bounding Box must be greater than the min Values."
const BOUNDING_BOX: &str =
    "The max values of the Bounding Box must be greater than the min Values.";
/// "Ragged Factor values must be [0,1] and min must be less than max."
const RAGGED_RANGE: &str = "Ragged Factor values must be [0,1] and min must be less than max.";
/// "Ragged Factor min must be leess than max" (sic).
const RAGGED_ORDER: &str = "Ragged Factor min must be leess than max";

impl Rules for BeardsAndShaversFeature {
    const INTERNAL: bool = true;

    fn rules(&mut self, check: &mut Check<'_>) {
        check.check_len(
            "bounding_box_min",
            Some(self.bounding_box_min.len()),
            3,
            Some(3),
        );
        check.check_len(
            "bounding_box_max",
            Some(self.bounding_box_max.len()),
            3,
            Some(3),
        );
        check.descriptor("surface_block_type", Some(&self.surface_block_type));
        check.descriptor("subsurface_block_type", Some(&self.subsurface_block_type));
        // The client logs these at level 3 and keeps the values: kept errors.
        let below = self
            .bounding_box_min
            .iter()
            .zip(&self.bounding_box_max)
            .any(|(min, max)| max.0 <= min.0);
        if below {
            check.kept("bounding_box_max", setter(BOUNDING_BOX));
        }
        let unit = |v: Option<StrictFloat<f32>>| v.is_none_or(|v| (0.0..=1.0).contains(&v.0));
        if !unit(self.beard_raggedness_min) || !unit(self.beard_raggedness_max) {
            check.kept("beard_raggedness_min", setter(RAGGED_RANGE));
        } else if let (Some(min), Some(max)) =
            (self.beard_raggedness_min, self.beard_raggedness_max)
            && min.0 >= max.0
        {
            check.kept("beard_raggedness_min", setter(RAGGED_ORDER));
        }
    }
}

/// A setter's message as an `InvalidValue`.
fn setter(text: &'static str) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(text)),
    }
}

impl Rules for SnapToSurfaceFeature {
    fn rules(&mut self, check: &mut Check<'_>) {
        check.since_engine(
            "embed_in_surface",
            self.embed_in_surface.is_some(),
            ENGINE_1_26_20,
        );
        check.descriptors(
            "allowed_surface_blocks",
            self.allowed_surface_blocks.as_deref(),
        );
    }
}

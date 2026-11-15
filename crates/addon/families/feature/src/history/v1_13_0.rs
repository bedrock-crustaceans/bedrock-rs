//! Format 1.13.0 (the initial release) of the three feature types whose schema
//! changed later, and their steps.

use facet::Facet;

use crate::family::HistoryJson;
use crate::molang::{DESCRIPTOR_TAGS, SCATTER_LEGACY};
use crate::{self as latest, BlockList, Description, JsonInt, PlacesBlock, strict_struct};
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{Molang, MolangSlot, MolangVisitor, Step};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, VersionStruct};
use bedrock_addon_shared_scatter::{
    CoordinateEvalOrder, CoordinateRange, ScatterChance, ScatterParams,
};
use bedrock_addon_shared_shapes::BlockDescriptor;

/// The version of the structs of this module.
pub(crate) const VERSION: FormatVersion = FormatVersion::new(1, 13, 0);

// ---- scatter_feature ---------------------------------------------------------------------------

/// `minecraft:scatter_feature` below format 1.21.10: the scatter parameters at the root
/// (the legacy schema).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ScatterFeature {
    /// JSON `description`.
    pub description: Description,
    /// JSON `places_feature`.
    pub places_feature: FeatureId,
    /// JSON `project_input_to_floor`.
    pub project_input_to_floor: Option<bool>,
    /// JSON `iterations` (required).
    #[facet(schema::molang = SCATTER_LEGACY)]
    pub iterations: Molang,
    /// JSON `scatter_chance`.
    #[facet(schema::molang_with = walk_legacy_chance)]
    pub scatter_chance: Option<ScatterChance>,
    /// JSON `coordinate_eval_order`.
    pub coordinate_eval_order: Option<CoordinateEvalOrder>,
    /// JSON `x` (not required by this schema).
    #[facet(schema::molang_with = walk_legacy_coordinate)]
    pub x: Option<CoordinateRange>,
    /// JSON `y`.
    #[facet(schema::molang_with = walk_legacy_coordinate)]
    pub y: Option<CoordinateRange>,
    /// JSON `z`.
    #[facet(schema::molang_with = walk_legacy_coordinate)]
    pub z: Option<CoordinateRange>,
}

strict_struct!(ScatterFeature, VERSION);

/// The Molang of a root `scatter_chance` that is a percentage, at the member: the legacy
/// schema's spec.
fn walk_legacy_chance(chance: &mut Option<ScatterChance>, visitor: &mut MolangVisitor<'_>) {
    if let Some(ScatterChance::Percent(chance)) = chance {
        chance.visit_slot(&SCATTER_LEGACY, visitor);
    }
}

/// The Molang of a root coordinate: the expression at the member, the extent's bounds below it.
fn walk_legacy_coordinate(
    coordinate: &mut Option<CoordinateRange>,
    visitor: &mut MolangVisitor<'_>,
) {
    match coordinate {
        None => {}
        Some(CoordinateRange::Single(value)) => value.visit_slot(&SCATTER_LEGACY, visitor),
        Some(CoordinateRange::Object(range)) => {
            visitor.enter(Step::Key("extent"));
            for (index, bound) in range.extent.iter_mut().enumerate() {
                visitor.field(index, &SCATTER_LEGACY, bound);
            }
            visitor.leave();
        }
    }
}

/// The upgrade step: the six root parameters move into `distribution` unchanged (absent ones stay
/// absent); `places_feature`, `project_input_to_floor` and `description` stay.
impl Upgrade for ScatterFeature {
    type Next = latest::ScatterFeature;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> latest::ScatterFeature {
        latest::ScatterFeature {
            description: self.description,
            places_feature: self.places_feature,
            project_input_to_floor: self.project_input_to_floor,
            distribution: ScatterParams {
                iterations: self.iterations,
                scatter_chance: self.scatter_chance,
                coordinate_eval_order: self.coordinate_eval_order,
                x: self.x,
                y: self.y,
                z: self.z,
            },
        }
    }
}

// ---- single_block_feature ----------------------------------------------------------------------

/// `minecraft:single_block_feature` below format 1.21.40.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct SingleBlockFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block to place, a descriptor (JSON `places_block`).
    #[facet(schema::read_with = not_an_array)]
    pub places_block: Option<BlockDescriptor>,
    /// JSON `enforce_placement_rules`.
    pub enforce_placement_rules: bool,
    /// JSON `enforce_survivability_rules`.
    pub enforce_survivability_rules: bool,
    /// JSON `may_attach_to`.
    pub may_attach_to: Option<AttachConditions>,
    /// JSON `may_replace`.
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

/// `may_attach_to` below format 1.21.40: no `diagonal`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AttachConditions {
    /// JSON `min_sides_must_attach`.
    pub min_sides_must_attach: Option<JsonInt>,
    /// JSON `auto_rotate`.
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
    /// JSON `all`.
    pub all: Option<BlockList>,
    /// JSON `sides`.
    pub sides: Option<BlockList>,
}

strict_struct!(SingleBlockFeature, VERSION);

/// The descriptor member has no array option in this schema: the client says "unknown child
/// schema option type.  Allowed types: %s".
fn not_an_array(node: &Node<'_>, read: &mut Read<'_>) -> Option<BlockDescriptor> {
    if node.value().is_array() {
        read.refuse(node, Code::from(&core_codes::INVALID_TYPE));
        return None;
    }
    BlockDescriptor::from_json(node, read)
}

/// The upgrade step: `places_block` becomes `PlacesBlock::Descriptor` unchanged; `may_attach_to`
/// keeps its members and has no diagonal neighbours (an absent `diagonal`, not an invented
/// `[]`); `may_not_attach_to` and `randomize_rotation` stay absent.
impl Upgrade for SingleBlockFeature {
    type Next = latest::SingleBlockFeature;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> latest::SingleBlockFeature {
        latest::SingleBlockFeature {
            description: self.description,
            places_block: self.places_block.map(PlacesBlock::Descriptor),
            enforce_placement_rules: self.enforce_placement_rules,
            enforce_survivability_rules: self.enforce_survivability_rules,
            randomize_rotation: None,
            may_attach_to: self.may_attach_to.map(|old| latest::AttachConditions {
                min_sides_must_attach: old.min_sides_must_attach,
                auto_rotate: old.auto_rotate,
                top: old.top,
                bottom: old.bottom,
                north: old.north,
                east: old.east,
                south: old.south,
                west: old.west,
                all: old.all,
                sides: old.sides,
                diagonal: None,
            }),
            may_not_attach_to: None,
            may_replace: self.may_replace,
        }
    }
}

// ---- snap_to_surface_feature -------------------------------------------------------------------

/// `minecraft:snap_to_surface_feature` below format 1.26.50:
/// the search range is named `vertical_search_range`; every other member, `wall` and
/// `allow_non_air_placement` included, is the same as in the newest schema.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = DESCRIPTOR_TAGS)]
pub struct SnapToSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// JSON `feature_to_snap`.
    pub feature_to_snap: FeatureId,
    /// How far it searches for the surface (JSON `vertical_search_range`).
    pub vertical_search_range: JsonInt,
    /// JSON `surface`.
    #[facet(schema::invalid = crate::codes::SNAP_SURFACE)]
    pub surface: Option<latest::Surface>,
    /// JSON `allow_air_placement`.
    pub allow_air_placement: Option<bool>,
    /// JSON `allow_non_air_placement`.
    pub allow_non_air_placement: Option<bool>,
    /// JSON `allow_underwater_placement`.
    pub allow_underwater_placement: Option<bool>,
    /// JSON `allowed_surface_blocks`.
    pub allowed_surface_blocks: Option<Vec<BlockDescriptor>>,
    /// JSON `embed_in_surface` (engine 1.26.20).
    pub embed_in_surface: Option<bool>,
}

strict_struct!(SnapToSurfaceFeature, VERSION);

/// The upgrade step: `vertical_search_range` is renamed `search_range`, its value unchanged; every
/// other member keeps its value (an absent `allow_non_air_placement` stays absent).
impl Upgrade for SnapToSurfaceFeature {
    type Next = latest::SnapToSurfaceFeature;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> latest::SnapToSurfaceFeature {
        latest::SnapToSurfaceFeature {
            description: self.description,
            feature_to_snap: self.feature_to_snap,
            search_range: self.vertical_search_range,
            surface: self.surface,
            allow_air_placement: self.allow_air_placement,
            allow_non_air_placement: self.allow_non_air_placement,
            allow_underwater_placement: self.allow_underwater_placement,
            allowed_surface_blocks: self.allowed_surface_blocks,
            embed_in_surface: self.embed_in_surface,
        }
    }
}

// ---- written as read ------------------------------------------------------------------------

/// An older struct writes its document in format 1.13.0.
fn older_json<T: VersionStruct>(value: &T, key: &'static str) -> Result<String, EncodeError> {
    let payload = value.write()?;
    let mut root = facet_value::VObject::new();
    root.insert("format_version", Value::from(VERSION.to_string()));
    root.insert(key, payload);
    bedrock_addon_core::json::to_json(&Value::from(root))
}

impl HistoryJson for ScatterFeature {
    fn history_json(&self, key: &'static str) -> Result<String, EncodeError> {
        older_json(self, key)
    }
}

impl HistoryJson for SingleBlockFeature {
    fn history_json(&self, key: &'static str) -> Result<String, EncodeError> {
        older_json(self, key)
    }
}

impl HistoryJson for SnapToSurfaceFeature {
    fn history_json(&self, key: &'static str) -> Result<String, EncodeError> {
        older_json(self, key)
    }
}

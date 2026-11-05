//! Format 1.13.0 (the initial release) of the three feature types whose schema
//! changed later, and their steps.

use facet::Facet;

use super::Any;
use crate::read::{parse_strict, parse_strict_with};
use crate::scatter::walk_params;
use crate::walk::{SCATTER_LEGACY, Walk, member};
use crate::{self as latest, BlockList, Description, JsonInt, PlacesBlock};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictBool, Value};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::shared::BlockDescriptor;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned, write_payload,
};
use bedrock_addon_support::scatter::{
    CoordinateEvalOrder, CoordinateRange, ScatterChance, ScatterParams,
};

/// The version of the structs of this module.
pub(crate) const VERSION: FormatVersion = FormatVersion::new(1, 13, 0);

// ---- scatter_feature ---------------------------------------------------------------------------

/// `minecraft:scatter_feature` below format 1.21.10: the scatter parameters at the root
/// (the legacy schema).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ScatterFeature {
    /// JSON `description`.
    pub description: Description,
    /// JSON `places_feature`.
    pub places_feature: FeatureId,
    /// JSON `project_input_to_floor`.
    #[facet(default)]
    pub project_input_to_floor: Option<StrictBool>,
    /// JSON `iterations` (required).
    pub iterations: Molang,
    /// JSON `scatter_chance`.
    #[facet(default)]
    pub scatter_chance: Option<ScatterChance>,
    /// JSON `coordinate_eval_order`.
    #[facet(default)]
    pub coordinate_eval_order: Option<CoordinateEvalOrder>,
    /// JSON `x` (not required by this schema).
    #[facet(default)]
    pub x: Option<CoordinateRange>,
    /// JSON `y`.
    #[facet(default)]
    pub y: Option<CoordinateRange>,
    /// JSON `z`.
    #[facet(default)]
    pub z: Option<CoordinateRange>,
}

impl Versioned for ScatterFeature {
    const VERSION: FormatVersion = VERSION;
}

impl VersionStruct for ScatterFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let feature: Self = parse_strict(payload, cx, d)?;
        // The parameters are at the root of this shape.
        crate::scatter::beyond_f32_findings(payload, &[], &cx.at, d);
        Some(feature)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl MolangFields for ScatterFeature {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        walk_params(
            v,
            &SCATTER_LEGACY,
            &mut self.iterations,
            &mut self.scatter_chance,
            [("x", &mut self.x), ("y", &mut self.y), ("z", &mut self.z)],
        );
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
                extra: Extra::default(),
            },
        }
    }
}

// ---- single_block_feature ----------------------------------------------------------------------

/// `minecraft:single_block_feature` below format 1.21.40.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SingleBlockFeature {
    /// JSON `description`.
    pub description: Description,
    /// The block to place, a descriptor (JSON `places_block`).
    #[facet(default)]
    pub places_block: Option<BlockDescriptor>,
    /// JSON `enforce_placement_rules`.
    pub enforce_placement_rules: StrictBool,
    /// JSON `enforce_survivability_rules`.
    pub enforce_survivability_rules: StrictBool,
    /// JSON `may_attach_to`.
    #[facet(default)]
    pub may_attach_to: Option<AttachConditions>,
    /// JSON `may_replace`.
    #[facet(default)]
    pub may_replace: Option<Vec<BlockDescriptor>>,
}

/// `may_attach_to` below format 1.21.40: no `diagonal`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct AttachConditions {
    /// JSON `min_sides_must_attach`.
    #[facet(default)]
    pub min_sides_must_attach: Option<JsonInt>,
    /// JSON `auto_rotate`.
    #[facet(default)]
    pub auto_rotate: Option<StrictBool>,
    /// JSON `top`.
    #[facet(default)]
    pub top: Option<BlockList>,
    /// JSON `bottom`.
    #[facet(default)]
    pub bottom: Option<BlockList>,
    /// JSON `north`.
    #[facet(default)]
    pub north: Option<BlockList>,
    /// JSON `east`.
    #[facet(default)]
    pub east: Option<BlockList>,
    /// JSON `south`.
    #[facet(default)]
    pub south: Option<BlockList>,
    /// JSON `west`.
    #[facet(default)]
    pub west: Option<BlockList>,
    /// JSON `all`.
    #[facet(default)]
    pub all: Option<BlockList>,
    /// JSON `sides`.
    #[facet(default)]
    pub sides: Option<BlockList>,
}

impl Versioned for SingleBlockFeature {
    const VERSION: FormatVersion = VERSION;
}

impl VersionStruct for SingleBlockFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The descriptor member has no array option in this schema: the client says "unknown
        // child schema option type.  Allowed types: %s".
        let array = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text)
            .ok()
            .and_then(|json| json.as_object()?.get("places_block").cloned())
            .is_some_and(|value| value.is_array());
        if array {
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                    .at(cx.at.child("places_block")),
            );
            return None;
        }
        parse_strict(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl Walk for AttachConditions {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        for (key, list) in [
            ("top", &mut self.top),
            ("bottom", &mut self.bottom),
            ("north", &mut self.north),
            ("east", &mut self.east),
            ("south", &mut self.south),
            ("west", &mut self.west),
            ("all", &mut self.all),
            ("sides", &mut self.sides),
        ] {
            member(v, key, list);
        }
    }
}

impl MolangFields for SingleBlockFeature {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        member(v, "places_block", &mut self.places_block);
        member(v, "may_attach_to", &mut self.may_attach_to);
        member(v, "may_replace", &mut self.may_replace);
    }
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SnapToSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// JSON `feature_to_snap`.
    pub feature_to_snap: FeatureId,
    /// How far it searches for the surface (JSON `vertical_search_range`).
    pub vertical_search_range: JsonInt,
    /// JSON `surface`.
    #[facet(default)]
    pub surface: Option<latest::Surface>,
    /// JSON `allow_air_placement`.
    #[facet(default)]
    pub allow_air_placement: Option<StrictBool>,
    /// JSON `allow_non_air_placement`.
    #[facet(default)]
    pub allow_non_air_placement: Option<StrictBool>,
    /// JSON `allow_underwater_placement`.
    #[facet(default)]
    pub allow_underwater_placement: Option<StrictBool>,
    /// JSON `allowed_surface_blocks`.
    #[facet(default)]
    pub allowed_surface_blocks: Option<Vec<BlockDescriptor>>,
    /// JSON `embed_in_surface` (engine 1.26.20).
    #[facet(default)]
    pub embed_in_surface: Option<StrictBool>,
}

impl Versioned for SnapToSurfaceFeature {
    const VERSION: FormatVersion = VERSION;
}

impl VersionStruct for SnapToSurfaceFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_strict_with(payload, cx, d, &[crate::structure::SNAP_SURFACE])
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl MolangFields for SnapToSurfaceFeature {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        member(
            v,
            "allowed_surface_blocks",
            &mut self.allowed_surface_blocks,
        );
    }
}

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

// ---- the public history enum -------------------------------------------------------------------

impl From<ScatterFeature> for Any {
    fn from(value: ScatterFeature) -> Self {
        Any::ScatterV1_13_0(value)
    }
}

impl TryFrom<Any> for ScatterFeature {
    type Error = Any;

    fn try_from(any: Any) -> Result<Self, Any> {
        match any {
            Any::ScatterV1_13_0(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl From<SingleBlockFeature> for Any {
    fn from(value: SingleBlockFeature) -> Self {
        Any::SingleBlockV1_13_0(value)
    }
}

impl TryFrom<Any> for SingleBlockFeature {
    type Error = Any;

    fn try_from(any: Any) -> Result<Self, Any> {
        match any {
            Any::SingleBlockV1_13_0(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl From<SnapToSurfaceFeature> for Any {
    fn from(value: SnapToSurfaceFeature) -> Self {
        Any::SnapToSurfaceV1_13_0(value)
    }
}

impl TryFrom<Any> for SnapToSurfaceFeature {
    type Error = Any;

    fn try_from(any: Any) -> Result<Self, Any> {
        match any {
            Any::SnapToSurfaceV1_13_0(value) => Ok(value),
            other => Err(other),
        }
    }
}

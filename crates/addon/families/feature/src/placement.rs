//! Feature types that place other features: aggregates, sequences, weighted and conditional
//! choices, layouts, searches and filters.

use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{StrictError, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VArray;

use super::Description;
use super::codes;
use super::molang::{WORLD_GEN, refers};
use super::values::{JsonInt, pair, weight_of, weight_value};

/// `minecraft:aggregate_feature`: places every listed feature at the same position, until the
/// early-out rule stops it.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AggregateFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features to place, in order; at least one (JSON `features`).
    #[facet(schema::min_len = 1)]
    pub features: Vec<FeatureId>,
    /// When to stop (JSON `early_out`); absent is `none`.
    pub early_out: Option<EarlyOut>,
}

/// `aggregate_feature.early_out`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum EarlyOut {
    /// `none`: place every feature (the default).
    #[default]
    None,
    /// `first_failure`: stop at the first feature that fails.
    FirstFailure,
    /// `first_success`: stop at the first feature that succeeds.
    FirstSuccess,
}

/// `minecraft:sequence_feature`: places the listed features in order, each at the position the
/// previous one ended at, stopping at the first failure.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SequenceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features to place, in order; at least one (JSON `features`).
    #[facet(schema::min_len = 1)]
    pub features: Vec<FeatureId>,
}

/// `minecraft:weighted_random_feature`: places one of the listed features, picked by weight.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct WeightedRandomFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features and their weights; at least one (JSON `features`).
    #[facet(schema::min_len = 1)]
    pub features: Vec<WeightedFeature>,
}

/// A feature with a weight: `[feature, weight]`.
///
/// JSON: a two-element array, a feature identifier and a number.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct WeightedFeature {
    /// Position 0: the feature.
    pub feature: FeatureId,
    /// Position 1: its weight.
    pub weight: f32,
}

impl TryFrom<Value> for WeightedFeature {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let (first, second) = pair(&value, "a [feature, weight] array")?;
        let feature = first.as_string().ok_or(StrictError::WrongType {
            expected: "a feature identifier",
            found: bedrock_addon_core::json::JsonType::Null,
        })?;
        Ok(WeightedFeature {
            feature: FeatureId::parse(feature.as_str()),
            weight: weight_of(second)?,
        })
    }
}

impl From<&WeightedFeature> for Value {
    fn from(entry: &WeightedFeature) -> Self {
        let mut array = VArray::new();
        array.push(Value::from(entry.feature.as_str()));
        array.push(weight_value(entry.weight));
        Value::from(array)
    }
}

/// `minecraft:conditional_list`: places the features of the list whose condition holds, until
/// its `early_out_scheme` ends the list.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ConditionalListFeature {
    /// JSON `description`.
    pub description: Description,
    /// The candidates, in order (JSON `conditional_features`).
    pub conditional_features: Vec<ConditionalFeature>,
    /// What counts as the end of the list (JSON `early_out_scheme`); absent (or `null`) is
    /// `none`, the constructor's value (`EarlyOutScheme::default()`).
    #[facet(schema::check_with = early_out_scheme)]
    pub early_out_scheme: Option<EarlyOutScheme>,
}

/// One entry of `conditional_list.conditional_features`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ConditionalFeature {
    /// The feature to place (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// A Molang expression over the `world_gen` queries (JSON `condition`); absent always
    /// places (the schema's help text says "If omitted, always places.").
    #[facet(schema::molang = WORLD_GEN)]
    pub condition: Option<Molang>,
}

/// `conditional_list.early_out_scheme`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum EarlyOutScheme {
    /// `condition_success`: stop at the first entry whose condition holds.
    ConditionSuccess,
    /// `placement_success`: stop at the first entry that places.
    PlacementSuccess,
    /// `none`: evaluate every entry (engine 1.26.50); the default, as the schema's help text
    /// says: "Defaults to 'none'.".
    #[default]
    None,
}

/// `minecraft:rect_layout` (base-game packs only): lays features out in rectangles.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct RectLayoutFeature {
    /// JSON `description`.
    pub description: Description,
    /// The share of the area left empty, below 1 (JSON `ratio_of_empty_space`).
    #[facet(
        schema::below = 1.0,
        schema::high_code = Code::from(&codes::EMPTY_SPACE),
        schema::kept
    )]
    pub ratio_of_empty_space: Option<f32>,
    /// The features and their areas (JSON `feature_areas`).
    pub feature_areas: Vec<FeatureArea>,
}

/// One entry of `rect_layout.feature_areas`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct FeatureArea {
    /// The feature (JSON `feature`).
    pub feature: FeatureId,
    /// Its area, `[x, z]` (JSON `area_dimensions`).
    #[facet(schema::min_len = 2, schema::max_len = 2)]
    pub area_dimensions: Vec<JsonInt>,
}

/// `minecraft:scan_surface` (base-game packs only): places a feature on every surface column.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ScanSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `scan_surface_feature`).
    pub scan_surface_feature: FeatureId,
}

/// `minecraft:search_feature`: sweeps a volume for positions where a feature places.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SearchFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// The volume searched, relative to the input position (JSON `search_volume`).
    pub search_volume: SearchVolume,
    /// The direction of the sweep (JSON `search_axis`).
    pub search_axis: SearchAxis,
    /// How many placements must succeed, at least one (JSON `required_successes`); absent
    /// is 1.
    #[facet(schema::min = 1)]
    pub required_successes: Option<JsonInt>,
}

/// `search_feature.search_volume`: two corners, `[x, y, z]` each.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SearchVolume {
    /// JSON `min`.
    #[facet(schema::min_len = 3, schema::max_len = 3)]
    pub min: Vec<JsonInt>,
    /// JSON `max`.
    #[facet(schema::min_len = 3, schema::max_len = 3)]
    pub max: Vec<JsonInt>,
}

/// `search_feature.search_axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum SearchAxis {
    /// `-x`.
    #[facet(rename = "-x")]
    NegativeX,
    /// `+x`.
    #[facet(rename = "+x")]
    PositiveX,
    /// `-y`.
    #[facet(rename = "-y")]
    NegativeY,
    /// `+y`.
    #[facet(rename = "+y")]
    PositiveY,
    /// `-z`.
    #[facet(rename = "-z")]
    NegativeZ,
    /// `+z`.
    #[facet(rename = "+z")]
    PositiveZ,
}

/// `minecraft:surface_relative_threshold_feature`: places a feature only below the surface.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SurfaceRelativeThresholdFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `feature_to_place`).
    pub feature_to_place: FeatureId,
    /// How far below the surface the position must be (JSON `minimum_distance_below_surface`);
    /// absent is 0.
    pub minimum_distance_below_surface: Option<JsonInt>,
}

/// `minecraft:height_difference_filter_feature` (engine 1.26.30): places a feature where the
/// terrain around it rises or falls within bounds.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = no_height_difference)]
pub struct HeightDifferenceFilterFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// JSON `min_required_upward_height_diff` (≥ 0).
    #[facet(schema::min = 0)]
    pub min_required_upward_height_diff: Option<JsonInt>,
    /// JSON `min_required_downward_height_diff` (≥ 0).
    #[facet(schema::min = 0)]
    pub min_required_downward_height_diff: Option<JsonInt>,
    /// JSON `max_allowed_upward_height_diff` (≥ 0).
    #[facet(schema::min = 0)]
    pub max_allowed_upward_height_diff: Option<JsonInt>,
    /// JSON `max_allowed_downward_height_diff` (≥ 0).
    #[facet(schema::min = 0)]
    pub max_allowed_downward_height_diff: Option<JsonInt>,
    /// The radius searched (JSON `search_radius`, ≥ 0).
    #[facet(schema::min = 0)]
    pub search_radius: JsonInt,
}

refers!(AggregateFeature { "features" => features });
refers!(SequenceFeature { "features" => features });
refers!(WeightedRandomFeature { "features" => features });
refers!(ConditionalListFeature { "conditional_features" => conditional_features });
refers!(RectLayoutFeature { "feature_areas" => feature_areas });
refers!(ScanSurfaceFeature { "scan_surface_feature" => scan_surface_feature });
refers!(SearchFeature { "places_feature" => places_feature });
refers!(SurfaceRelativeThresholdFeature { "feature_to_place" => feature_to_place });
// `height_difference_filter_feature.places_feature` is looked up when the feature is placed
// no reference of the link pass.
refers!(HeightDifferenceFilterFeature {});

bedrock_addon_core::from_json_via_value!(WeightedFeature);
bedrock_addon_core::impl_schema_leaf!(WeightedFeature);
bedrock_addon_core::impl_no_molang!(WeightedFeature);

impl super::molang::Refer for WeightedFeature {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        // `[feature, weight]`.
        out.element(0, &self.feature);
    }
}

impl super::molang::Refer for ConditionalFeature {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        out.member("places_feature", &self.places_feature);
    }
}

impl super::molang::Refer for FeatureArea {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        out.member("feature", &self.feature);
    }
}

/// The engine that adds `early_out_scheme: none`.
const EARLY_OUT_NONE_ENGINE: bedrock_addon_core::version::FormatVersion = known::V1_26_50;

/// The enum has two values before engine 1.26.50; the 1.26.50 schema adds `none`.
fn early_out_scheme(scheme: &mut EarlyOutScheme, cx: &mut SchemaCx<'_>) {
    if *scheme == EarlyOutScheme::None
        && cx
            .engine_version()
            .is_some_and(|engine| engine < EARLY_OUT_NONE_ENGINE)
    {
        cx.error("", Code::from(&core_codes::INVALID_VALUE));
    }
}

/// The engine that adds `height_difference_filter_feature`.
pub(crate) const HEIGHT_DIFFERENCE_ENGINE: bedrock_addon_core::version::FormatVersion =
    known::V1_26_30;

/// Logged at level 3 while the feature stays registered: a kept error. The client checks this
/// only when no bound is set, and fails nothing.
fn no_height_difference(filter: &mut HeightDifferenceFilterFeature, cx: &mut SchemaCx<'_>) {
    let none = [
        filter.min_required_upward_height_diff,
        filter.min_required_downward_height_diff,
        filter.max_allowed_upward_height_diff,
        filter.max_allowed_downward_height_diff,
    ]
    .iter()
    .all(Option::is_none);
    if none {
        cx.kept("", Code::from(&codes::NO_HEIGHT_DIFFERENCES));
    }
}

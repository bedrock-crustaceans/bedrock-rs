//! Feature types that place other features: aggregates, sequences, weighted and conditional
//! choices, layouts, searches and filters.

use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VArray;

use super::Description;
use super::Rules;
use super::molang::{WORLD_GEN, Walk, walk};
use super::validate::Check;
use super::values::{JsonInt, pair, weight_of, weight_value};
use bedrock_addon_core::diagnostic::{DiagnosticCode, VanillaText};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{StrictError, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor};
use bedrock_addon_core::versioning::ValidateCx;

/// `minecraft:aggregate_feature`: places every listed feature at the same position, until the
/// early-out rule stops it.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AggregateFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features to place, in order; at least one (JSON `features`).
    pub features: Vec<FeatureId>,
    /// When to stop (JSON `early_out`); absent is `none`.
    pub early_out: Option<EarlyOut>,
}

/// `aggregate_feature.early_out`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SequenceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features to place, in order; at least one (JSON `features`).
    pub features: Vec<FeatureId>,
}

/// `minecraft:weighted_random_feature`: places one of the listed features, picked by weight.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct WeightedRandomFeature {
    /// JSON `description`.
    pub description: Description,
    /// The features and their weights; at least one (JSON `features`).
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ConditionalListFeature {
    /// JSON `description`.
    pub description: Description,
    /// The candidates, in order (JSON `conditional_features`).
    pub conditional_features: Vec<ConditionalFeature>,
    /// What counts as the end of the list (JSON `early_out_scheme`); absent (or `null`) is
    /// `none`, the constructor's value (`EarlyOutScheme::default()`).
    pub early_out_scheme: Option<EarlyOutScheme>,
}

/// One entry of `conditional_list.conditional_features`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ConditionalFeature {
    /// The feature to place (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// A Molang expression over the `world_gen` queries (JSON `condition`); absent always
    /// places (the schema's help text says "If omitted, always places.").
    pub condition: Option<Molang>,
}

/// `conditional_list.early_out_scheme`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RectLayoutFeature {
    /// JSON `description`.
    pub description: Description,
    /// The share of the area left empty, below 1 (JSON `ratio_of_empty_space`).
    pub ratio_of_empty_space: Option<f32>,
    /// The features and their areas (JSON `feature_areas`).
    pub feature_areas: Vec<FeatureArea>,
}

/// One entry of `rect_layout.feature_areas`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FeatureArea {
    /// The feature (JSON `feature`).
    pub feature: FeatureId,
    /// Its area, `[x, z]` (JSON `area_dimensions`).
    pub area_dimensions: Vec<JsonInt>,
}

/// `minecraft:scan_surface` (base-game packs only): places a feature on every surface column.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ScanSurfaceFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `scan_surface_feature`).
    pub scan_surface_feature: FeatureId,
}

/// `minecraft:search_feature`: sweeps a volume for positions where a feature places.
#[derive(Clone, Debug, PartialEq, Facet)]
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
    pub required_successes: Option<JsonInt>,
}

/// `search_feature.search_volume`: two corners, `[x, y, z]` each.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SearchVolume {
    /// JSON `min`.
    pub min: Vec<JsonInt>,
    /// JSON `max`.
    pub max: Vec<JsonInt>,
}

/// `search_feature.search_axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
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
#[derive(Clone, Debug, PartialEq, Facet)]
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct HeightDifferenceFilterFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature to place (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// JSON `min_required_upward_height_diff` (≥ 0).
    pub min_required_upward_height_diff: Option<JsonInt>,
    /// JSON `min_required_downward_height_diff` (≥ 0).
    pub min_required_downward_height_diff: Option<JsonInt>,
    /// JSON `max_allowed_upward_height_diff` (≥ 0).
    pub max_allowed_upward_height_diff: Option<JsonInt>,
    /// JSON `max_allowed_downward_height_diff` (≥ 0).
    pub max_allowed_downward_height_diff: Option<JsonInt>,
    /// The radius searched (JSON `search_radius`, ≥ 0).
    pub search_radius: JsonInt,
}

// The lists after `+`: the members that name a feature.
walk!(fields AggregateFeature {} + { "features" => features });
walk!(fields SequenceFeature {} + { "features" => features });
walk!(fields WeightedRandomFeature {} + { "features" => features });
walk!(fields ConditionalListFeature { "conditional_features" => conditional_features });
walk!(fields RectLayoutFeature {} + { "feature_areas" => feature_areas });
walk!(fields ScanSurfaceFeature {} + { "scan_surface_feature" => scan_surface_feature });
walk!(fields SearchFeature {} + { "places_feature" => places_feature });
walk!(fields SurfaceRelativeThresholdFeature {} + { "feature_to_place" => feature_to_place });
// `height_difference_filter_feature.places_feature` is looked up when the feature is placed
// no reference of the link pass.
walk!(fields HeightDifferenceFilterFeature {});

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

impl Walk for ConditionalFeature {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        v.field("condition", &WORLD_GEN, &mut self.condition);
    }
}

impl Rules for AggregateFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.check_len("features", Some(self.features.len()), 1, None);
    }
}

impl Rules for SequenceFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.check_len("features", Some(self.features.len()), 1, None);
    }
}

impl Rules for WeightedRandomFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.check_len("features", Some(self.features.len()), 1, None);
    }
}

/// The engine that adds `early_out_scheme: none`.
const EARLY_OUT_NONE_ENGINE: bedrock_addon_core::version::FormatVersion = known::V1_26_50;

// Any pack may use the type (it is not internal).
impl Rules for ConditionalListFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        // The enum has two values before engine 1.26.50; the 1.26.50 schema adds `none`.
        if self.early_out_scheme == Some(EarlyOutScheme::None)
            && check.load.engine_version < EARLY_OUT_NONE_ENGINE
        {
            check.error(
                "early_out_scheme",
                DiagnosticCode::InvalidValue { text: None },
            );
        }
    }
}

impl Rules for RectLayoutFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        // The client logs this at level 3 and keeps the value: a kept error.
        if self.ratio_of_empty_space.is_some_and(|r| r >= 1.0) {
            check.kept(
                "ratio_of_empty_space",
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(
                        "100% or above empty space is nonsensical for this feature.",
                    )),
                },
            );
        }
        check.each("feature_areas", &self.feature_areas, |check, area| {
            check.check_len(
                "area_dimensions",
                Some(area.area_dimensions.len()),
                2,
                Some(2),
            );
        });
    }
}

impl Rules for ScanSurfaceFeature {}

impl Rules for SearchFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        check.nested("search_volume", |check| {
            check.check_len("min", Some(self.search_volume.min.len()), 3, Some(3));
            check.check_len("max", Some(self.search_volume.max.len()), 3, Some(3));
        });
        check.int_min("required_successes", self.required_successes, 1);
    }
}

impl Rules for SurfaceRelativeThresholdFeature {}

/// The engine that adds `height_difference_filter_feature`.
pub(crate) const HEIGHT_DIFFERENCE_ENGINE: bedrock_addon_core::version::FormatVersion =
    known::V1_26_30;

impl Rules for HeightDifferenceFilterFeature {
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        let diffs = [
            (
                "min_required_upward_height_diff",
                self.min_required_upward_height_diff,
            ),
            (
                "min_required_downward_height_diff",
                self.min_required_downward_height_diff,
            ),
            (
                "max_allowed_upward_height_diff",
                self.max_allowed_upward_height_diff,
            ),
            (
                "max_allowed_downward_height_diff",
                self.max_allowed_downward_height_diff,
            ),
        ];
        for (key, value) in diffs {
            check.int_min(key, value, 0);
        }
        check.int_min("search_radius", Some(self.search_radius), 0);
        // Logged at level 3 while the feature stays registered: a kept error. The client
        // checks this only when no bound is set, and fails nothing.
        if diffs.iter().all(|(_, value)| value.is_none()) {
            check.kept(
                "",
                DiagnosticCode::MissingRequiredField {
                    text: Some(VanillaText::Fixed(
                        "height_difference_filter_feature: No height differences specified",
                    )),
                },
            );
        }
    }
}

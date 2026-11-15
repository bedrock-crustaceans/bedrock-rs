//! `minecraft:scatter_feature`: a feature placed at positions scattered around the input
//! position, and the scatter parameters it shares with feature rules.

use facet::Facet;

use super::Description;
use super::codes;
use super::read::required_child;
use super::strict_struct;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::kit::{Node, Pass, Read};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::VersionStruct;
use bedrock_addon_shared_scatter::{CoordinateRange, ScatterChance, ScatterParams};

/// The format of the `distribution` layout.
pub(crate) const SCATTER_FORMAT: FormatVersion = FormatVersion::new(1, 21, 10);

/// `minecraft:scatter_feature`: places a feature at positions scattered around the input
/// position. Format 1.21.10 moved the scatter parameters into `distribution`; an older
/// document's root parameters are moved there when it loads.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct ScatterFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed at each position (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// Whether positions are moved down to the floor first (JSON `project_input_to_floor`);
    /// absent is `false`.
    pub project_input_to_floor: Option<bool>,
    /// The scatter parameters (JSON `distribution`).
    #[facet(schema::read_with = distribution, schema::check_with = scatter_rules)]
    pub distribution: ScatterParams,
}

strict_struct!(ScatterFeature, SCATTER_FORMAT);

impl super::molang::Refer for ScatterFeature {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        out.member("places_feature", &self.places_feature);
    }
}

/// `distribution`, each of its three coordinates present: the parameters of a document are
/// complete (a scatter feature of a format below 1.21.10 may leave one out at the root, and the
/// type is as lenient). Each missing one is told: the client names every one it lacks.
fn distribution(node: &Node<'_>, read: &mut Read<'_>) -> Option<ScatterParams> {
    let params = ScatterParams::from_json(node, read)?;
    let mut complete = true;
    for (key, coordinate) in [("x", &params.x), ("y", &params.y), ("z", &params.z)] {
        if coordinate.is_none() {
            read.defer(Pass::Absent, required_child(node.path().child(key)));
            complete = false;
        }
    }
    complete.then_some(params)
}

// ---- runtime checks --------------------------------------------------------------------------

/// A `ScatterParamValue` finding with vanilla's text at `key`: a kept error. The client logs
/// each such line at level 3 and goes on with the replacement value (for the iterations, the
/// percent and fraction chances, a missing coordinate, `step_size` and `grid_offset`), so the
/// feature stays loaded.
fn scatter_warning(cx: &mut SchemaCx<'_>, key: &str, code: Code) {
    cx.kept(key, code);
}

/// The client's checks on a scatter constant, and its replacement values, applied to the
/// parameters at the member `distribution`, or at the payload's root where a file below 1.21.10
/// has them (the upgrade moves them into `distribution`): a rule names the member where the file
/// has it. A schema minimum (`numerator`, `denominator`, `step_size`, `grid_offset`: the shared
/// type's bounds) fails the whole schema.
fn scatter_rules(distribution: &mut ScatterParams, cx: &mut SchemaCx<'_>) {
    let at_root = cx
        .original_version()
        .is_some_and(|version| version < SCATTER_FORMAT);
    if at_root {
        cx.leave();
    }
    distribution.check(cx);
    if matches!(distribution.iterations, Molang::Const(c) if c < 0.0) {
        scatter_warning(cx, "iterations", Code::from(&codes::ITERATIONS_NEGATIVE));
        distribution.iterations = Molang::Const(1.0);
    }
    match &mut distribution.scatter_chance {
        Some(ScatterChance::Percent(Molang::Const(c))) if *c <= 0.0 || *c > 100.0 => {
            scatter_warning(
                cx,
                "scatter_chance",
                Code::from(&codes::SCATTER_CHANCE_PERCENT),
            );
            *c = 100.0;
        }
        Some(ScatterChance::Fraction(ratio)) if ratio.denominator <= ratio.numerator => {
            scatter_warning(
                cx,
                "scatter_chance",
                Code::from(&codes::SCATTER_CHANCE_FRACTION),
            );
            ratio.denominator = 1;
        }
        _ => {}
    }
    for (key, coordinate) in [
        ("x", &distribution.x),
        ("y", &distribution.y),
        ("z", &distribution.z),
    ] {
        coordinate_rules(cx, key, coordinate.as_ref());
    }
    if at_root {
        cx.enter("distribution");
    }
}

/// The checks of one coordinate at `key`.
fn coordinate_rules(cx: &mut SchemaCx<'_>, key: &str, coordinate: Option<&CoordinateRange>) {
    let empty = match coordinate {
        None => true,
        Some(CoordinateRange::Single(Molang::Expr(source))) => source.is_empty(),
        _ => false,
    };
    if empty {
        // The finding is for an absent or empty member.
        scatter_warning(cx, key, codes::MUST_BE_PROVIDED.with([key]));
        return;
    }
    let Some(CoordinateRange::Object(range)) = coordinate else {
        return;
    };
    cx.enter(key);
    if !range.distribution.is_grid() {
        if range.step_size.is_some() {
            scatter_warning(cx, "step_size", Code::from(&codes::STEP_SIZE_NOT_GRID));
        }
        if range.grid_offset.is_some() {
            scatter_warning(cx, "grid_offset", Code::from(&codes::GRID_OFFSET_NOT_GRID));
        }
    }
    if let [Molang::Const(min), Molang::Const(max)] = &range.extent
        && max <= min
    {
        scatter_warning(cx, "extent", Code::from(&codes::EXTENT_ORDER));
    }
    cx.leave();
}

/// A distribution without `x`, `y` or `z` (an upgraded 1.13.0 document) has no 1.21.10 form:
/// written in the 1.13.0 root layout, so it reads back unchanged.
pub(crate) fn older_form(
    feature: &ScatterFeature,
) -> Option<Result<(FormatVersion, Value), EncodeError>> {
    use crate::history::v1_13_0;
    let distribution = &feature.distribution;
    if distribution.x.is_some() && distribution.y.is_some() && distribution.z.is_some() {
        return None;
    }
    let older = v1_13_0::ScatterFeature {
        description: feature.description.clone(),
        places_feature: feature.places_feature.clone(),
        project_input_to_floor: feature.project_input_to_floor,
        iterations: distribution.iterations.clone(),
        scatter_chance: distribution.scatter_chance.clone(),
        coordinate_eval_order: distribution.coordinate_eval_order,
        x: distribution.x.clone(),
        y: distribution.y.clone(),
        z: distribution.z.clone(),
    };
    Some(older.write().map(|payload| (v1_13_0::VERSION, payload)))
}

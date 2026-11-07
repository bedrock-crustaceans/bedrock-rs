//! `minecraft:scatter_feature`: a feature placed at positions scattered around the input
//! position, and the scatter parameters it shares with feature rules.

use facet::Facet;

use super::Description;
use super::Rules;
use super::codes;
use super::molang::{SCATTER, Walk};
use super::read::{parse_strict, required_child};
use super::validate::Check;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{Molang, MolangFieldSpec, MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_shared_scatter::{CoordinateRange, ScatterChance, ScatterParams};

/// The format of the `distribution` layout.
pub(crate) const SCATTER_FORMAT: FormatVersion = FormatVersion::new(1, 21, 10);

/// `minecraft:scatter_feature`: places a feature at positions scattered around the input
/// position. Format 1.21.10 moved the scatter parameters into `distribution`; an older
/// document's root parameters are moved there when it loads.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ScatterFeature {
    /// JSON `description`.
    pub description: Description,
    /// The feature placed at each position (JSON `places_feature`).
    pub places_feature: FeatureId,
    /// Whether positions are moved down to the floor first (JSON `project_input_to_floor`);
    /// absent is `false`.
    pub project_input_to_floor: Option<bool>,
    /// The scatter parameters (JSON `distribution`).
    pub distribution: ScatterParams,
}

// ---- numbers beyond a float -----------------------------------------------------------------

/// The choice members of the scatter parameters (`scatter_chance`, `x`, `y`, `z`, and the
/// two bounds of a coordinate's `extent`) are read through conversions that have no
/// diagnostics at hand: a number there whose nearest `f32` is an infinity is read as the
/// largest finite float of its sign and nothing says so. This says so: core's one warning
/// (`InvalidValue`, parse phase) at each such number of the payload, on its token. `members`
/// are the keys from the payload to the object that holds the four. `iterations` is a typed
/// `Molang` member, which refuses such a number itself.
pub(crate) fn beyond_f32_findings(
    payload: JsonSlice<'_>,
    members: &[&str],
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    let Ok(root) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) else {
        return;
    };
    let mut holder = &root;
    for key in members {
        let Some(next) = holder.as_object().and_then(|object| object.get(key)) else {
            return;
        };
        holder = next;
    }
    let Some(holder) = holder.as_object() else {
        return;
    };
    let beyond = |value: &Value| {
        value
            .as_number()
            .map(facet_value::VNumber::to_f64_lossy)
            .is_some_and(|number| bedrock_addon_core::molang::beyond_f32(number).is_some())
    };
    let mut report = |tail: &[Tail<'_>]| {
        let mut relative = JsonPath::new();
        for key in members {
            relative.push(key);
        }
        for step in tail {
            match step {
                Tail::Key(key) => relative.push(key),
                Tail::Index(index) => relative.push_index(*index),
            }
        }
        let span = bedrock_addon_core::json::span_at(payload, &relative);
        d.push(bedrock_addon_core::molang::beyond_f32_warning(
            at.join(&relative),
            span,
        ));
    };
    for key in ["scatter_chance", "x", "y", "z"] {
        let Some(value) = holder.get(key) else {
            continue;
        };
        if beyond(value) {
            report(&[Tail::Key(key)]);
        }
        let extent = value
            .as_object()
            .and_then(|object| object.get("extent"))
            .and_then(Value::as_array);
        for (index, bound) in extent.into_iter().flatten().enumerate() {
            if beyond(bound) {
                report(&[Tail::Key(key), Tail::Key("extent"), Tail::Index(index)]);
            }
        }
    }
}

/// A step below the scatter parameters.
enum Tail<'a> {
    Key(&'a str),
    Index(usize),
}

// ---- version struct --------------------------------------------------------------------------

impl Versioned for ScatterFeature {
    const VERSION: FormatVersion = SCATTER_FORMAT;
}

impl VersionStruct for ScatterFeature {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let feature: Self = parse_strict(payload, cx, d)?;
        beyond_f32_findings(payload, &["distribution"], &cx.at, d);
        // The three coordinates are required.
        let at = cx.at.child("distribution");
        let mut complete = true;
        for (key, coordinate) in [
            ("x", &feature.distribution.x),
            ("y", &feature.distribution.y),
            ("z", &feature.distribution.z),
        ] {
            if coordinate.is_none() {
                d.push(required_child(at.child(key)));
                complete = false;
            }
        }
        complete.then_some(feature)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// Hands the scatter parameters' Molang fields to `v` with `spec`.
pub(crate) fn walk_params(
    v: &mut MolangVisitor<'_>,
    spec: &MolangFieldSpec,
    iterations: &mut Molang,
    scatter_chance: &mut Option<ScatterChance>,
    coordinates: [(&str, &mut Option<CoordinateRange>); 3],
) {
    v.field("iterations", spec, iterations);
    if let Some(ScatterChance::Percent(value)) = scatter_chance {
        v.field("scatter_chance", spec, value);
    }
    for (key, coordinate) in coordinates {
        match coordinate {
            Some(CoordinateRange::Single(value)) => v.field(key, spec, value),
            Some(CoordinateRange::Object(range)) => {
                v.enter(Step::Key(key));
                v.enter(Step::Key("extent"));
                for (index, bound) in range.extent.iter_mut().enumerate() {
                    v.field(index, spec, bound);
                }
                v.leave();
                v.leave();
            }
            None => {}
        }
    }
}

impl Walk for ScatterParams {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        walk_params(
            v,
            &SCATTER,
            &mut self.iterations,
            &mut self.scatter_chance,
            [("x", &mut self.x), ("y", &mut self.y), ("z", &mut self.z)],
        );
    }
}

impl super::molang::Refer for ScatterFeature {
    fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
        out.member("places_feature", &self.places_feature);
    }
}

impl MolangFields for ScatterFeature {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        super::molang::member(v, "distribution", &mut self.distribution);
    }
}

// ---- runtime checks --------------------------------------------------------------------------

/// A `ScatterParamValue` finding with vanilla's text at `key`: a kept error. The client logs
/// each such line at level 3 and goes on with the replacement value (for the iterations, the
/// percent and fraction chances, a missing coordinate, `step_size` and `grid_offset`), so the
/// feature stays loaded.
fn scatter_warning(check: &mut ValidateCx<'_>, key: &str, code: Code) {
    check.kept(key, code);
}

/// The client's checks on a scatter constant, and its replacement values, applied to the
/// parameters at the current path; a schema minimum (`numerator`, `denominator`,
/// `step_size`, `grid_offset`) fails the whole schema.
pub(crate) fn scatter_rules(check: &mut ValidateCx<'_>, distribution: &mut ScatterParams) {
    if matches!(distribution.iterations, Molang::Const(c) if c < 0.0) {
        scatter_warning(check, "iterations", Code::from(&codes::ITERATIONS_NEGATIVE));
        distribution.iterations = Molang::Const(1.0);
    }
    match &mut distribution.scatter_chance {
        Some(ScatterChance::Percent(Molang::Const(c))) if *c <= 0.0 || *c > 100.0 => {
            scatter_warning(
                check,
                "scatter_chance",
                Code::from(&codes::SCATTER_CHANCE_PERCENT),
            );
            *c = 100.0;
        }
        Some(ScatterChance::Fraction(ratio)) => {
            let (numerator, denominator) = (ratio.numerator, ratio.denominator);
            check.nested("scatter_chance", |check| {
                check.check_int("numerator", Some(numerator.into()), 1, i32::MAX);
                check.check_int("denominator", Some(denominator.into()), 1, i32::MAX);
            });
            if denominator <= numerator {
                scatter_warning(
                    check,
                    "scatter_chance",
                    Code::from(&codes::SCATTER_CHANCE_FRACTION),
                );
                ratio.denominator = 1;
            }
        }
        _ => {}
    }
    for (key, coordinate) in [
        ("x", &distribution.x),
        ("y", &distribution.y),
        ("z", &distribution.z),
    ] {
        coordinate_rules(check, key, coordinate.as_ref());
    }
}

/// The checks of one coordinate at `key`.
fn coordinate_rules(check: &mut ValidateCx<'_>, key: &str, coordinate: Option<&CoordinateRange>) {
    let empty = match coordinate {
        None => true,
        Some(CoordinateRange::Single(Molang::Expr(source))) => source.is_empty(),
        _ => false,
    };
    if empty {
        // The finding is for an absent or empty member.
        scatter_warning(check, key, codes::MUST_BE_PROVIDED.with([key]));
        return;
    }
    let Some(CoordinateRange::Object(range)) = coordinate else {
        return;
    };
    check.nested(key, |check| {
        check.check_int(
            "step_size",
            range.step_size.map(crate::JsonInt::from),
            1,
            i32::MAX,
        );
        check.check_int(
            "grid_offset",
            range.grid_offset.map(crate::JsonInt::from),
            0,
            i32::MAX,
        );
        if !range.distribution.is_grid() {
            if range.step_size.is_some() {
                scatter_warning(check, "step_size", Code::from(&codes::STEP_SIZE_NOT_GRID));
            }
            if range.grid_offset.is_some() {
                scatter_warning(
                    check,
                    "grid_offset",
                    Code::from(&codes::GRID_OFFSET_NOT_GRID),
                );
            }
        }
        if let [Molang::Const(min), Molang::Const(max)] = &range.extent
            && max <= min
        {
            scatter_warning(check, "extent", Code::from(&codes::EXTENT_ORDER));
        }
    });
}

impl Rules for ScatterFeature {
    /// A distribution without `x`, `y` or `z` (an upgraded 1.13.0 document) has no
    /// 1.21.10 form: written in the 1.13.0 root layout, so it reads back unchanged.
    fn older_form(&self) -> Option<Result<(FormatVersion, Value), EncodeError>> {
        use crate::history::v1_13_0;
        let distribution = &self.distribution;
        if distribution.x.is_some() && distribution.y.is_some() && distribution.z.is_some() {
            return None;
        }
        let older = v1_13_0::ScatterFeature {
            description: self.description.clone(),
            places_feature: self.places_feature.clone(),
            project_input_to_floor: self.project_input_to_floor,
            iterations: distribution.iterations.clone(),
            scatter_chance: distribution.scatter_chance.clone(),
            coordinate_eval_order: distribution.coordinate_eval_order,
            x: distribution.x.clone(),
            y: distribution.y.clone(),
            z: distribution.z.clone(),
        };
        Some(older.write().map(|payload| (v1_13_0::VERSION, payload)))
    }

    /// A file below 1.21.10 has the parameters at the root of the payload (the upgrade moves
    /// them into `distribution`): a rule names the member where the file has it, as the
    /// finding for a number beyond a float does.
    fn rules(&mut self, check: &mut ValidateCx<'_>) {
        if check.original_version < SCATTER_FORMAT {
            scatter_rules(check, &mut self.distribution);
        } else {
            check.nested("distribution", |check| {
                scatter_rules(check, &mut self.distribution);
            });
        }
    }
}

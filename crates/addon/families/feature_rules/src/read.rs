//! Reading [`FeatureRule`] from the text of a payload as vanilla's cereal schema does: an
//! unknown member warns, every other failure skips the file. Also the Molang walk of the rule.

use facet::Facet;
use facet_json::RawJson;

use super::{FeatureRule, FeatureRuleConditions, FeatureRuleDescription};
use bedrock_addon_core::diagnostic::{Class, Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::{
    JsonSlice, NullRule, ScalarFailure, Value, keep_unknown_members_with, parse_with_rules,
    reject_nulls,
};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::versioning::{ParseCx, parse_payload};
use bedrock_addon_shared_scatter::{
    CoordinateDistribution, CoordinateEvalOrder, CoordinateRange, ScatterChance, ScatterFraction,
    ScatterParams,
};

/// What a payload is read into: the members whose JSON form is a choice of shapes are held as
/// raw slices and read one at a time.
#[derive(Facet)]
struct Shell<'a> {
    description: FeatureRuleDescription,
    conditions: FeatureRuleConditions,
    distribution: Option<DistributionShell<'a>>,
}

/// What `distribution` is read into.
#[derive(Facet)]
struct DistributionShell<'a> {
    iterations: Molang,
    scatter_chance: Option<RawJson<'a>>,
    coordinate_eval_order: Option<CoordinateEvalOrder>,
    x: RawJson<'a>,
    y: RawJson<'a>,
    z: RawJson<'a>,
}

/// Reads one `minecraft:feature_rules` payload; `None` (with an error in `d`) when vanilla's
/// cereal pass would fail the file.
pub(crate) fn parse_rule(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<FeatureRule> {
    let shell: Shell<'_> = parse_payload(payload, cx, d)?;
    if let Some(filter) = &shell.conditions.biome_filter {
        let at = cx.at.with(&["conditions", "minecraft:biome_filter"]);
        d.extend(filter.check(&at, cx.load.grade(Class::Unknown)));
    }
    let distribution = match shell.distribution {
        None => None,
        Some(distribution) => {
            let at = cx.at.child("distribution");
            Some(scatter_params(
                payload,
                distribution,
                &at,
                cx.load.null_rule,
                d,
            )?)
        }
    };
    Some(FeatureRule {
        description: shell.description,
        conditions: shell.conditions,
        distribution,
        source_format_version: cx.original_version,
    })
}

/// `distribution`, its choice members read from their slices.
fn scatter_params(
    payload: JsonSlice<'_>,
    shell: DistributionShell<'_>,
    at: &JsonPath,
    rule: Option<NullRule>,
    d: &mut Diagnostics,
) -> Option<ScatterParams> {
    let scatter_chance = match &shell.scatter_chance {
        None => None,
        Some(raw) => Some(chance(payload, raw, &at.child("scatter_chance"), rule, d)?),
    };
    let x = coordinate(payload, &shell.x, &at.child("x"), rule, d);
    let y = coordinate(payload, &shell.y, &at.child("y"), rule, d);
    let z = coordinate(payload, &shell.z, &at.child("z"), rule, d);
    let mut within = true;
    if let Some(ScatterChance::Fraction(fraction)) = &scatter_chance {
        let at = at.child("scatter_chance");
        within &= minimum(fraction.numerator, 1, &at.child("numerator"), d);
        within &= minimum(fraction.denominator, 1, &at.child("denominator"), d);
    }
    for (key, coordinate) in [("x", &x), ("y", &y), ("z", &z)] {
        if let Some(CoordinateRange::Object(range)) = coordinate {
            let at = at.child(key);
            if let Some(step) = range.step_size {
                within &= minimum(step, 1, &at.child("step_size"), d);
            }
            if let Some(offset) = range.grid_offset {
                within &= minimum(offset, 0, &at.child("grid_offset"), d);
            }
        }
    }
    if !within {
        return None;
    }
    Some(ScatterParams {
        iterations: shell.iterations,
        scatter_chance,
        coordinate_eval_order: shell.coordinate_eval_order,
        x: Some(x?),
        y: Some(y?),
        z: Some(z?),
    })
}

/// A bind's minimum: below it vanilla's constraint violation is fatal under the slice's error
/// mask and fails the file; the crate reports it as `InvalidValue`, without vanilla's generic
/// cereal text. The game's minimums are 1 for a fraction's parts and a coordinate's
/// `step_size`, and 0 for `grid_offset`.
fn minimum(value: i32, min: i32, at: &JsonPath, d: &mut Diagnostics) -> bool {
    if value >= min {
        return true;
    }
    d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(at.clone()));
    false
}

/// Whether `value` is the object form of a Molang value (`{"expression", "version"}`); any other
/// object is the struct form (vanilla tries the struct first, as the order of the cereal
/// options shows).
fn is_expression_object(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.contains_key("expression") && o.contains_key("version"))
}

/// Whether the raw value is the struct form of a choice member.
fn is_struct_form(slice: JsonSlice<'_>) -> bool {
    bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text)
        .is_ok_and(|value| value.is_object() && !is_expression_object(&value))
}

/// `scatter_chance`: `{numerator, denominator}` or a percentage.
fn chance(
    payload: JsonSlice<'_>,
    raw: &RawJson<'_>,
    at: &JsonPath,
    rule: Option<NullRule>,
    d: &mut Diagnostics,
) -> Option<ScatterChance> {
    let slice = payload.nested(raw)?;
    if is_struct_form(slice) {
        return typed::<ScatterFraction>(slice, at, rule, d).map(ScatterChance::Fraction);
    }
    typed::<Molang>(slice, at, rule, d).map(ScatterChance::Percent)
}

/// `x`, `y` or `z`: a distribution object or an expression.
fn coordinate(
    payload: JsonSlice<'_>,
    raw: &RawJson<'_>,
    at: &JsonPath,
    rule: Option<NullRule>,
    d: &mut Diagnostics,
) -> Option<CoordinateRange> {
    let slice = payload.nested(raw)?;
    if is_struct_form(slice) {
        return typed::<CoordinateDistribution>(slice, at, rule, d).map(CoordinateRange::Object);
    }
    typed::<Molang>(slice, at, rule, d).map(CoordinateRange::Single)
}

/// One member read from its own slice, by the slice's scalar rules ([`crate::READING`]), its
/// unknown members kept and warned; a failure is an error at the failing member (a slice that is itself `null` fails the typed read, `Molang`
/// refuses it: `InvalidType`). When the slice rejects nulls (`rule`) a `null` where the member
/// reads a value inside it is an `InvalidType` error at it, and the result is `None`.
fn typed<T: for<'f> Facet<'f>>(
    slice: JsonSlice<'_>,
    at: &JsonPath,
    rule: Option<NullRule>,
    d: &mut Diagnostics,
) -> Option<T> {
    let read = parse_with_rules::<T>(slice, None, crate::READING.scalar_rules());
    match read.map(|read| read.value) {
        Ok(value) if rule == Some(NullRule::Rejected) && reject_nulls(&value, slice, at, d) => None,
        Ok(value) => {
            if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text) {
                let mut kept = bedrock_addon_core::document::Unknown::new();
                let keeps = crate::READING.keeps_unknown;
                let found = keep_unknown_members_with(&value, &json, keeps, &mut kept);
                d.unknown_mut().absorb(at, kept);
                for (path, _) in found {
                    let mut full = at.clone();
                    full.append(&path);
                    // Under the slice's error mask an unknown member is a level-2 content-log
                    // line, a warning that does not stop the load: the reading's grade.
                    d.push(
                        Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                            .severity(crate::READING.grade(Class::Unknown))
                            .at(full),
                    );
                }
            }
            Some(value)
        }
        Err(ScalarFailure::Read(error)) => {
            d.push(error.into_diagnostic(at, |code, _| code));
            None
        }
        // A scalar no arm of an untagged enum takes: a type error at the enum.
        Err(ScalarFailure::Refused(misread)) => {
            d.push(misread.spanned_finding(at));
            None
        }
    }
}

// ---- Molang ------------------------------------------------

impl MolangFields for FeatureRule {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(params) = &mut self.distribution {
            visitor.nested("distribution", params);
        }
    }
}

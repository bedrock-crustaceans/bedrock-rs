//! The family rules of feature rules that need the loaded rule only; the identifier and file-name checks are the loader's (they need the
//! path).

use super::codes;
use super::{CoordinateDistribution, CoordinateRange, FeatureRule, ScatterChance, ScatterParams};
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, Phase};
use bedrock_addon_core::molang::{Molang, MolangSource};
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_shared_scatter::percent_in_range;

/// The rules on a loaded rule.
pub(crate) fn validate(rule: &mut FeatureRule, cx: &mut ValidateCx<'_>) {
    // Vanilla checks the pass, then fills the filter and the scatter parameters.
    unknown_pass(rule, cx);
    if let Some(params) = &rule.distribution {
        let at = cx.at.child("distribution");
        scatter_checks(params, &at, cx);
    }
}

/// The checks the game makes on the scatter parameters' constants, each logged with vanilla's
/// text; the value is replaced or ignored where vanilla converts it (the `effective_*`
/// helpers), the document keeps what was written.
fn scatter_checks(params: &ScatterParams, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    if matches!(params.iterations, Molang::Const(c) if c < 0.0) {
        warn(cx, at.child("iterations"), &codes::ITERATIONS_NEGATIVE);
    }
    match &params.scatter_chance {
        Some(ScatterChance::Percent(Molang::Const(c))) if !percent_in_range(*c) => {
            warn(cx, at.child("scatter_chance"), &codes::SCATTER_CHANCE_RANGE);
        }
        Some(ScatterChance::Fraction(fraction)) if fraction.denominator <= fraction.numerator => {
            warn(
                cx,
                at.child("scatter_chance"),
                &codes::SCATTER_CHANCE_FRACTION,
            );
        }
        _ => {}
    }
    for (key, coordinate) in [("x", &params.x), ("y", &params.y), ("z", &params.z)] {
        let at = at.child(key);
        match coordinate {
            Some(CoordinateRange::Object(range)) => coordinate_checks(range, &at, cx),
            Some(CoordinateRange::Single(value)) if usable(value, &at, cx) => {}
            // An absent coordinate (never from a rule file, where the three are required) is
            // as unusable as an empty expression.
            Some(CoordinateRange::Single(_)) | None => {
                {
                    // The game logs this for the expression form only.
                    warn(cx, at, codes::COORDINATE_REQUIRED.with([key]));
                }
            }
        }
    }
}

/// Whether a coordinate's expression compiles to something: an empty text is no expression
/// (the game treats an empty string as a failed node), and with the compile check a rejected
/// one is none either.
fn usable(value: &Molang, at: &JsonPath, cx: &ValidateCx<'_>) -> bool {
    if value.source().is_some_and(MolangSource::is_empty) {
        return false;
    }
    #[cfg(feature = "molang")]
    {
        use bedrock_addon_core::molang::MolangOutcome;
        if cx.molang_failures.iter().any(|failure| {
            failure.path == *at && matches!(failure.outcome, MolangOutcome::Failed(_))
        }) {
            return false;
        }
    }
    #[cfg(not(feature = "molang"))]
    let _ = (at, cx);
    true
}

/// The checks on one coordinate object: grid members outside a grid distribution, an empty extent.
fn coordinate_checks(range: &CoordinateDistribution, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    if !range.distribution.is_grid() {
        // The grid members mean something only to the two grid distributions.
        if range.step_size.is_some() {
            warn(cx, at.child("step_size"), &codes::STEP_SIZE_NOT_GRID);
        }
        if range.grid_offset.is_some() {
            warn(cx, at.child("grid_offset"), &codes::GRID_OFFSET_NOT_GRID);
        }
    }
    if range.extent_is_empty() {
        warn(cx, at.child("extent"), &codes::EXTENT_EMPTY);
    }
}

/// A pass vanilla does not define is logged and the rule is kept, in the bucket of that pass,
/// which nothing places.
fn unknown_pass(rule: &FeatureRule, cx: &mut ValidateCx<'_>) {
    if rule.conditions.pass().is_some() {
        return;
    }
    let at = cx.at.with(&["conditions", "placement_pass"]);
    warn(
        cx,
        at,
        codes::UNKNOWN_PASS.with([
            rule.description.identifier.as_str(),
            rule.conditions.placement_pass.as_str(),
        ]),
    );
}

/// An `InvalidValue` with vanilla's `text` at `at`: vanilla logs it at content-log error level
/// and keeps the rule: a kept error, an error-level finding after which the game keeps the
/// document. The game logs the unknown pass and goes on; it logs each scatter check the same
/// way and goes on with the replaced value, and the rule is inserted.
fn warn(cx: &mut ValidateCx<'_>, at: JsonPath, code: impl Into<Code>) {
    cx.diagnostics
        .push_kept_error(Diagnostic::of(code).at(at).phase(Phase::Validate));
}

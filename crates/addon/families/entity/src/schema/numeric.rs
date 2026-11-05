//! The numeric checks of the legacy goal readers: the bounds the game applies to the numeric
//! members of a goal (an int at least a bound, a float above zero, a float from 0 up to a bound)
//! and the post-parse check of a pair of members where the lower one must stay below the upper
//! one. The rows are [`numeric_table`](super::numeric_table), generated from the game's schema
//! description.
//!
//! Each setter stores the value read, then, when the value is out of its bound, stores the default
//! and logs the row's text at the content log's error level, printf-style with `%d` or `%.4f`.
//! No setter marks the parse as failed: the goal is kept, so the crate reports each line as a kept
//! error. A value of the node's type only reaches a setter: the validation before it took out or
//! refused every other (`validate`).

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::Value;

use super::Problem;
use super::numeric_table::CHECKS;

/// What a row checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// An int that must be at least the bound (`bound <= value`).
    IntAtLeast,
    /// An int that must be above the bound (`bound < value`).
    IntAbove,
    /// A float that must be at least the bound.
    FloatAtLeast,
    /// A float that must be above the bound.
    FloatAbove,
    /// A float from 0 up to the bound.
    FloatUpTo,
    /// The post-parse check of a pair: the row's `dependency` (the lower member) must be below
    /// its `member`; else the member takes the lower one's value.
    Dependency,
}

/// One row: a goal's member, its check and the text logged.
#[derive(Debug)]
pub(crate) struct Check {
    /// The goal's key.
    pub(crate) goal: &'static str,
    /// The member (of a dependency, the upper one).
    pub(crate) member: &'static str,
    /// What is checked.
    pub(crate) kind: Kind,
    /// The bound (of [`Kind::FloatUpTo`], the upper end; unused by [`Kind::Dependency`]).
    pub(crate) bound: f64,
    /// The default the member takes when its value fails.
    pub(crate) default: f64,
    /// The lower member of a [`Kind::Dependency`], else empty.
    pub(crate) dependency: &'static str,
    /// The text, the value's place as `{}`.
    pub(crate) template: &'static str,
}

/// jsoncpp's `asInt` of a number the node took (`Json::Value::asInt`): a bool 0 or 1,
/// an integer its low 32 bits, a real truncated toward zero, saturating.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the low 32 bits and a saturating truncation, as the game converts"
)]
fn as_int(value: &Value) -> Option<i32> {
    if let Some(flag) = value.as_bool() {
        return Some(i32::from(flag));
    }
    let number = value.as_number()?;
    Some(if number.is_float() {
        number.to_f64_lossy() as i32
    } else {
        number
            .to_i64()
            .or_else(|| number.to_u64().map(u64::cast_signed))
            .map_or(0, |n| n as i32)
    })
}

/// jsoncpp's `asFloat` of a number the node took (`Json::Value::asFloat`): a bool 0 or 1, a number as `f32`.
#[allow(clippy::cast_possible_truncation, reason = "the game holds an f32")]
fn as_float(value: &Value) -> Option<f32> {
    if let Some(flag) = value.as_bool() {
        return Some(f32::from(u8::from(flag)));
    }
    Some(value.as_number()?.to_f64_lossy() as f32)
}

/// A row's default as the JSON number written back.
#[allow(
    clippy::cast_possible_truncation,
    reason = "an int row's default is an integer"
)]
fn default_value(check: &Check) -> Value {
    match check.kind {
        Kind::IntAtLeast | Kind::IntAbove => Value::from(check.default as i64),
        _ => Value::from(check.default),
    }
}

/// `%.4f` of an `f32` widened to `double`, as the game's `%.4f` prints it.
fn four_places(value: f32) -> String {
    format!("{:.4}", f64::from(value))
}

/// Runs the numeric checks of `goal` over `json` (the goal's value as its reader parses it, after
/// a validation that did not fail): the lines the game logs, in the table's order, and the JSON
/// with each failed member at its default (`None` when no member failed).
pub(crate) fn check(goal: &str, json: &Value) -> (Vec<Problem>, Option<Value>) {
    let mut problems = Vec::new();
    let Some(object) = json.as_object() else {
        return (problems, None);
    };
    let mut object = object.clone();
    let mut changed = false;
    let rows = CHECKS.iter().filter(|check| check.goal == goal);
    // In the order the file writes the members: the setters run as the parse meets them, so the
    // lines come out in the file's member order, not the table's.
    let names: Vec<String> = object.keys().map(|key| key.as_str().to_owned()).collect();
    for name in &names {
        let Some(row) = rows
            .clone()
            .find(|row| row.kind != Kind::Dependency && row.member == name.as_str())
        else {
            continue;
        };
        let Some(value) = object.get(row.member) else {
            continue;
        };
        let (failed, shown) = match row.kind {
            Kind::IntAtLeast | Kind::IntAbove => {
                let Some(v) = as_int(value) else { continue };
                let v64 = f64::from(v);
                let failed = if row.kind == Kind::IntAtLeast {
                    v64 < row.bound
                } else {
                    v64 <= row.bound
                };
                (failed, v.to_string())
            }
            Kind::FloatAtLeast | Kind::FloatAbove | Kind::FloatUpTo => {
                let Some(v) = as_float(value) else { continue };
                let v64 = f64::from(v);
                let failed = match row.kind {
                    // A NaN fails every comparison, as the game's comparisons do.
                    Kind::FloatAtLeast => v64.is_nan() || v64 < row.bound,
                    Kind::FloatAbove => v64.is_nan() || v64 <= row.bound,
                    _ => v64.is_nan() || v64 < 0.0 || v64 > row.bound,
                };
                (failed, four_places(v))
            }
            Kind::Dependency => continue,
        };
        if failed {
            object.insert(row.member, default_value(row));
            changed = true;
            problems.push(Problem::Numeric {
                at: JsonPath::new().child(row.member),
                template: row.template,
                value: shown,
            });
        }
    }
    for row in rows.filter(|row| row.kind == Kind::Dependency) {
        // The two values the goal holds after the parse: as read (and checked), or the
        // defaults of their rows.
        let held = |name: &str| -> Option<f32> {
            match object.get(name) {
                Some(value) => as_float(value),
                #[allow(clippy::cast_possible_truncation, reason = "the game holds an f32")]
                None => CHECKS
                    .iter()
                    .find(|other| other.goal == row.goal && other.member == name)
                    .map(|other| other.default as f32),
            }
        };
        let (Some(lower), Some(upper)) = (held(row.dependency), held(row.member)) else {
            continue;
        };
        if lower.is_nan() || upper.is_nan() || lower >= upper {
            // The `f32` as its shortest decimal (`0.55`, not `0.550000011920929`).
            let lower_value = lower
                .to_string()
                .parse::<f64>()
                .map_or(Value::NULL, Value::from);
            object.insert(row.member, lower_value);
            changed = true;
            problems.push(Problem::Numeric {
                at: JsonPath::new().child(row.member),
                template: row.template,
                value: four_places(lower),
            });
        }
    }
    (problems, changed.then(|| Value::from(object)))
}

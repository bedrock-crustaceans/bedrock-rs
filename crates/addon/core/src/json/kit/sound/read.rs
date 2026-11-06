//! The scalar members of a sound document as the game's reader reads them: each with its
//! coercion and the warning when the member has another JSON type.

use super::report::Reporter;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{DiagnosticCode, Severity};
use crate::json::Value;
use crate::json::jsoncpp::fork::{self, Stored};

/// A float member read with `asFloat(default)`; a string, array or object reads as 0.0 with a
/// warning.
pub fn float(value: &Value, key: &str, default: f32, report: &mut Reporter<'_>) -> f32 {
    let member = fork::member(value, key);
    if matches!(
        fork::stored(member),
        Stored::String | Stored::Array | Stored::Object
    ) {
        report.at(
            key,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
    }
    fork::as_float(member, default)
}

/// A float member that counts only when it is written with a fraction (`isDouble()`).
pub fn real(value: &Value, key: &str) -> f32 {
    let member = fork::member(value, key);
    if fork::stored(member) == Stored::Real {
        fork::as_float(member, 0.0)
    } else {
        0.0
    }
}

/// A bool member read with `asBool(default)`; any other type but `null` reads by the reader's
/// rule with a warning.
pub fn boolean(value: &Value, key: &str, default: bool, report: &mut Reporter<'_>) -> bool {
    let member = fork::member(value, key);
    if !matches!(fork::stored(member), Stored::Null | Stored::Bool) {
        report.at(
            key,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
    }
    fork::as_bool(member, default)
}

/// An integer member read with `asInt(default)`; any other type but `null` reads by the
/// reader's rule with a warning.
pub fn integer(value: &Value, key: &str, default: i32, report: &mut Reporter<'_>) -> i32 {
    let member = fork::member(value, key);
    if !matches!(fork::stored(member), Stored::Null | Stored::Integer) {
        report.at(
            key,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
    }
    fork::as_int(member, default)
}

/// A string member read with `asString()`; a number, bool, array or object reads by the
/// reader's rule with a warning.
pub fn string(value: &Value, key: &str, report: &mut Reporter<'_>) -> String {
    let member = fork::member(value, key);
    if !matches!(fork::stored(member), Stored::Null | Stored::String) {
        report.at(
            key,
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
        );
    }
    fork::to_text(member)
}

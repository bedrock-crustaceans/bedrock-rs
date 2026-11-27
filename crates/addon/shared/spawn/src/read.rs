//! The findings the spawn shell tells itself, before the payload is read: a parse finding at a
//! path. The members are read by the derived reader under the game's jsoncpp accessors
//! (`Scalars::Accessors`): a default applies only to a missing or `null` member, every other JSON
//! type is coerced, never rejected, and the crate tells the coercion (vanilla is silent).

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};

/// A finding of the spawn shell at `at` (a parse finding without a span).
pub fn finding(code: Code, severity: Severity, at: &JsonPath, d: &mut Diagnostics) {
    d.push(Diagnostic::error(code).severity(severity).at(at.clone()));
}

/// A finding that is built already (it carries a detail) at `at`, told like [`finding`].
pub fn finding_with(found: Diagnostic, at: &JsonPath, d: &mut Diagnostics) {
    d.push(found.at(at.clone()));
}

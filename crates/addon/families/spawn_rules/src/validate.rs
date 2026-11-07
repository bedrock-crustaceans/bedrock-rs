//! Family rules on the loaded documents: the spawn pool of standard spawn rules, the constraints
//! and placement names of simplified spawn rules, and the Molang outcome of the `tags` members.

use super::codes;
use super::{SimplifiedSpawnRules, StandardSpawnRules};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_shared_spawn::legacy::RULES_KEY;
use bedrock_addon_shared_spawn::read::finding;
use bedrock_addon_shared_spawn::validate::{molang, path};

/// Standard spawn rules: a missing or empty `population_control` is a `MissingRequiredField`
/// information (vanilla: silent), an unknown one an `InvalidEnumValue` information with the
/// text vanilla writes to its developer log; either way the document loads and is not
/// registered.
pub(crate) fn standard(rules: &mut StandardSpawnRules, cx: &mut ValidateCx<'_>) {
    let at = path(&cx.at, &[RULES_KEY, "description", "population_control"]);
    // Both are informations: an absent or empty name is silent, and "No population control
    // specified for '%s'" goes only to the developer log.
    match rules.description.population_control.as_deref() {
        None | Some("") => finding(
            DiagnosticCode::from(&core_codes::MISSING_REQUIRED_FIELD),
            Severity::Info,
            &at,
            cx.diagnostics,
        ),
        Some(name) if super::PopulationControl::from_name(name).is_none() => finding(
            DiagnosticCode::from(
                codes::NO_POPULATION_CONTROL.with([rules.description.identifier.as_str()]),
            ),
            Severity::Info,
            &at,
            cx.diagnostics,
        ),
        Some(_) => {}
    }
    molang(rules, cx);
}

/// Simplified spawn rules: the cereal constraints and the placement validator.
pub(crate) fn simplified(rules: &SimplifiedSpawnRules, cx: &mut ValidateCx<'_>) {
    super::simplified::validate(rules, &cx.at, cx.diagnostics);
}

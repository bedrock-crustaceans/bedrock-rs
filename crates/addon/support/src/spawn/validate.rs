//! The helpers of the family rules on the loaded documents: a path below another and the Molang
//! outcome of the `tags` members.

use bedrock_addon_core::diagnostic::JsonPath;
#[cfg(feature = "molang")]
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::versioning::ValidateCx;

/// `at` followed by `keys`.
pub fn path(at: &JsonPath, keys: &[&str]) -> JsonPath {
    let mut path = at.clone();
    for key in keys {
        path.push(key);
    }
    path
}

/// A rejected `tags` expression is told even when the compiler's own messages are infos (an
/// empty text below Molang 4, which the crate reports itself): the descriptor is kept, its
/// expression the rejected constant 0.
#[cfg(feature = "molang")]
pub fn molang<T: bedrock_addon_core::molang::MolangFields>(
    document: &mut T,
    cx: &mut ValidateCx<'_>,
) {
    use bedrock_addon_core::diagnostic::{Diagnostic, Location, Phase};
    use bedrock_addon_core::molang::{MolangFieldMut, MolangOutcome, visit_fields};
    let failed: Vec<JsonPath> = cx
        .molang_failures
        .iter()
        .filter(|f| matches!(f.outcome, MolangOutcome::Failed(_)))
        .map(|f| f.path.clone())
        .collect();
    for at in failed {
        let reported = cx.diagnostics.iter().any(|d| {
            matches!(d.code, DiagnosticCode::InvalidMolang { .. })
                && d.severity >= Severity::Warning
                && d.location.json_path == at
        });
        if reported {
            continue;
        }
        let mut expression = None;
        visit_fields(document, &JsonPath::new(), |field| {
            if *field.path == at
                && let MolangFieldMut::Source(source) = field.value
            {
                expression = Some(source.as_str().to_owned());
            }
        });
        // The crate's own information: below Molang 4 an empty text has no parser line ("No
        // tokens found in expression" is logged from version 4 only).
        cx.diagnostics.push(Diagnostic::new(
            DiagnosticCode::InvalidMolang {
                expression: expression.unwrap_or_default(),
            },
            Severity::Info,
            Phase::Validate,
            Location::new(None, None, at, None),
        ));
    }
}

/// Without the `molang` feature nothing compiles the Molang of a document: no findings.
#[cfg(not(feature = "molang"))]
pub fn molang<T: bedrock_addon_core::molang::MolangFields>(_: &mut T, _: &mut ValidateCx<'_>) {}

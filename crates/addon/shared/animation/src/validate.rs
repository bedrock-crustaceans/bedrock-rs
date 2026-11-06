//! The findings both file kinds share: event strings the game ignores, and events in a pack
//! without `min_engine_version`.

use super::types::EventString;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::load::LoadContext;

/// Push a finding of the validate phase at `at`.
pub fn push(diagnostics: &mut Diagnostics, code: DiagnosticCode, severity: Severity, at: JsonPath) {
    diagnostics.push(
        Diagnostic::error(code)
            .severity(severity)
            .at(at)
            .phase(Phase::Validate),
    );
}

/// `UnrecognizedEventString` (this crate's own) for each `@…` event that is not `@s <name>`, and
/// whether any event string was seen.
pub fn events<'a>(
    list: impl IntoIterator<Item = (JsonPath, &'a EventString)>,
    diagnostics: &mut Diagnostics,
) -> bool {
    let mut any = false;
    for (at, event) in list {
        any = true;
        if let EventString::Ignored(text) = event
            && text.starts_with('@')
        {
            push(
                diagnostics,
                DiagnosticCode::UnrecognizedEventString,
                Severity::Info,
                at,
            );
        }
    }
    any
}

/// `MissingMinEngineVersion` when a pack without `min_engine_version` has event strings: their
/// command version is unset. The game only asserts here, so the crate reports it as an
/// information.
pub fn command_version_unset(cx: &LoadContext, any_event: bool, diagnostics: &mut Diagnostics) {
    if any_event && cx.min_engine_version.is_none() {
        push(
            diagnostics,
            DiagnosticCode::MissingMinEngineVersion,
            Severity::Info,
            JsonPath::new(),
        );
    }
}

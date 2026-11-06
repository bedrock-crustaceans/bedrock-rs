//! Golden values of the crate's finding helpers, each built by hand the long way
//! (`Diagnostic::new`): the helpers are written over core's builder and must give exactly these.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};

fn at() -> JsonPath {
    JsonPath::of_keys(["a", "b"])
}

fn code() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// The finding of `code` the long way.
fn long(severity: Severity, phase: Phase, location: Location) -> Diagnostic {
    Diagnostic::new(code(), severity, phase, location)
}

fn here(at: JsonPath) -> Location {
    Location::new(None, None, at, None)
}

#[test]
fn the_validate_push_is_a_validate_finding_at_the_member() {
    for severity in [Severity::Info, Severity::Warning, Severity::Error] {
        let mut d = Diagnostics::default();
        crate::validate::push(&mut d, code(), severity, at());
        let told: Vec<_> = d.iter().cloned().collect();
        assert_eq!(told, [long(severity, Phase::Validate, here(at()))]);
    }
}

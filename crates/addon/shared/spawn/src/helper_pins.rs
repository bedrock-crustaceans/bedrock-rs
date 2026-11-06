//! Golden values of the crate's finding helpers, each built by hand the long way
//! (`Diagnostic::new`): the helpers are written over core's builder and must give exactly these.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
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

/// Every field of two findings, `kept` and the detail included.
fn same(left: &Diagnostic, right: &Diagnostic) {
    assert_eq!(left, right);
    assert_eq!(left.kept, right.kept, "kept");
    assert_eq!(left.detail, right.detail, "detail");
}

#[test]
fn the_closing_line_and_the_load_error() {
    let fixed = |text: &'static str| DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Fixed(text)),
    };
    same(
        &crate::load::closing("closing"),
        &Diagnostic::new(
            fixed("closing"),
            Severity::Error,
            Phase::Validate,
            Location::default(),
        ),
    );
    same(
        &crate::load::error(code()),
        &long(Severity::Error, Phase::Parse, Location::default()),
    );
}

#[test]
fn the_reader_finding_is_a_parse_finding_at_the_member() {
    for severity in [Severity::Info, Severity::Warning, Severity::Error] {
        let mut d = Diagnostics::default();
        crate::read::finding(code(), severity, &at(), &mut d);
        let told: Vec<_> = d.iter().cloned().collect();
        assert_eq!(told, [long(severity, Phase::Parse, here(at()))]);
    }
}

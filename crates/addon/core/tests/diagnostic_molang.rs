//! The `Molang` detail (feature `molang`): an `InvalidMolang` carries the
//! compiler's diagnostic, and the content-log line is vanilla's scope chain
//! `<member> | <expression> | <message>`.
#![cfg(feature = "molang")]

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticDetail, JsonPath, Location, Phase, Severity, codes,
};
use molang::compile::{CompileOptions, compile};
use molang::version::MolangVersion;

/// The first compiler diagnostic for `source` on the server side.
fn compiler_diagnostic(source: &str) -> molang::diag::Diagnostic {
    let compiled = compile(source, &CompileOptions::server(MolangVersion::LATEST));
    compiled
        .diagnostics()
        .first()
        .cloned()
        .expect("the source must log a diagnostic")
}

#[test]
fn invalid_molang_renders_the_scope_chain() {
    let source = "q.not_a_query";
    let molang = compiler_diagnostic(source);
    let mut json_path = JsonPath::new();
    json_path.push("minecraft:entity");
    json_path.push("scripts");
    json_path.push("pre_animation");
    let mut diagnostic = Diagnostic::new(
        codes::INVALID_MOLANG.with([source]),
        Severity::Error,
        Phase::Validate,
        Location::new(None, None, json_path, None),
    );
    diagnostic.detail = Some(Box::new(DiagnosticDetail::Molang(molang.clone())));
    assert!(matches!(
        diagnostic.detail.as_deref(),
        Some(DiagnosticDetail::Molang(d)) if d.code() == molang.code()
    ));
    assert_eq!(
        diagnostic.to_string(),
        format!("[error] minecraft:entity/scripts/pre_animation | {source} | {molang}")
    );
}

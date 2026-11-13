//! What the client entity tests share: resource-pack contexts, loading one file, finding
//! diagnostics by code and vanilla text, round trips and the fixture files. The in-memory
//! resource stacks are in `bedrock_addon`'s `tests/client_entity/stacks.rs`.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::PathBuf;
use std::sync::Arc;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::MinEngineVersion;
use bedrock_addon_family_client_entity::ClientEntity;
use uuid::Uuid;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{clean, line, lines, loaded, rejected, round_trip};

/// The path the single-file tests load their document as; vanilla's texts name it.
pub const PATH: &str = "entity/x.entity.json";

/// A resource pack at `min_engine_version` `a.b.c`, the Molang compile check on when the
/// feature is.
pub fn rp(a: u16, b: u16, c: u16) -> LoadContext {
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Resource,
        Some(MinEngineVersion::new(a, b, c)),
        false,
    );
    cx.molang_version =
        bedrock_addon_core::molang::pack_molang_version(cx.min_engine_version.as_ref());
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon_core::molang::MolangCheck::new()));
    }
    let _ = Arc::<()>::new;
    cx
}

/// The fixtures' context: `min_engine_version` 1.21.0.
pub fn cx() -> LoadContext {
    rp(1, 21, 0)
}

/// Loads `text` as a client entity file.
pub fn entity(text: &str, cx: &LoadContext) -> Result<Document<ClientEntity>, Rejected> {
    ClientEntity::parse(&PackPath::from_static(PATH), text.as_bytes(), cx)
}

/// A client entity document of `version` whose description is `description` (a JSON object).
pub fn doc(version: &str, description: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:client_entity": {{"description": {description}}}}}"#
    )
}

/// A description of `demo:ghost` with `members` after the identifier.
pub fn ghost(version: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    doc(
        version,
        &format!(r#"{{"identifier": "demo:ghost"{comma}{members}}}"#),
    )
}

/// The finding with code `name` and severity, if any.
pub fn find<'d>(d: &'d Diagnostics, name: &str, severity: Severity) -> Option<&'d Diagnostic> {
    d.iter()
        .find(|f| f.code.id() == name && f.severity == severity)
}

/// Requires a finding with code `name`, `severity` and vanilla's `text` exactly.
#[track_caller]
pub fn expect(d: &Diagnostics, name: &str, severity: Severity, text: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.id() == name && f.severity == severity && f.code.to_string() == text),
        "no {severity:?} {name} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name` and `severity` at the JSON path `path`.
#[track_caller]
pub fn expect_at(d: &Diagnostics, name: &str, severity: Severity, path: &str) {
    assert!(
        d.iter().any(|f| f.code.id() == name
            && f.severity == severity
            && f.location.json_path.to_string() == path),
        "no {severity:?} {name} at {path} in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name` and `severity` whose text starts with `prefix`.
#[track_caller]
pub fn expect_prefix(d: &Diagnostics, name: &str, severity: Severity, prefix: &str) {
    assert!(
        d.iter().any(|f| f.code.id() == name
            && f.severity == severity
            && f.code.to_string().starts_with(prefix)),
        "no {severity:?} {name} \"{prefix}…\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires no finding with code `name`.
#[track_caller]
pub fn none(d: &Diagnostics, name: &str) {
    assert!(
        d.iter().all(|f| f.code.id() != name),
        "{}",
        lines(d).join("\n")
    );
}

/// Requires exactly one finding at warning level or above: the warning with code `name` at
/// `path`.
#[track_caller]
pub fn only_warning_at(d: &Diagnostics, name: &str, path: &str) {
    expect_at(d, name, Severity::Warning, path);
    let told = d.at_least(Severity::Warning).count();
    assert_eq!(told, 1, "{}", lines(d).join("\n"));
}

/// Requires `name` at information at `path` and nothing at warning level or above.
#[track_caller]
pub fn only_info_at(d: &Diagnostics, name: &str, path: &str) {
    expect_at(d, name, Severity::Info, path);
    let told = d.at_least(Severity::Warning).count();
    assert_eq!(told, 0, "{}", lines(d).join("\n"));
}

/// Requires no finding at all.
#[track_caller]
pub fn silent(d: &Diagnostics) {
    assert!(d.is_empty(), "{}", lines(d).join("\n"));
}

/// The `tests/fixtures` folder of `bedrock_addon`: this file is compiled by the family crate and
/// included by `bedrock_addon`'s own test root, which owns the folder.
fn fixtures_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

/// The fixture directory of `family` (`client_entity`, `atomic_client_entity`).
pub fn fixtures(family: &str) -> PathBuf {
    fixtures_root().join("entity").join(family)
}

/// A fixture file's text.
pub fn fixture(family: &str, name: &str) -> String {
    std::fs::read_to_string(fixtures(family).join(name))
        .unwrap_or_else(|e| panic!("{family}/{name}: {e}"))
}

/// The findings of a load, loaded or not.
pub fn findings<T>(result: Result<Document<T>, Rejected>) -> Diagnostics {
    match result {
        Ok(document) => document.diagnostics().clone(),
        Err(rejected) => rejected.diagnostics,
    }
}

/// One finding as a line with its span and the text the span covers in `text`.
pub fn spanned(f: &Diagnostic, text: &str) -> String {
    let covered = f.location.span.map(|span| {
        let start = span.offset as usize;
        let end = start + span.len as usize;
        (
            span.offset,
            span.len,
            text.get(start..end)
                .unwrap_or("<outside the text>")
                .to_owned(),
        )
    });
    format!("{} @ {covered:?}", line(f))
}

/// Requires the findings of `text`, which holds the numbers `beyond` (each beyond `f32`), to
/// be the findings of the same text with each of them replaced by `within` (in range, and as
/// long), code, path, message and span alike, beside one `InvalidValue` information for each
/// number, whose span is the number's token. `load` gives a text's findings.
#[track_caller]
pub fn clamp_changes_no_other_finding(
    text: &str,
    beyond: &[(&str, &str, &str)],
    load: impl Fn(&str) -> Diagnostics,
) {
    let mut in_range = text.to_owned();
    for (token, within, _) in beyond {
        assert_eq!(token.len(), within.len(), "{token} {within}");
        assert_eq!(text.matches(token).count(), 1, "{token} in {text}");
        in_range = in_range.replace(token, within);
    }
    // The clamp's information is ours and carries no vanilla text.
    let is_clamp = |f: &Diagnostic| {
        f.code.id() == "core/invalid_value"
            && f.severity == Severity::Info
            && f.code.to_string() == "invalid value"
            && beyond
                .iter()
                .any(|(_, _, path)| f.location.json_path.to_string() == *path)
    };
    let found = load(text);
    let others: Vec<String> = found
        .iter()
        .filter(|f| !is_clamp(f))
        .map(|f| spanned(f, text))
        .collect();
    let expected: Vec<String> = load(&in_range)
        .iter()
        .map(|f| spanned(f, &in_range))
        .collect();
    assert_eq!(others, expected, "the findings beside the clamp's");
    for (token, _, path) in beyond {
        let warning = found
            .iter()
            .find(|f| is_clamp(f) && f.location.json_path.to_string() == *path)
            .unwrap_or_else(|| {
                panic!(
                    "no clamp warning at {path} in\n{}",
                    lines(&found).join("\n")
                )
            });
        let span = warning.location.span.expect("the clamp warning has a span");
        let start = span.offset as usize;
        assert_eq!(&text[start..start + span.len as usize], *token, "{path}");
    }
}

/// The `InvalidValue` warning of a clamped number at `path`: its span must be exactly the
/// bytes `at..at + len` of the file, the number's own token.
#[track_caller]
pub fn clamp_warning_is_at(d: &Diagnostics, path: &str, at: usize, len: usize) {
    let warning = d
        .iter()
        .find(|f| {
            f.code.id() == "core/invalid_value"
                && f.severity == Severity::Info
                && f.code.to_string() == "invalid value"
                && f.location.json_path.to_string() == path
        })
        .unwrap_or_else(|| panic!("no clamp warning at {path} in\n{}", lines(d).join("\n")));
    let span = warning.location.span.expect("the clamp warning has a span");
    assert_eq!(
        (span.offset as usize, span.len as usize),
        (at, len),
        "{path}"
    );
}

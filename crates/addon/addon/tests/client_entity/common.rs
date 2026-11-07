//! What the client entity tests share: resource-pack contexts,
//! loading one file, finding diagnostics by code and vanilla text, round trips, in-memory
//! resource stacks and the fixture files.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::PathBuf;
use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::client_entity::ClientEntity;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{PackType, StackSet};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

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
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
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

/// Requires a load to succeed.
#[track_caller]
pub fn loaded<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Document<T> {
    match result {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", lines(&rejected.diagnostics).join("\n")),
    }
}

/// Requires a load to fail; its findings.
#[track_caller]
pub fn rejected<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Diagnostics {
    match result {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings, one line each: severity, code name, path and text.
pub fn lines(d: &Diagnostics) -> Vec<String> {
    d.iter().map(line).collect()
}

/// One finding as a line.
pub fn line(f: &Diagnostic) -> String {
    format!(
        "{:?} {} {} {}",
        f.severity,
        f.code.id(),
        f.location.json_path,
        f.code
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

/// Requires no finding at warning level or above.
#[track_caller]
pub fn clean(d: &Diagnostics) {
    let found: Vec<_> = d.at_least(Severity::Warning).collect();
    assert!(found.is_empty(), "{}", lines(d).join("\n"));
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

/// Writes `document`, reads the text again with `parse` and requires an equal value and an
/// identical second write.
#[track_caller]
pub fn round_trip<T: Content + PartialEq + std::fmt::Debug>(
    document: &Document<T>,
    parse: impl Fn(&str) -> Result<Document<T>, Rejected>,
) -> String {
    let text = document.to_json().unwrap();
    let again = loaded(parse(&text));
    assert_eq!(again.value(), document.value(), "{text}");
    assert_eq!(again.to_json().unwrap(), text);
    text
}

/// One resource pack of an in-memory stack: its manifest `min_engine_version` and its files.
pub struct TestPack<'a> {
    pub min_engine_version: (u16, u16, u16),
    pub files: &'a [(&'a str, &'a str)],
}

/// A pack at `min_engine_version` 1.21.0.
pub fn pack<'a>(files: &'a [(&'a str, &'a str)]) -> TestPack<'a> {
    TestPack {
        min_engine_version: (1, 21, 0),
        files,
    }
}

/// A stack set whose resource stack is `packs`, highest priority first.
pub fn stack(packs: &[TestPack<'_>]) -> StackSet {
    stack_with_unread(packs, &[])
}

/// [`stack`], with the files at `unread` (`p<n>/<path>`, `n` counted from 1) listed and their
/// reads failing.
pub fn stack_with_unread(packs: &[TestPack<'_>], unread: &[&str]) -> StackSet {
    use crate::responder::{Order, Responder};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (index, pack) in packs.iter().enumerate() {
        let n = index as u128 + 1;
        let (a, b, c) = pack.min_engine_version;
        let manifest = format!(
            r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [{a}, {b}, {c}]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
            Uuid::from_u128(n),
            Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
        );
        responder.file(1, &format!("p{n}/manifest.json"), manifest);
        for (path, text) in pack.files {
            responder.file(1, &format!("p{n}/{path}"), (*text).to_string());
        }
        list.push(format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
            Uuid::from_u128(n)
        ));
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let resource = format!("[{}]", list.join(", "));
    let world = WorldPacks::parse(None, Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    for path in unread {
        responder.failed(1, path, std::io::ErrorKind::PermissionDenied);
    }
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}

/// The fixture directory of `family` (`client_entity`, `atomic_client_entity`).
pub fn fixtures(family: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/entity")
        .join(family)
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

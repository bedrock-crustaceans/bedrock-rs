//! What the render test files share: contexts per `min_engine_version`, finding diagnostics by
//! code and vanilla text, round trips, and in-memory resource-pack stacks.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::PathBuf;
use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// A resource pack's context at `min_engine_version` `version` (`None`: one that did not
/// parse, Molang version −1), the Molang compile check on when the feature is.
pub fn context(version: Option<(u16, u16, u16)>) -> LoadContext {
    #[cfg_attr(not(feature = "molang"), allow(unused_mut))]
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Resource,
        version.map(|(a, b, c)| MinEngineVersion::new(a, b, c)),
        false,
    );
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
    }
    let _ = Arc::<()>::new;
    cx
}

/// A resource pack at `min_engine_version` `a.b.c`.
pub fn rp(a: u16, b: u16, c: u16) -> LoadContext {
    context(Some((a, b, c)))
}

/// A resource pack at 1.21.100 without the Molang compile check.
pub fn unchecked() -> LoadContext {
    #[cfg_attr(not(feature = "molang"), allow(unused_mut))]
    let mut cx = rp(1, 21, 100);
    #[cfg(feature = "molang")]
    {
        cx.molang = None;
    }
    cx
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

/// One finding as a line: severity, code name, path and text.
pub fn line(f: &Diagnostic) -> String {
    let detail = f.detail.as_deref().map(ToString::to_string);
    format!(
        "{:?} {} {} {}{}",
        f.severity,
        f.code.name(),
        f.location.json_path,
        f.code,
        detail.unwrap_or_default()
    )
}

/// The findings, one line each.
pub fn lines(d: &Diagnostics) -> Vec<String> {
    d.iter().map(line).collect()
}

/// The finding with code `name` and severity, if any.
pub fn find<'d>(d: &'d Diagnostics, name: &str, severity: Severity) -> Option<&'d Diagnostic> {
    d.iter()
        .find(|f| f.code.name() == name && f.severity == severity)
}

/// How many findings have code `name` and `severity`.
pub fn count(d: &Diagnostics, name: &str, severity: Severity) -> usize {
    d.iter()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .count()
}

/// Requires a finding with code `name`, `severity` and vanilla's `text` exactly.
#[track_caller]
pub fn expect(d: &Diagnostics, name: &str, severity: Severity, text: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.name() == name && f.severity == severity && f.code.to_string() == text),
        "no {severity:?} {name} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name`, `severity` and `text` at the JSON path `at`.
#[track_caller]
pub fn expect_at(d: &Diagnostics, name: &str, severity: Severity, text: &str, at: &str) {
    assert!(
        d.iter().any(|f| f.code.name() == name
            && f.severity == severity
            && f.code.to_string() == text
            && f.location.json_path.to_string() == at),
        "no {severity:?} {name} \"{text}\" at {at} in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name` and `severity` whose text starts with `prefix`.
#[track_caller]
pub fn expect_prefix(d: &Diagnostics, name: &str, severity: Severity, prefix: &str) {
    assert!(
        d.iter().any(|f| f.code.name() == name
            && f.severity == severity
            && f.code.to_string().starts_with(prefix)),
        "no {severity:?} {name} \"{prefix}…\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding whose whole line (code text and detail) contains `needle`.
#[track_caller]
pub fn expect_line(d: &Diagnostics, name: &str, severity: Severity, needle: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.name() == name && f.severity == severity && line(f).contains(needle)),
        "no {severity:?} {name} with \"{needle}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires no finding at warning level or above.
#[track_caller]
pub fn clean(d: &Diagnostics) {
    let found: Vec<_> = d.at_least(Severity::Warning).collect();
    assert!(found.is_empty(), "{}", lines(d).join("\n"));
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
) {
    let text = document.to_json().unwrap();
    let again = loaded(parse(&text));
    assert_eq!(again.value(), document.value(), "{text}");
    assert_eq!(again.to_json().unwrap(), text);
}

/// A format-2 resource-pack manifest with UUID `n` and `min_engine_version` `version` (JSON).
pub fn manifest_at(n: u128, version: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": {version} }},
  "modules": [{{ "type": "resources", "uuid": "{}", "version": [1, 0, 0] }}]
}}"#,
        Uuid::from_u128(n),
        Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

/// A stack set read from in-memory resource packs, each a list of `(path, text)` files,
/// highest priority first (`p0`, `p1`, …); each pack gets a manifest at `min_engine_version`
/// 1.21.100 unless it brings a `manifest.json` (pack `i` has UUID `i + 1`).
pub fn stacks(resource: &[&[(&str, &str)]]) -> bedrock_addon::pack::StackSet {
    stacks_with(resource, false)
}

/// [`stacks`], keeping the shadowed definitions as layers when `keep_shadowed`.
pub fn stacks_with(
    resource: &[&[(&str, &str)]],
    keep_shadowed: bool,
) -> bedrock_addon::pack::StackSet {
    use crate::responder::{Order, Responder};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (index, files) in resource.iter().enumerate() {
        let n = index as u128 + 1;
        if !files.iter().any(|(path, _)| *path == "manifest.json") {
            responder.file(
                1,
                &format!("p{index}/manifest.json"),
                manifest_at(n, "[1, 21, 100]"),
            );
        }
        for (path, text) in *files {
            responder.file(1, &format!("p{index}/{path}"), (*text).to_string());
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
    let world = WorldPacks::parse(Some(b"[]"), Some(resource.as_bytes())).unwrap();
    let options = LoadOptions {
        #[cfg(feature = "molang")]
        molang: Some(Arc::new(bedrock_addon::molang::MolangCheck::new())),
        keep_shadowed,
        ..LoadOptions::default()
    };
    let set = repository.stacks(&world, options);
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}

/// The fixture directory of `family` (`render_controller`, `fog`).
pub fn fixtures(family: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/render")
        .join(family)
}

/// A fixture file's text.
pub fn fixture(family: &str, name: &str) -> String {
    std::fs::read_to_string(fixtures(family).join(name))
        .unwrap_or_else(|e| panic!("{family}/{name}: {e}"))
}

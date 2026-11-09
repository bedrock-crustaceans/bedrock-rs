//! What the dialogue tests share: the fixture packs, their `expected.json`, and loading one
//! file.
//!
//! A fixture is `tests/fixtures/dialogue/dialogue/<name>/`: either one behaviour-pack tree
//! (`dialogue/…`) or, for a stack, `stack/<n>_<pack>/dialogue/…` with `0_…` the highest
//! priority. A pack folder may hold its own `manifest.json`; otherwise one is generated
//! (`min_engine_version` 1.26.20). `expected.json` lists what the load gives:
//!
//! ```json
//! { "scenes": ["a", "b"], "rejected": 1,
//!   "diagnostics": [ { "code": "MissingRequiredField", "severity": "error",
//!                      "path": "dialogue/a.json", "text": "…" } ] }
//! ```
//!
//! `scenes` are the effective tags in load order. The list is exhaustive at every severity:
//! each finding of the registry (infos included) must match one listed entry and each listed
//! entry one finding; `path`, `json_path` and `text` are compared when given.

#![allow(dead_code, reason = "each test module uses part of the helpers")]
#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Registry, Rejected};
use bedrock_addon::families::dialogue::{DialogueFile, DialogueScene};
use bedrock_addon::id::DialogueSceneTag;
use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// The fixture directory `tests/fixtures/dialogue/dialogue/<name>`.
pub fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dialogue/dialogue")
        .join(name)
}

/// A fixture file's bytes.
pub fn fixture(rest: &str) -> Vec<u8> {
    let path = fixture_dir(rest);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A fixture file's text.
pub fn fixture_text(rest: &str) -> String {
    String::from_utf8(fixture(rest)).expect("UTF-8")
}

/// A format-2 behaviour-pack manifest with `min_engine_version` 1.26.20.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// Every file below `dir`, by its path relative to `base`.
fn files(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, base, out);
        } else {
            let relative = path.strip_prefix(base).unwrap().to_string_lossy();
            let relative = relative.replace('\\', "/");
            if relative != "expected.json" {
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
}

/// A pack of `origin` from its files (a manifest is added unless one is among them).
pub fn pack_of(uuid: u128, files: Vec<(String, Vec<u8>)>, origin: PackOrigin) -> Pack {
    let mut all = Vec::new();
    if !files.iter().any(|(path, _)| path == "manifest.json") {
        all.push((
            PackPath::new("manifest.json").unwrap(),
            manifest(uuid, &format!("pack {uuid}")).into_bytes(),
        ));
    }
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes)),
    );
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A user pack from its files.
pub fn pack(uuid: u128, files: Vec<(String, Vec<u8>)>) -> Pack {
    pack_of(uuid, files, PackOrigin::User)
}

/// The packs of a fixture, top first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    let dir = fixture_dir(name);
    let stack = dir.join("stack");
    if stack.is_dir() {
        let mut members: Vec<PathBuf> = std::fs::read_dir(&stack)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        members.sort();
        members
            .iter()
            .enumerate()
            .map(|(i, member)| {
                let mut found = Vec::new();
                files(member, member, &mut found);
                pack(10 * (i as u128 + 1), found)
            })
            .collect()
    } else {
        let mut found = Vec::new();
        files(&dir, &dir, &mut found);
        vec![pack(10, found)]
    }
}

/// A behaviour stack of `packs`, top first.
pub fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).expect("a behaviour stack")
}

/// A behaviour stack of `packs`, top first, that keeps shadowed definitions.
pub fn stack_keeping_shadowed(packs: Vec<Pack>) -> PackStack<Behavior> {
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

/// Loads a fixture and checks it against its `expected.json`; the stack, for further checks.
#[track_caller]
pub fn load_fixture(name: &str) -> PackStack<Behavior> {
    let stack = stack_keeping_shadowed(fixture_packs(name));
    let expected: Value = facet_json::from_slice(&fixture(&format!("{name}/expected.json")))
        .expect("expected.json is JSON");
    check_expected(name, stack.dialogue(), &expected);
    stack
}

/// The member `key` of an object.
fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value.as_object()?.get(key)
}

/// The string `key` of an object.
fn string(value: &Value, key: &str) -> Option<String> {
    member(value, key)?
        .as_string()
        .map(|s| s.as_str().to_owned())
}

/// The strings of the array `key`.
fn strings(value: &Value, key: &str) -> Vec<String> {
    member(value, key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_string().map(|s| s.as_str().to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

/// The severity word of `expected.json`.
fn severity(word: &str) -> Severity {
    match word {
        "info" => Severity::Info,
        "warning" => Severity::Warning,
        "error" => Severity::Error,
        other => panic!("unknown severity {other}"),
    }
}

/// Whether `finding` is the expected entry.
fn matches(finding: &Diagnostic, entry: &Value) -> bool {
    let code = string(entry, "code").expect("an expected code");
    let level = severity(&string(entry, "severity").expect("an expected severity"));
    let path_ok = string(entry, "path").is_none_or(|path| {
        finding
            .location
            .path
            .as_ref()
            .is_some_and(|p| p.as_str() == path)
    });
    let json_path_ok = string(entry, "json_path")
        .is_none_or(|json_path| finding.location.json_path.to_string() == json_path);
    let text_ok = string(entry, "text").is_none_or(|text| finding.code.to_string() == text);
    finding.code.id() == code && finding.severity == level && path_ok && json_path_ok && text_ok
}

/// Compares the registry with the expected scene tags (in load order), rejected count and
/// findings.
#[track_caller]
pub fn check_expected(name: &str, registry: &Registry<DialogueScene>, expected: &Value) {
    let tags: Vec<String> = registry
        .iter()
        .map(|(tag, _)| tag.as_str().to_owned())
        .collect();
    let findings: Vec<&Diagnostic> = registry.diagnostics().collect();
    assert_eq!(
        tags,
        strings(expected, "scenes"),
        "{name}: scenes\nall: {}",
        describe(&findings)
    );
    if let Some(rejected) = member(expected, "rejected").and_then(Value::as_number) {
        assert_eq!(
            registry.rejected().len() as u64,
            rejected.to_u64().unwrap(),
            "{name}: rejected\nall: {}",
            describe(&findings)
        );
    }
    let entries: Vec<Value> = member(expected, "diagnostics")
        .and_then(Value::as_array)
        .map(|a| a.iter().cloned().collect())
        .unwrap_or_default();
    let mut used = vec![false; entries.len()];
    for finding in &findings {
        let found = entries
            .iter()
            .enumerate()
            .find(|(i, entry)| !used[*i] && matches(finding, entry));
        match found {
            Some((i, _)) => used[i] = true,
            None => panic!(
                "{name}: unexpected {} {} at {:?} {}: {:?}\nall: {}",
                finding.severity,
                finding.code.id(),
                finding.location.path.as_ref().map(PackPath::as_str),
                finding.location.json_path,
                finding.code.to_string(),
                describe(&findings),
            ),
        }
    }
    for (entry, used) in entries.iter().zip(used) {
        assert!(
            used,
            "{name}: expected {entry:?} not found\nall: {}",
            describe(&findings)
        );
    }
}

/// The findings, one per line.
pub fn describe(findings: &[&Diagnostic]) -> String {
    findings.iter().fold(String::new(), |mut out, f| {
        write!(
            out,
            "\n  {} {} {:?} {} {:?}",
            f.severity,
            f.code.id(),
            f.location.path.as_ref().map(PackPath::as_str),
            f.location.json_path,
            f.code.to_string()
        )
        .unwrap();
        out
    })
}

/// The effective scene `tag` of a registry.
#[track_caller]
pub fn scene<'r>(registry: &'r Registry<DialogueScene>, tag: &str) -> &'r Document<DialogueScene> {
    registry
        .get(&DialogueSceneTag::parse(tag))
        .unwrap_or_else(|| panic!("no scene {tag}"))
}

/// A behaviour-pack context with `min_engine_version` 1.26.20, no toggles.
pub fn context() -> LoadContext {
    context_at(Some(MinEngineVersion::new(1, 26, 20)))
}

/// A behaviour-pack context with this `min_engine_version`.
pub fn context_at(min_engine_version: Option<MinEngineVersion>) -> LoadContext {
    LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        bedrock_addon::pack::PackType::Behavior,
        min_engine_version,
        false,
    )
}

/// Loads `text` as a dialogue file at `path`.
pub fn load(path: &str, text: &str) -> Result<Document<DialogueFile>, Rejected> {
    DialogueFile::parse(
        &PackPath::new(path).expect("a pack path"),
        text.as_bytes(),
        &context(),
    )
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str) -> Document<DialogueFile> {
    match load("dialogue/a.json", text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected(text: &str) -> Diagnostics {
    match load("dialogue/a.json", text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The code names of `diagnostics` at `severity`.
pub fn names(diagnostics: &Diagnostics, severity: Severity) -> Vec<&'static str> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .map(|d| d.code.id())
        .collect()
}

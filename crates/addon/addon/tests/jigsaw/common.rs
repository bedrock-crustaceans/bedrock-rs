//! What the jigsaw tests share: the fixture packs, their `expected.json`, and loading one
//! document.
//!
//! A fixture is `tests/fixtures/worldgen/jigsaw/<name>/`: either one behaviour-pack tree
//! (`worldgen/…`) or, for a stack, `stack/<n>_<pack>/worldgen/…` with `0_…` the highest
//! priority. `expected.json` lists what the load gives (an expected file listing
//! diagnostics (code, level, vanilla text) and the resulting identifier → document maps; JSON
//! because the crate has no TOML reader):
//!
//! ```json
//! { "loaded": { "structures": ["demo:a"], "pools": [], "structure_sets": [], "processors": [],
//!               "metadata": ["worldgen/metadata/a.dat"] },
//!   "diagnostics": [ { "code": "UnrecognizedField", "severity": "warning",
//!                      "path": "worldgen/structures/a.json", "text": "…" } ] }
//! ```
//!
//! Every finding at warning level or above must match one listed entry, and every listed entry
//! (info ones included) one finding; `path` and `text` are compared when given.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::jigsaw::Jigsaw;
use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// The fixture directory `tests/fixtures/worldgen/jigsaw/<name>`.
pub fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/worldgen/jigsaw")
        .join(name)
}

/// A fixture file's bytes.
pub fn fixture(rest: &str) -> Vec<u8> {
    let path = fixture_dir(rest);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A format-2 behaviour-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// Every file below `dir`, by its path relative to `dir`.
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

/// A pack from its files (a manifest is added).
pub fn pack(uuid: u128, files: Vec<(String, Vec<u8>)>) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(uuid, &format!("pack {uuid}")).into_bytes(),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes)),
    );
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
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

/// Loads a fixture and checks it against its `expected.json`; the stack, for further checks.
#[track_caller]
pub fn load_fixture(name: &str) -> PackStack<Behavior> {
    let stack = stack(fixture_packs(name));
    let expected: Value = facet_json::from_slice(&fixture(&format!("{name}/expected.json")))
        .expect("expected.json is JSON");
    check_expected(name, stack.jigsaw(), &expected);
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
    finding.code.name() == code && finding.severity == level && path_ok && json_path_ok && text_ok
}

/// Compares the group with the expected identifiers and findings.
#[track_caller]
pub fn check_expected(name: &str, jigsaw: &Jigsaw, expected: &Value) {
    let loaded = member(expected, "loaded").expect("expected.json has `loaded`");
    let ids = |key: &str, actual: Vec<String>| {
        let mut want = strings(loaded, key);
        want.sort();
        let mut actual = actual;
        actual.sort();
        assert_eq!(actual, want, "{name}: {key}");
    };
    ids(
        "structures",
        jigsaw
            .structures
            .iter()
            .map(|(id, _)| id.as_str().to_owned())
            .collect(),
    );
    ids(
        "pools",
        jigsaw
            .pools
            .iter()
            .map(|(id, _)| id.as_str().to_owned())
            .collect(),
    );
    ids(
        "structure_sets",
        jigsaw
            .structure_sets
            .iter()
            .map(|(id, _)| id.as_str().to_owned())
            .collect(),
    );
    ids(
        "processors",
        jigsaw
            .processors
            .iter()
            .map(|(id, _)| id.as_str().to_owned())
            .collect(),
    );
    ids(
        "metadata",
        jigsaw
            .metadata
            .iter()
            .map(|(path, _)| path.as_str().to_owned())
            .collect(),
    );
    let entries: Vec<Value> = member(expected, "diagnostics")
        .and_then(Value::as_array)
        .map(|a| a.iter().cloned().collect())
        .unwrap_or_default();
    let findings: Vec<&Diagnostic> = jigsaw.diagnostics().collect();
    let mut used = vec![false; entries.len()];
    for finding in &findings {
        let found = entries
            .iter()
            .enumerate()
            .find(|(i, entry)| !used[*i] && matches(finding, entry));
        match found {
            Some((i, _)) => used[i] = true,
            None if finding.severity >= Severity::Warning => panic!(
                "{name}: unexpected {} {} at {:?} {}: {}\nall: {}",
                finding.severity,
                finding.code.name(),
                finding.location.path.as_ref().map(PackPath::as_str),
                finding.location.json_path,
                finding.code,
                describe(&findings),
            ),
            None => {}
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
            "\n  {} {} {:?} {} {}",
            f.severity,
            f.code.name(),
            f.location.path.as_ref().map(PackPath::as_str),
            f.location.json_path,
            f.code
        )
        .unwrap();
        out
    })
}

/// A behaviour-pack context at the engine version, no toggles.
pub fn context() -> LoadContext {
    LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        bedrock_addon::pack::PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 20)),
        false,
    )
}

/// Loads `text` as the document of type `T` at `path`.
pub fn load<T: Content>(path: &str, text: &str) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::new(path).expect("a pack path"),
        text.as_bytes(),
        &context(),
    )
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads<T: Content>(path: &str, text: &str) -> Document<T> {
    match load::<T>(path, text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected<T: Content + std::fmt::Debug>(path: &str, text: &str) -> Diagnostics {
    match load::<T>(path, text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The code names of `diagnostics` at `severity`.
pub fn names(diagnostics: &Diagnostics, severity: Severity) -> Vec<&'static str> {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .map(|d| d.code.name())
        .collect()
}

/// A document of `version` with `payload` under `key`.
pub fn document(version: &str, key: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "{key}": {payload}}}"#)
}

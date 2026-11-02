//! What the POI tests share: the fixture packs and loading one document.
//!
//! A fixture is `tests/fixtures/poi/poi/<name>/`: one behaviour-pack tree (`poi/…`) or, for a
//! stack, `stack/<n>_<pack>/poi/…` with `0_…` the highest priority. Unless a test says
//! otherwise the `poi` experiment is on.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::{Experiment, Experiments, LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// The fixture directory `tests/fixtures/poi/poi/<name>`.
pub fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/poi/poi")
        .join(name)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A format-2 behaviour-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 40]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
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
            out.push((relative.replace('\\', "/"), std::fs::read(&path).unwrap()));
        }
    }
}

/// A pack of `origin` from its files (a manifest is added).
pub fn pack_from(uuid: u128, origin: PackOrigin, files: Vec<(String, Vec<u8>)>) -> Pack {
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
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A user pack from `(path, text)` pairs.
pub fn pack(uuid: u128, files: &[(&str, &str)]) -> Pack {
    pack_from(
        uuid,
        PackOrigin::User,
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
            .collect(),
    )
}

/// The packs of a fixture, top first, each of `origin`.
pub fn fixture_packs_from(name: &str, origin: PackOrigin) -> Vec<Pack> {
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
                pack_from(10 * (i as u128 + 1), origin, found)
            })
            .collect()
    } else {
        let mut found = Vec::new();
        files(&dir, &dir, &mut found);
        vec![pack_from(10, origin, found)]
    }
}

/// The user packs of a fixture, top first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    fixture_packs_from(name, PackOrigin::User)
}

/// The world options with the `poi` experiment on.
pub fn poi_options() -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

/// A behaviour stack of `packs`, top first, loaded with `options`.
pub fn stack_with(packs: Vec<Pack>, options: LoadOptions) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

/// A behaviour stack of `packs`, top first, with the `poi` experiment on.
pub fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    stack_with(packs, poi_options())
}

/// The stack of a fixture, with the `poi` experiment on.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    stack(fixture_packs(name))
}

/// A behaviour-pack context loaded with `options`.
pub fn context_with(options: &LoadOptions, base_game: bool) -> LoadContext {
    LoadContext::for_pack(
        options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        base_game,
    )
}

/// A behaviour-pack context at the engine version with the `poi` experiment on.
pub fn context() -> LoadContext {
    context_with(&poi_options(), false)
}

/// Loads `text` as the document of type `T` at `poi/<file>` with `cx`.
pub fn load_with<T: Content>(text: &str, cx: &LoadContext) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::new("poi/document.json").expect("a pack path"),
        text.as_bytes(),
        cx,
    )
}

/// Loads `text` as the document of type `T`, the `poi` experiment on.
pub fn load<T: Content>(text: &str) -> Result<Document<T>, Rejected> {
    load_with(text, &context())
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads<T: Content>(text: &str) -> Document<T> {
    match load::<T>(text) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", describe(rejected.diagnostics.iter())),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected<T: Content + std::fmt::Debug>(text: &str) -> Diagnostics {
    match load::<T>(text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Loads `text`, requires it to be rejected and returns the code names of its errors.
#[track_caller]
pub fn errors<T: Content + std::fmt::Debug>(text: &str) -> Vec<&'static str> {
    names(rejected::<T>(text).iter(), Severity::Error)
}

/// Whether `finding` is the closing error "Failed to load POI Type" every failed file ends
/// with. A test of one rule looks at the findings before it; `rules.rs` pins the
/// closing line itself.
pub fn is_closing(finding: &Diagnostic) -> bool {
    matches!(
        finding.code,
        bedrock_addon::diagnostic::DiagnosticCode::DocumentRejected { .. }
    )
}

/// The code names of the findings at `severity`, the closing line left out.
pub fn names<'a>(
    findings: impl IntoIterator<Item = &'a Diagnostic>,
    severity: Severity,
) -> Vec<&'static str> {
    findings
        .into_iter()
        .filter(|d| d.severity == severity && !is_closing(d))
        .map(|d| d.code.name())
        .collect()
}

/// The findings, one per line: severity, code, file, JSON path, text.
pub fn describe<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> String {
    findings
        .into_iter()
        .map(|f| {
            format!(
                "\n  {} {} {:?} {} {}",
                f.severity,
                f.code.name(),
                f.location.path.as_ref().map(PackPath::as_str),
                f.location.json_path,
                f.code
            )
        })
        .collect()
}

/// A `minecraft:poi_block` document of `version` with `payload`.
pub fn block_document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:poi_block": {payload}}}"#)
}

/// A `minecraft:poi_block` document at 1.26.40 with `payload`.
pub fn block(payload: &str) -> String {
    block_document("1.26.40", payload)
}

/// A `minecraft:poi_tag` document at 1.26.40 with `payload`.
pub fn tag(payload: &str) -> String {
    format!(r#"{{"format_version": "1.26.40", "minecraft:poi_tag": {payload}}}"#)
}

/// A minimal type document `identifier` of `blocks` (a JSON array) with `rest` (further
/// members, each ending in a comma) before `blocks`.
pub fn block_of(identifier: &str, rest: &str, blocks: &str) -> String {
    block(&format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, {rest} "blocks": {blocks}}}"#
    ))
}

/// A tag document `identifier` with `rest` (further members, each starting with a comma).
pub fn tag_of(identifier: &str, rest: &str) -> String {
    tag(&format!(
        r#"{{"description": {{"identifier": "{identifier}"}}{rest}}}"#
    ))
}

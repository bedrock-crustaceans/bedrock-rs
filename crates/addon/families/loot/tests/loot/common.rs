//! Loading loot table text outside a stack, and reading the findings.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_loot::LootTable;

/// The `tests/fixtures` folder of `bedrock_addon`: this file is compiled by the family crate and
/// included by `bedrock_addon`'s own test root, which owns the folder.
fn fixtures_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

/// The bytes of `tests/fixtures/loot/loot/<name>`.
pub fn fixture(name: &str) -> Vec<u8> {
    let path = fixtures_root().join("loot/loot").join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The path every test table is loaded from.
pub fn table_path() -> PackPath {
    PackPath::new("loot_tables/test.json").unwrap()
}

/// A detached context at the engine version.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// A detached context with the engine at `major.minor.patch` and Upcoming Creator Features as
/// given.
pub fn cx_at(major: u16, minor: u16, patch: u16, ucf: bool) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.engine_version = FormatVersion::new(major, minor, patch);
    cx.upcoming_creator_features = ucf;
    cx
}

/// Loads `text` in `cx`.
pub fn load_in(text: &str, cx: &LoadContext) -> Result<Document<LootTable>, Rejected> {
    LootTable::parse(&table_path(), text.as_bytes(), cx)
}

/// Loads `text`; it must load.
pub fn load(text: &str) -> Document<LootTable> {
    load_in(text, &cx()).unwrap_or_else(|rejected| panic!("rejected: {rejected}"))
}

/// Loads a fixture file; it must load.
pub fn load_fixture(name: &str) -> Document<LootTable> {
    let bytes = fixture(name);
    LootTable::parse(&table_path(), &bytes, &cx())
        .unwrap_or_else(|rejected| panic!("{name} rejected: {rejected}"))
}

/// The findings of a document.
pub fn findings(document: &Document<LootTable>) -> Vec<&Diagnostic> {
    document.diagnostics().iter().collect()
}

/// The findings of `severity` or worse.
pub fn at_least(document: &Document<LootTable>, severity: Severity) -> Vec<&Diagnostic> {
    document
        .diagnostics()
        .iter()
        .filter(|d| d.severity >= severity)
        .collect()
}

/// The finding's JSON path as text (`pools/0/entries/1`).
pub fn path(diagnostic: &Diagnostic) -> String {
    diagnostic.location.json_path.to_string()
}

/// The code's text as `Display` renders it (vanilla's text where it has one).
pub fn text(code: &Code) -> String {
    code.to_string()
}

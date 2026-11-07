//! Helpers of the structure tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_structure::Structure;

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

/// The fixture directory.
pub fn dir() -> String {
    fixtures_root()
        .join("structure/structure")
        .to_string_lossy()
        .into_owned()
}

/// A file's bytes under the fixture directory.
pub fn read(name: &str) -> Vec<u8> {
    let path = format!("{}/{name}", dir());
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The NBT tree of an SNBT text (lines starting with `//` are comments).
pub fn tree(snbt: &str) -> nbtx::Value {
    let text: String = snbt
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    nbtx::from_string::<nbtx::Value>(&text).unwrap_or_else(|e| panic!("{e}\n{text}"))
}

/// Little-endian NBT bytes of an SNBT text, with an empty root name: what `/structure save`
/// writes.
pub fn bytes(snbt: &str) -> Vec<u8> {
    nbtx::to_le_bytes(&tree(snbt)).expect("encodes")
}

/// The `.mcstructure` bytes of fixture `name`: the capture `<name>.mcstructure` if there is
/// one, else `<name>.snbt` encoded.
pub fn fixture(name: &str) -> Vec<u8> {
    let capture = format!("{}/{name}.mcstructure", dir());
    if std::path::Path::new(&capture).exists() {
        return read(&format!("{name}.mcstructure"));
    }
    bytes(&String::from_utf8(read(&format!("{name}.snbt"))).unwrap())
}

/// A third-party behaviour pack at the engine version.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// Loads `bytes` as a structure file.
pub fn load(bytes: &[u8]) -> Result<Document<Structure>, Rejected> {
    Structure::parse(
        &PackPath::from_static("structures/demo/test.mcstructure"),
        bytes,
        &cx(),
    )
}

/// Loads `bytes`, which must load.
#[track_caller]
pub fn loads(bytes: &[u8]) -> Document<Structure> {
    match load(bytes) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `bytes`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(bytes: &[u8]) -> Diagnostics {
    match load(bytes) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The one finding of a rejected file, which must be an error with `code` (a code name) and
/// vanilla's `text`.
#[track_caller]
pub fn fails_with(bytes: &[u8], code: &str, text: &str) -> Diagnostic {
    let d = rejects(bytes);
    let found: Vec<&Diagnostic> = d.iter().collect();
    assert_eq!(found.len(), 1, "one finding: {d:#?}");
    let finding = found[0];
    assert_eq!(finding.severity, Severity::Error, "{finding:#?}");
    assert_eq!(finding.code.id(), code, "{finding:#?}");
    assert_eq!(finding.code.to_string(), text, "{finding:#?}");
    finding.clone()
}

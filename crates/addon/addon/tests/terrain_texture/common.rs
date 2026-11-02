//! The fixture packs of `tests/fixtures/texture/<family>/`: each fixture is a directory with
//! one sub-directory per pack, each holding the pack's files below its root. The helpers add a
//! manifest to each pack, so a fixture holds only what its row is about. The texture families
//! share this file.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;

/// The directory of fixture `name` of `family`.
pub fn fixture_dir(family: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/texture")
        .join(family)
        .join(name)
}

/// The bytes of `file` of fixture `name` of `family`.
pub fn fixture_file(family: &str, name: &str, file: &str) -> Vec<u8> {
    let path = fixture_dir(family, name).join(file);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
pub fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A format-2 manifest with UUID `n` and one module of `module` type; `subpacks` are folder
/// names.
pub fn manifest(n: u128, module: &str, subpacks: &[&str]) -> String {
    let subpacks: Vec<String> = subpacks
        .iter()
        .map(|folder| {
            format!(r#"{{ "folder_name": "{folder}", "name": "{folder}", "memory_tier": 0 }}"#)
        })
        .collect();
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}],
  "subpacks": [{}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
        subpacks.join(", "),
    )
}

/// A resource pack of `files` (path, bytes) with UUID `n`, opened as coming from `origin`.
pub fn pack_of(n: u128, files: &[(&str, &[u8])], origin: PackOrigin) -> Pack {
    pack_with(n, "resources", &[], files, origin)
}

/// A pack of `files` with UUID `n`, one module of `module` type and the subpack folders
/// `subpacks`, opened as coming from `origin`.
pub fn pack_with(
    n: u128,
    module: &str,
    subpacks: &[&str],
    files: &[(&str, &[u8])],
    origin: PackOrigin,
) -> Pack {
    // A fixture that is about its manifest (a capability) ships its own.
    let mut all = Vec::new();
    if !files.iter().any(|(path, _)| *path == "manifest.json") {
        let manifest = manifest(n, module, subpacks);
        all.push((
            PackPath::new("manifest.json").unwrap(),
            manifest.into_bytes(),
        ));
    }
    for (path, bytes) in files {
        all.push((PackPath::new(path).expect("a pack path"), bytes.to_vec()));
    }
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).unwrap_or_else(|error| panic!("pack {n}: {error}"))
}

/// The pack in the directory `dir`, with UUID `n`.
pub fn pack_in(dir: &Path, n: u128, origin: PackOrigin) -> Pack {
    let files = files_below(dir);
    let files: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
        .collect();
    pack_of(n, &files, origin)
}

/// The pack directories of fixture `name` of `family`, sorted by name.
pub fn pack_dirs(family: &str, name: &str) -> Vec<PathBuf> {
    let dir = fixture_dir(family, name);
    let mut packs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    packs.sort();
    assert!(!packs.is_empty(), "{name} has no pack");
    packs
}

/// A resource stack of `packs`, the first on top.
pub fn stack_of(packs: Vec<Pack>) -> PackStack<Resource> {
    PackStack::<Resource>::of(packs, LoadOptions::default()).expect("resource packs")
}

/// The resource stack of fixture `name` of `family`: its packs `p0`, `p1`, …, `p0` on top.
pub fn fixture_stack(family: &str, name: &str) -> PackStack<Resource> {
    let packs = pack_dirs(family, name)
        .iter()
        .enumerate()
        .map(|(n, dir)| pack_in(dir, n as u128 + 1, PackOrigin::User))
        .collect();
    stack_of(packs)
}

/// A resource stack of in-line packs, the first on top.
pub fn stack_of_files(packs: &[&[(&str, &[u8])]]) -> PackStack<Resource> {
    let packs = packs
        .iter()
        .enumerate()
        .map(|(n, files)| pack_of(n as u128 + 1, files, PackOrigin::User))
        .collect();
    stack_of(packs)
}

/// What a test compares of a finding: the code's name, the severity, the place in the document
/// and the rendered text.
pub type Rendered = (&'static str, Severity, String, String);

/// Every finding of `diagnostics`, in order.
pub fn rendered<'a>(diagnostics: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Rendered> {
    diagnostics
        .into_iter()
        .map(|finding| {
            (
                finding.code.name(),
                finding.severity,
                finding.location.json_path.to_string(),
                finding.code.to_string(),
            )
        })
        .collect()
}

/// One expected finding.
pub fn finding(code: &'static str, severity: Severity, at: &str, text: &str) -> Rendered {
    (code, severity, at.to_owned(), text.to_owned())
}

/// The index in the stack of the pack a finding names.
pub fn pack_index(finding: &Diagnostic) -> Option<usize> {
    finding
        .location
        .pack
        .as_ref()
        .and_then(bedrock_addon::pack::PackRef::index)
}

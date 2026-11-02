//! The fixture packs of `tests/fixtures/function/function/`: each fixture is a directory with
//! one sub-directory per pack, `p0` (the highest priority), `p1`, …, each holding a
//! `manifest.json` with the pack's `min_engine_version` and a `functions/` tree.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::Document;
use bedrock_addon::families::function::Function;
use bedrock_addon::id::FunctionPath;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;

/// The directory of fixture `name`.
pub fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/function/function")
        .join(name)
}

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
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

/// The pack in the directory `dir`.
pub fn pack_in(dir: &Path) -> Pack {
    let files = files_below(dir)
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
}

/// The packs of fixture `name`, `p0` first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    let dir = fixture_dir(name);
    let mut packs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    packs.sort();
    assert!(!packs.is_empty(), "{name} has no pack");
    packs.iter().map(|dir| pack_in(dir)).collect()
}

/// The behaviour stack of fixture `name`: its packs, `p0` on top.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(fixture_packs(name), LoadOptions::default()).expect("data packs")
}

/// The loaded function `name` of `stack`.
pub fn function<'a>(stack: &'a PackStack<Behavior>, name: &str) -> &'a Document<Function> {
    stack
        .functions()
        .get(&FunctionPath::parse(name))
        .unwrap_or_else(|| panic!("no function `{name}`"))
}

/// The names of the stack's functions, in load order.
pub fn function_names(stack: &PackStack<Behavior>) -> Vec<&str> {
    stack
        .functions()
        .iter()
        .map(|(_, function)| function.name.as_str())
        .collect()
}

/// The trimmed command lines of `function`.
pub fn commands(function: &Function) -> Vec<&str> {
    function
        .commands
        .iter()
        .map(|line| line.raw.trim())
        .collect()
}

/// What a test compares of a finding: the code's name, the severity and the rendered text.
pub type Rendered = (&'static str, Severity, String);

/// Every finding of `diagnostics`, in order, as its code name, severity and text.
pub fn rendered<'a>(diagnostics: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Rendered> {
    diagnostics
        .into_iter()
        .map(|finding| {
            (
                finding.code.name(),
                finding.severity,
                finding.code.to_string(),
            )
        })
        .collect()
}

/// One expected finding.
pub fn finding(code: &'static str, severity: Severity, text: &str) -> Rendered {
    (code, severity, text.to_owned())
}

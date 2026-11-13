//! Fixture loading, load contexts and finding matchers for the recipe tests.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon_core::diagnostic::{CodeMeta, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::pack_molang_version;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use bedrock_addon_family_recipe::{Recipe, Shaped};

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::clean;

/// The fixture directory of the family. The file is included by the family crate's tests and by
/// `bedrock_addon`'s, which owns the `tests/fixtures` folder, so the folder is looked for in the
/// package that compiles it first.
pub fn fixtures() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own.join("recipe/recipe")
    } else {
        manifest.join("../../addon/tests/fixtures/recipe/recipe")
    }
}

/// The bytes of fixture `name`.
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// A third-party pack of the engine version, outside any stack.
pub fn cx() -> LoadContext {
    LoadContext::detached()
}

/// A third-party pack with `min_engine_version` `version`.
pub fn cx_mev(major: u16, minor: u16, patch: u16) -> LoadContext {
    let mut cx = LoadContext::detached();
    let mev = MinEngineVersion::new(major, minor, patch);
    cx.molang_version = pack_molang_version(Some(&mev));
    cx.min_engine_version = Some(mev);
    cx
}

/// A base-game pack.
pub fn cx_base_game() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx
}

/// A pack of a world whose base-game version is `major.minor.0`.
pub fn cx_world(major: u16, minor: u16) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.world_base_game_version = BaseGameVersion::new(SemVersion::new(major, minor, 0));
    cx
}

/// Parses `bytes` as `recipes/<name>`.
pub fn parse_bytes(
    name: &str,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<Recipe>, Rejected> {
    let path = PackPath::new(&format!("recipes/{name}")).unwrap();
    Recipe::parse(&path, bytes, cx)
}

/// Parses fixture `name`.
pub fn load(name: &str, cx: &LoadContext) -> Result<Document<Recipe>, Rejected> {
    parse_bytes(name, &fixture(name), cx)
}

/// Fixture `name`, which must load.
pub fn loaded(name: &str, cx: &LoadContext) -> Document<Recipe> {
    load(name, cx)
        .unwrap_or_else(|rejected| panic!("{name} was rejected:\n{:#?}", rejected.diagnostics))
}

/// Fixture `name`, which must be rejected.
pub fn rejected(name: &str, cx: &LoadContext) -> Rejected {
    match load(name, cx) {
        Ok(document) => panic!(
            "{name} loaded:\n{:#?}\n{:#?}",
            document.value(),
            document.diagnostics()
        ),
        Err(rejected) => rejected,
    }
}

/// The shaped recipe of a document.
pub fn shaped(document: &Document<Recipe>) -> &Shaped {
    match document.value() {
        Recipe::Shaped(shaped) => shaped,
        other => panic!("not shaped: {other:?}"),
    }
}

/// The recipe family's definition a finding was raised on, if it was.
fn recipe_def(diagnostic: &Diagnostic) -> Option<&'static CodeMeta> {
    let code = &diagnostic.code;
    bedrock_addon_family_recipe::codes::ALL
        .iter()
        .find(|meta| meta.id() == code.id())
        .copied()
}

/// The game sentence of a finding of a recipe definition, `{}` where each argument goes.
pub fn template(diagnostic: &Diagnostic) -> Option<&'static str> {
    recipe_def(diagnostic)?.game_template()
}

/// The arguments of a finding of a recipe definition.
pub fn args(diagnostic: &Diagnostic) -> Vec<String> {
    match recipe_def(diagnostic) {
        Some(_) => diagnostic.code.args().to_vec(),
        None => Vec::new(),
    }
}

/// The arguments of a finding's vanilla text of `template`: the formatted text's, or those
/// that fill the template in the sentence of a finding raised on a worded definition.
pub fn args_in(diagnostic: &Diagnostic, template_: &str) -> Vec<String> {
    if template(diagnostic).is_some() {
        return args(diagnostic);
    }
    let Some(line) = diagnostic.vanilla() else {
        return Vec::new();
    };
    let pieces: Vec<&str> = template_.split("{}").collect();
    let mut rest = line.strip_prefix(pieces[0]).unwrap_or_default();
    let mut found = Vec::new();
    for (index, piece) in pieces.iter().enumerate().skip(1) {
        let end = if index + 1 == pieces.len() {
            rest.len() - piece.len()
        } else {
            rest.find(piece).unwrap_or(rest.len())
        };
        found.push(rest[..end].to_owned());
        rest = &rest[(end + piece.len()).min(rest.len())..];
    }
    found
}

/// The findings whose vanilla text has `template`. A finding raised on a worded definition
/// carries the sentence alone (the link pass's): its sentence fits the template.
pub fn with_text<'d>(diagnostics: &'d Diagnostics, template_: &str) -> Vec<&'d Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| match template(d) {
            Some(template) => template == template_,
            None => d.vanilla().is_some_and(|line| fits(template_, &line)),
        })
        .collect()
}

/// Whether `line` is `template` with each `{}` filled.
fn fits(template: &str, line: &str) -> bool {
    let mut pieces = template.split("{}");
    let first = pieces.next().unwrap_or_default();
    let Some(mut rest) = line.strip_prefix(first) else {
        return false;
    };
    let pieces: Vec<&str> = pieces.collect();
    for (index, piece) in pieces.iter().enumerate() {
        if index + 1 == pieces.len() {
            return rest.ends_with(piece);
        }
        let Some(at) = rest.find(piece) else {
            return false;
        };
        rest = &rest[at + piece.len()..];
    }
    rest.is_empty()
}

/// Exactly one finding with `template`, a kept error (the game's level-3
/// line after which the recipe loads); returns it.
pub fn one_kept<'d>(diagnostics: &'d Diagnostics, template_: &str) -> &'d Diagnostic {
    let finding = one(diagnostics, template_, Severity::Error);
    assert!(finding.kept, "{finding:#?}");
    finding
}

/// Exactly one finding with `template`, at `severity`; returns it.
pub fn one<'d>(
    diagnostics: &'d Diagnostics,
    template_: &str,
    severity: Severity,
) -> &'d Diagnostic {
    let found = with_text(diagnostics, template_);
    assert_eq!(
        found.len(),
        1,
        "expected one `{template_}`, found {}:\n{diagnostics:#?}",
        found.len()
    );
    assert_eq!(found[0].severity, severity, "{:#?}", found[0]);
    found[0]
}

/// No finding at all.
pub fn silent(diagnostics: &Diagnostics) {
    assert!(
        diagnostics.is_empty(),
        "unexpected findings:\n{diagnostics:#?}"
    );
}

/// Every file below fixture directory `dir`, as pack-relative paths.
pub fn files_below(dir: &str) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
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
    let root = fixtures().join(dir);
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// The unknown member `key` the document keeps in its table at the object `at`.
pub fn kept<'d>(
    document: &'d Document<Recipe>,
    at: &[&str],
    key: &str,
) -> Option<&'d bedrock_addon_core::json::Value> {
    document
        .unknown()
        .get(&bedrock_addon_core::diagnostic::JsonPath::of_keys(at), key)
}

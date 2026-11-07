//! Fixture loading, load contexts and finding matchers for the recipe tests.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon::diagnostic::{CodeMeta, Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::recipe::{Recipe, Shaped};
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::pack_molang_version;
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use bytes::Bytes;
use uuid::Uuid;

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recipe/recipe")
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
    bedrock_addon::families::recipe::codes::ALL
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

/// No finding at warning level or above.
pub fn clean(diagnostics: &Diagnostics) {
    let loud: Vec<_> = diagnostics.at_least(Severity::Warning).collect();
    assert!(loud.is_empty(), "unexpected findings:\n{loud:#?}");
}

/// No finding at all.
pub fn silent(diagnostics: &Diagnostics) {
    assert!(
        diagnostics.is_empty(),
        "unexpected findings:\n{diagnostics:#?}"
    );
}

/// A behaviour pack `n` with `min_engine_version` `mev`, opened from memory with `origin`, holding
/// `files` (pack-relative paths).
pub fn pack(n: u128, mev: [u16; 3], origin: PackOrigin, files: Vec<(String, Vec<u8>)>) -> Pack {
    let uuid = Uuid::from_u128(n);
    let module = Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let [a, b, c] = mev;
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [{a}, {b}, {c}]}}, "modules": [{{"type": "data", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    );
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), Bytes::from(bytes))),
    );
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
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
) -> Option<&'d bedrock_addon::json::Value> {
    document
        .unknown()
        .get(&bedrock_addon::diagnostic::JsonPath::of_keys(at), key)
}

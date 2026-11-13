//! The fixture packs of `tests/fixtures/script/script_bundle/packs/`: one folder per case (a
//! manifest and its files) and beside it `<case>.expected.toml`: the members, the main file,
//! what the entry reaches, every import site and every finding.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::{Path, PathBuf};

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, ReferenceHint};
use bedrock_addon_family_script::{ImportKind, Resolution, ScriptBundle};

/// The fixture folder of the family.
pub fn fixtures() -> PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR"))
        .join("script/script_bundle")
}

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
pub fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
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

/// The files of the fixture pack `case`.
pub fn case_files(case: &str) -> Vec<(String, Vec<u8>)> {
    files_below(&fixtures().join("packs").join(case))
}

/// A manifest of a behaviour pack named `name` with one script module of `entry` and a
/// dependency on `@minecraft/server`, its UUIDs made from `n`.
pub fn script_manifest(name: &str, n: u128, entry: &str) -> Vec<u8> {
    format!(
        r#"{{
    "format_version": 2,
    "header": {{ "name": "{name}", "description": "", "uuid": "{}", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] }},
    "modules": [{{ "type": "script", "language": "javascript", "entry": "{entry}",
        "uuid": "{}", "version": [1, 0, 0] }}],
    "dependencies": [{{ "module_name": "@minecraft/server", "version": "2.0.0" }}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n + 1_000_000),
    )
    .into_bytes()
}

/// One import site, as an expected file writes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Import {
    pub file: String,
    pub kind: String,
    pub raw: String,
    pub normalized: String,
    pub resolution: String,
    pub hint: String,
    pub near: String,
}

/// One finding, as an expected file writes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub text: String,
    pub file: String,
    pub near: String,
}

/// What an expected file says of a bundle, and what a bundle is rendered to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shape {
    pub members: Vec<String>,
    pub others: Vec<String>,
    pub main: Option<String>,
    pub reachable_static: Option<Vec<String>>,
    pub reachable_dynamic: Option<Vec<String>>,
    pub imports: Vec<Import>,
    pub diagnostics: Vec<Finding>,
}

fn kind_name(kind: ImportKind) -> &'static str {
    match kind {
        ImportKind::Static => "static",
        ImportKind::ReExport => "re_export",
        ImportKind::SideEffect => "side_effect",
        ImportKind::Dynamic => "dynamic",
        ImportKind::DynamicNonLiteral => "dynamic_non_literal",
        other => panic!("an import kind the fixtures do not name: {other:?}"),
    }
}

/// One finding as the expected files write it.
pub fn finding_of(finding: &Diagnostic) -> Finding {
    let near = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference {
            hint: Some(ReferenceHint::CaseMismatch(near)),
            ..
        }) => near.clone(),
        _ => String::new(),
    };
    Finding {
        code: finding.code.id().to_owned(),
        severity: finding.severity.to_string(),
        text: finding.code.to_string(),
        file: finding
            .location
            .path
            .as_ref()
            .map_or(String::new(), |path| path.as_str().to_owned()),
        near,
    }
}

/// `bundle` as an expected file writes one.
pub fn shape_of(bundle: &ScriptBundle) -> Shape {
    let names = |set: &std::collections::BTreeSet<bedrock_addon_family_script::ScriptPath>| {
        set.iter().map(|name| name.as_str().to_owned()).collect()
    };
    let imports = bundle
        .imports()
        .map(|(file, site)| {
            let (resolution, normalized, hint, near) = match &site.resolution {
                Resolution::File(name) => ("file", name.as_str().to_owned(), None, None),
                Resolution::Native { name } => ("native", name.clone(), None, None),
                Resolution::Restricted { name } => ("restricted", name.clone(), None, None),
                Resolution::Unrecognized { name } => ("unrecognized", name.clone(), None, None),
                Resolution::NotFound {
                    looked_up,
                    hint,
                    near,
                } => ("not_found", looked_up.clone(), *hint, near.clone()),
                Resolution::CrashesEngine => ("crash", String::new(), None, None),
                Resolution::NotChecked => ("not_checked", site.raw.clone(), None, None),
                other => panic!("a resolution the fixtures do not name: {other:?}"),
            };
            Import {
                file: file.as_str().to_owned(),
                kind: kind_name(site.kind).to_owned(),
                raw: site.raw.clone(),
                normalized,
                resolution: resolution.to_owned(),
                hint: hint.map_or(String::new(), |hint| format!("{hint:?}")),
                near: near.map_or(String::new(), |near| near.as_str().to_owned()),
            }
        })
        .collect();
    Shape {
        members: bundle.files.keys().map(|n| n.as_str().to_owned()).collect(),
        others: bundle
            .others
            .iter()
            .map(|p| p.as_str().to_owned())
            .collect(),
        main: bundle.entry.as_ref().map(|e| e.main.as_str().to_owned()),
        reachable_static: bundle.graph.as_ref().map(|g| names(&g.reachable_static)),
        reachable_dynamic: bundle.graph.as_ref().map(|g| names(&g.reachable_dynamic)),
        imports,
        diagnostics: bundle.findings().map(finding_of).collect(),
    }
}

/// The value of a basic string at the start of `text`, and what follows it.
fn string(text: &str) -> (String, &str) {
    let mut chars = text.char_indices();
    assert_eq!(chars.next().map(|(_, c)| c), Some('"'), "a string: {text}");
    let mut out = String::new();
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => return (out, &text[at + 1..]),
            '\\' => match chars.next().map(|(_, c)| c) {
                Some('n') => out.push('\n'),
                Some(other) => out.push(other),
                None => break,
            },
            other => out.push(other),
        }
    }
    panic!("an unterminated string: {text}");
}

/// The values of an array of basic strings.
fn strings(text: &str) -> Vec<String> {
    let mut rest = text
        .trim()
        .strip_prefix('[')
        .expect("an array")
        .trim_start();
    let mut out = Vec::new();
    while !rest.starts_with(']') {
        let (value, after) = string(rest);
        out.push(value);
        rest = after.trim_start().trim_start_matches(',').trim_start();
    }
    out
}

/// The tables of a file in the small TOML the fixtures are written in: `key = "string"` and
/// `key = ["strings"]` lines, `[[table]]` headers, `#` comments. Each table is its name and
/// its `(key, value)` lines; the lines before the first header are the table `""`.
pub fn tables(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut out = vec![(String::new(), Vec::new())];
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            out.push((name.to_owned(), Vec::new()));
            continue;
        }
        let (key, value) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("a line: {line}"));
        out.last_mut()
            .unwrap()
            .1
            .push((key.to_owned(), value.to_owned()));
    }
    out
}

/// The value of the string member `key` of a table; empty when the table has none.
pub fn member(table: &[(String, String)], key: &str) -> String {
    table
        .iter()
        .find(|(name, _)| name == key)
        .map_or(String::new(), |(_, value)| string(value).0)
}

/// The expected file of `case`. A finding marked `only = "script-syntax"` is one the bundle
/// has with that feature alone.
pub fn expected(case: &str) -> Shape {
    let path = fixtures()
        .join("packs")
        .join(format!("{case}.expected.toml"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut shape = Shape::default();
    for (name, table) in tables(&text) {
        let has = |key: &str| {
            table
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        match name.as_str() {
            "" => {
                shape.members = strings(has("members").expect("members"));
                shape.others = strings(has("others").expect("others"));
                shape.main = has("main").map(|value| string(value).0);
                shape.reachable_static = has("reachable_static").map(strings);
                shape.reachable_dynamic = has("reachable_dynamic").map(strings);
            }
            "imports" => shape.imports.push(Import {
                file: member(&table, "file"),
                kind: member(&table, "kind"),
                raw: member(&table, "raw"),
                normalized: member(&table, "normalized"),
                resolution: member(&table, "resolution"),
                hint: member(&table, "hint"),
                near: member(&table, "near"),
            }),
            "diagnostics" => {
                let only = member(&table, "only");
                assert!(
                    only.is_empty() || only == "script-syntax",
                    "{case}: only = {only}"
                );
                if only.is_empty() || ScriptBundle::SYNTAX_CHECKED {
                    shape.diagnostics.push(Finding {
                        code: member(&table, "code"),
                        severity: member(&table, "severity"),
                        text: member(&table, "text"),
                        file: member(&table, "file"),
                        near: member(&table, "near"),
                    });
                }
            }
            other => panic!("{case}: table {other}"),
        }
    }
    shape
}

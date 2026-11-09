//! Every `codes.rs` of the toolkit is a complete table that the registry checks: each
//! `pub static` definition of the file is in its `ALL`, once, and the file's owner has its
//! `ALL` in `tables()` of `diagnostic_ids.rs`, so the rule that no id appears twice covers it.
//!
//! The test reads the toolkit's own source files, which library code may never do; the scan
//! needs no feature, so a family whose feature is off is checked too.

use std::fs;
use std::path::{Path, PathBuf};

/// `crates/addon/`.
fn addon_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every `codes.rs` below a `src/` of `dir`, sorted.
fn codes_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            codes_files(&path, out);
        } else if path.file_name().is_some_and(|name| name == "codes.rs")
            && path.components().any(|part| part.as_os_str() == "src")
        {
            out.push(path);
        }
    }
}

/// The names of the `pub static <NAME>: CodeDef<..>` definitions of a file, in order; a
/// definition declared once per build (`#[cfg]` twins of one name) counts once.
fn definitions(source: &str) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for name in source
        .lines()
        .filter_map(|line| line.strip_prefix("pub static "))
        .filter_map(|rest| rest.split_once(": CodeDef<"))
        .map(|(name, _)| name)
    {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// The names `ALL` of a file lists (`NAME.meta(),`), in order; `None` without an `ALL`.
fn listed(source: &str) -> Option<Vec<&str>> {
    let start = source.find("pub static ALL: &[&CodeMeta] = &[")?;
    let body = &source[start..];
    let body = &body[body.find('[').unwrap() + 1..];
    let body = &body[body.find('[').unwrap() + 1..body.find("];").unwrap()];
    Some(
        body.split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty() && !entry.starts_with("//"))
            .map(|entry| entry.strip_suffix(".meta()").unwrap_or(entry))
            .collect(),
    )
}

/// The owner of a file's definitions: the part before `/` of its first id.
fn owner(source: &str) -> Option<&str> {
    let at = source.find("CodeDef::new(")?;
    let rest = &source[at..];
    let quote = rest.find('"')? + 1;
    let id = &rest[quote..quote + rest[quote..].find('"')?];
    id.split_once('/').map(|(owner, _)| owner)
}

/// What is wrong with one `codes.rs`: a definition `ALL` does not list, an entry of `ALL` that
/// is no definition of the file, an entry listed twice, or an owner `tables()` does not name.
fn violations(file: &str, source: &str, registry: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let defined = definitions(source);
    let Some(all) = listed(source) else {
        wrong.push(format!("{file}: no `ALL`"));
        return wrong;
    };
    if defined.len() != all.len() {
        wrong.push(format!(
            "{file}: {} definitions, {} in `ALL`",
            defined.len(),
            all.len()
        ));
    }
    for name in &defined {
        match all.iter().filter(|entry| *entry == name).count() {
            0 => wrong.push(format!("{file}: `{name}` is not in `ALL`")),
            1 => {}
            n => wrong.push(format!("{file}: `{name}` is in `ALL` {n} times")),
        }
    }
    for entry in &all {
        if !defined.contains(entry) {
            wrong.push(format!(
                "{file}: `ALL` lists `{entry}`, no definition of the file"
            ));
        }
    }
    match owner(source) {
        Some(owner) if registry.contains(&format!("\"{owner}\",")) => {}
        Some(owner) => wrong.push(format!(
            "{file}: the owner `{owner}` has no table in `diagnostic_ids.rs`"
        )),
        None => wrong.push(format!("{file}: no definition")),
    }
    wrong
}

#[test]
fn every_codes_file_lists_each_definition_once_and_is_in_the_registry() {
    let dir = addon_dir();
    let registry = fs::read_to_string(dir.join("addon/tests/diagnostic_ids.rs")).unwrap();
    let mut files = Vec::new();
    codes_files(&dir, &mut files);
    files.sort();
    assert!(files.len() > 50, "{files:?}");
    let mut wrong = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path).unwrap();
        let file = path.strip_prefix(&dir).unwrap().display().to_string();
        wrong.extend(violations(&file, &source, &registry));
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The scan's own rules on a small file.
#[test]
fn a_definition_left_out_of_all_or_of_the_registry_is_named() {
    let source = r#"
pub static A: CodeDef<0> = CodeDef::new(
    "demo/a",
    Kind::Lint,
    Class::Lint,
    "a",
);
pub static B: CodeDef<1> = CodeDef::new("demo/b", Kind::Lint, Class::Lint, "{}");

pub static ALL: &[&CodeMeta] = &[A.meta(), A.meta(), C.meta()];
"#;
    assert_eq!(
        violations("demo/codes.rs", source, r#"("other", x::ALL)"#),
        [
            "demo/codes.rs: 2 definitions, 3 in `ALL`",
            "demo/codes.rs: `A` is in `ALL` 2 times",
            "demo/codes.rs: `B` is not in `ALL`",
            "demo/codes.rs: `ALL` lists `C`, no definition of the file",
            "demo/codes.rs: the owner `demo` has no table in `diagnostic_ids.rs`",
        ]
    );
    let whole = "pub static A: CodeDef<0> = CodeDef::new(\"demo/a\", Kind::Lint, Class::Lint, \"a\");\npub static ALL: &[&CodeMeta] = &[\n    A.meta(),\n];\n";
    assert_eq!(
        violations("demo/codes.rs", whole, "(\"demo\", demo::ALL)"),
        [] as [String; 0]
    );
}

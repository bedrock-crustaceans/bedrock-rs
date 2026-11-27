//! the `crafting_catalog` family crate reads no other content module, by any spelling. The crate's
//! `module_independence` test scans for `crate::<module>`; this one closes the ways past it: a
//! chain of `super::` up to the crate root, the block family's loader, a `crate::{…}` group
//! import that names another content module, and an alias of the crate root.
//!
//! The test reads the crate's own source files, which library code may never do.

use std::path::{Path, PathBuf};

/// The families other than the catalogue itself: the sibling family crates under
/// `crates/addon/families/`, plus the `families` module itself, which a `crate::{…}` group of the catalogue has no reason to name.
fn other_families() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .filter(|name| name != "crafting_catalog")
        .collect();
    names.push("families".to_owned());
    names.sort();
    names
}

/// The names a `crate::{…}` group starting at `group` (the text after the brace) imports
/// from the crate root: the identifier at the start of each top-level member.
fn group_members(group: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut depth = 1_usize;
    let mut at_member_start = true;
    let mut rest = group;
    while let Some(c) = rest.chars().next() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        } else if c == ',' && depth == 1 {
            at_member_start = true;
        } else if at_member_start && depth == 1 && (c.is_ascii_alphabetic() || c == '_') {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            members.push(&rest[..end]);
            at_member_start = false;
            rest = &rest[end..];
            continue;
        }
        rest = &rest[c.len_utf8()..];
    }
    members
}

/// What `source` contains that reaches, or could reach, another content module.
fn forbidden_in(source: &str, others: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for text in ["super::super::super", "block::LOADER", "crate as"] {
        if source.contains(text) {
            found.push(text.to_owned());
        }
    }
    for (index, _) in source.match_indices("crate::{") {
        for member in group_members(&source[index + "crate::{".len()..]) {
            if others.iter().any(|other| other == member) {
                found.push(format!("crate::{{{member}}}"));
            }
        }
    }
    found
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn the_catalogue_reaches_no_other_content_module() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&dir, &mut files);
    assert!(
        files.len() >= 8,
        "crafting_catalog/src/ has its source files ({} found)",
        files.len()
    );
    let others = other_families();
    assert!(
        others.len() > 50,
        "crates/addon/families/ has the family crates ({} found)",
        others.len()
    );
    let mut violations = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        for text in forbidden_in(&source, &others) {
            violations.push(format!("{} contains `{text}`", path.display()));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn the_guard_sees_every_spelling() {
    let others = other_families();
    let forbidden_in = |source: &str| forbidden_in(source, &others);
    let relative = "use super::super::super::super::block::{Block, LOADER};";
    assert_eq!(forbidden_in(relative), ["super::super::super"]);
    assert_eq!(
        forbidden_in("crate::families::block::LOADER"),
        ["block::LOADER"]
    );
    assert_eq!(
        forbidden_in("use crate::{document::Merged, block::Block, id::{ItemId, kind}};"),
        ["crate::{block}"]
    );
    assert_eq!(
        forbidden_in("use crate::{json::Value, families::item::Item};"),
        ["crate::{families}"]
    );
    assert_eq!(
        forbidden_in(
            "use crate::{\n    json::Value,\n    entity,\n    recipe::{self, Recipes},\n};"
        ),
        ["crate::{entity}", "crate::{recipe}"]
    );
    assert_eq!(forbidden_in("use crate as root;"), ["crate as"]);
    assert_eq!(forbidden_in("extern crate self as root;"), [] as [&str; 0]);
    // What the catalogue does write: its own module, core vocabulary, a nested name that
    // only looks like a module.
    for allowed in [
        "use super::super::load::ParsedItem;",
        "use crate::families::item::load::ParsedItem;",
        "use crate::{document::{Document, text}, id::{ItemId, kind::Block}};",
        "use crate::id::{ItemId, Kind, kind};",
    ] {
        assert_eq!(forbidden_in(allowed), [] as [&str; 0], "{allowed}");
    }
}

#[test]
fn group_members_are_the_top_level_names() {
    assert_eq!(
        group_members("a::B, c::{d, e::{f}}, g as h, self}; use crate::{x};"),
        ["a", "c", "g", "self"]
    );
    assert_eq!(group_members("}"), [] as [&str; 0]);
}

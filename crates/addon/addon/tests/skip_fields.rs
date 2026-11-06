//! The skip-field convention: facet binds a JSON member to a
//! `#[facet(skip)]` field of its name, so every skipped field of the crate is declared
//! `#[facet(skip, rename = "\u{0}<name>")]` (core's `Extra`, "Skipped fields"). One test holds
//! the source to it; the others load documents whose members are named like a skipped field.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

/// The source roots the convention holds for, as paths below the workspace's `crates/`.
const ROOTS: &[&str] = &["addon/src", "core/src", "manifest/src", "storage/src"];

/// Skipped fields exempt from the convention, as `(file below its root, field line trimmed,
/// reason)`. The scan reads whole files, so `#[cfg(test)]` code is exempted here by path.
const EXEMPT: &[(&str, &str, &str)] = &[(
    "json/walk/mod.rs",
    "extra: Extra,",
    "the test-only structs of `mod tests` keep a bare skip on purpose: they pin the walk over a \
     skipped field and are no document type",
)];

/// Every `.rs` file below `dir`.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Whether `attribute` (a trimmed line) is a `#[facet]` that skips the field: `skip` as one
/// of its comma-separated arguments (not `skip_serializing_if`).
fn skips(attribute: &str) -> bool {
    let Some(inner) = attribute
        .strip_prefix("#[facet(")
        .and_then(|rest| rest.split_once(")]").map(|(inner, _)| inner))
    else {
        return false;
    };
    inner.split(',').any(|argument| argument.trim() == "skip")
}

/// The field name of a declaration line (`pub extra: Extra,` → `extra`; a `pub(crate)` or
/// macro `$crate::` type included).
fn field_name(line: &str) -> Option<&str> {
    let (head, _) = line.split_once(':')?;
    let name = head.rsplit(' ').next()?;
    Some(name)
}

/// The trimmed lines of `text`, a `#[facet(` attribute that spans several lines joined (by a
/// space) from its first line to the one holding its `)]`, as `(line number, text)`.
fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut lines = text.lines().map(str::trim).enumerate();
    while let Some((index, line)) = lines.next() {
        let mut joined = line.to_owned();
        if line.starts_with("#[facet(") && !line.contains(")]") {
            for (_, next) in lines.by_ref() {
                joined.push(' ');
                joined.push_str(next);
                if next.contains(")]") {
                    break;
                }
            }
        }
        out.push((index + 1, joined));
    }
    out
}

/// The skipped fields of `text` (the file `relative` below its root) as `(skipped fields seen,
/// those without the rename)`.
fn scan(relative: &str, text: &str) -> (usize, Vec<String>) {
    let lines = logical_lines(text);
    let mut seen = 0;
    let mut offenders = Vec::new();
    for (index, (number, line)) in lines.iter().enumerate() {
        if !skips(line) {
            continue;
        }
        seen += 1;
        // The field: the next line that is no attribute, doc or comment.
        let field = lines[index + 1..]
            .iter()
            .map(|(_, l)| l.as_str())
            .find(|l| !l.starts_with("#[") && !l.starts_with("//"))
            .unwrap_or_default();
        if EXEMPT.iter().any(|(f, l, _)| *f == relative && *l == field) {
            continue;
        }
        let expected = field_name(field).map(|name| format!(r#"rename = "\u{{0}}{name}""#));
        if !expected.is_some_and(|rename| line.contains(&rename)) {
            offenders.push(format!("{relative}:{number}: {line} {field}"));
        }
    }
    (seen, offenders)
}

#[test]
fn every_skipped_field_is_renamed_behind_a_nul() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut seen = 0;
    let mut offenders = Vec::new();
    for root in ROOTS {
        let src = crates.join(root);
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        files.sort();
        for file in &files {
            let text = std::fs::read_to_string(file).unwrap();
            let relative = file.strip_prefix(&src).unwrap().to_string_lossy();
            let (found, bad) = scan(&relative, &text);
            seen += found;
            offenders.extend(bad.into_iter().map(|o| format!("{root}/{o}")));
        }
    }
    assert!(seen > 0, "no skipped field found: the scan is broken");
    assert!(
        offenders.is_empty(),
        "skipped fields without `rename = \"\\u{{0}}<name>\"`:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn an_attribute_over_several_lines_without_the_rename_is_reported() {
    let bare = "struct S {\n    #[facet(\n        skip,\n        default\n    )]\n    pub extra: Extra,\n}\n";
    let (seen, offenders) = scan("demo.rs", bare);
    assert_eq!(seen, 1, "the multi-line skip was not recognised");
    assert_eq!(offenders.len(), 1, "{offenders:?}");
    let renamed = "struct S {\n    #[facet(\n        skip,\n        rename = \"\\u{0}extra\"\n    )]\n    pub extra: Extra,\n}\n";
    assert_eq!(scan("demo.rs", renamed), (1, Vec::new()));
}

#[test]
fn the_scan_tells_skip_from_its_neighbours() {
    assert!(skips("#[facet(skip)]"));
    assert!(skips(r#"#[facet(skip, rename = "\u{0}extra")]"#));
    assert!(skips(r#"#[facet(rename = "\u{0}extra", skip)]"#));
    assert!(!skips("#[facet(skip_serializing_if = Option::is_none)]"));
    assert!(!skips("#[facet(default)]"));
    assert_eq!(field_name("pub extra: Extra,"), Some("extra"));
    assert_eq!(field_name("pub(crate) extra: Extra,"), Some("extra"));
    assert_eq!(
        field_name("pub extra: $crate::document::Extra,"),
        Some("extra")
    );
    assert_eq!(field_name("min: Option<u32>,"), Some("min"));
}

#[cfg(feature = "item")]
mod item {
    use bedrock_addon::diagnostic::DiagnosticCode;
    use bedrock_addon::document::Content;
    use bedrock_addon::families::item::Item;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;

    /// A member named `extra` in the item object and in its description loads as an unknown
    /// member, warned and kept for write-back (at 1.21.90: from 1.26.30 the strict slice
    /// refuses an unknown member).
    #[test]
    fn a_member_named_extra_is_an_unrecognized_field() {
        const TEXT: &str = r#"{"format_version": "1.21.90", "minecraft:item": {"description": {"identifier": "demo:x", "extra": {"a": 1}}, "components": {"minecraft:max_stack_size": 16}, "extra": 1}}"#;
        let document = Item::parse(
            &PackPath::from_static("items/x.json"),
            TEXT.as_bytes(),
            &LoadContext::detached(),
        )
        .unwrap_or_else(|rejected| panic!("rejected: {:#?}", rejected.diagnostics));
        let mut unrecognized: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. }))
            .map(|f| f.location.json_path.to_string())
            .collect();
        unrecognized.sort();
        assert_eq!(
            unrecognized,
            ["minecraft:item/description/extra", "minecraft:item/extra"],
            "{:#?}",
            document.diagnostics()
        );
    }
}

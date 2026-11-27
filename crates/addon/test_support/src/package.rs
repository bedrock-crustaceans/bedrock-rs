//! The package-include rail: what a package's test files include from outside the package.
//! A package is built from its own files alone (`cargo package`, a registry download), so a
//! test that includes a neighbour crate's file builds in the workspace and nowhere else. Each
//! package keeps its own list of the includes it knows of and its own check, in
//! `tests/package_includes.rs`; this module is the text function and the walk they share.

/// The files of the package a test file includes from outside it: every `#[path = "…"]`
/// attribute and every `include!`, `include_str!` or `include_bytes!` of `source` whose path
/// leaves the package, each as `<file>: <path>`. `file` is the test file's path below the
/// package root.
///
/// What is seen: a `#[path]` attribute that opens a line, on one line or over several; an
/// include of a string literal (a path relative to the folder of the file that names it); and
/// an include of `concat!(env!("CARGO_MANIFEST_DIR"), "…", …)`, whose literals together are a
/// path from the package root. What is not: a path made any other way (a constant, another
/// macro, a file read when the test runs).
///
/// A package is built from its own files alone (`cargo package`, a registry download): a test
/// that includes a neighbour crate's file builds in the workspace and nowhere else.
pub fn includes_outside_the_package(file: &str, source: &str) -> Vec<String> {
    // The string literal `rest` starts with, after white space, and what follows it.
    fn literal(rest: &str) -> Option<(&str, &str)> {
        let rest = rest.trim_start().strip_prefix('"')?;
        let end = rest.find('"')?;
        Some((&rest[..end], &rest[end + 1..]))
    }

    // How many folders `file` is below the package root.
    let folders = file.split('/').count() - 1;
    let leaves = |path: &str, below: usize| {
        let mut depth = below;
        for step in path.split('/') {
            match step {
                "" | "." => {}
                ".." if depth == 0 => return true,
                ".." => depth -= 1,
                _ => depth += 1,
            }
        }
        false
    };
    // (where in the source, the path as written, whether it leaves the package)
    let mut found: Vec<(usize, String)> = Vec::new();
    for (at, attribute) in source.match_indices("#[path") {
        // The attribute opens its line: one inside a string literal of a test does not.
        let line = source[..at].rfind('\n').map_or(0, |end| end + 1);
        if !source[line..at].trim().is_empty() {
            continue;
        }
        let rest = source[at + attribute.len()..].trim_start();
        if let Some((path, _)) = rest.strip_prefix('=').and_then(literal)
            // An absolute path is no path of the package.
            && (path.starts_with('/') || leaves(path, folders))
        {
            found.push((at, path.to_owned()));
        }
    }
    for include in ["include!(", "include_str!(", "include_bytes!("] {
        for (at, _) in source.match_indices(include) {
            let rest = source[at + include.len()..].trim_start();
            if let Some((path, _)) = literal(rest) {
                if path.starts_with('/') || leaves(path, folders) {
                    found.push((at, path.to_owned()));
                }
            } else if let Some(mut rest) = rest.strip_prefix("concat!(") {
                // The literals up to the parenthesis that closes `concat!(`; the one inside
                // `env!(…)` names the variable and is no part of the path.
                let (mut path, mut from_root, mut depth) = (String::new(), false, 1usize);
                let mut in_env = false;
                while depth > 0 {
                    let Some(next) = rest.find(['"', '(', ')']) else {
                        break;
                    };
                    match &rest[next..=next] {
                        "(" => {
                            in_env = rest[..next].trim_end().ends_with("env!");
                            depth += 1;
                        }
                        ")" => {
                            in_env = false;
                            depth -= 1;
                        }
                        _ => {
                            let Some((piece, after)) = literal(&rest[next..]) else {
                                break;
                            };
                            if in_env {
                                from_root |= piece == "CARGO_MANIFEST_DIR";
                            } else {
                                path.push_str(piece);
                            }
                            rest = after;
                            continue;
                        }
                    }
                    rest = &rest[next + 1..];
                }
                let outside = if from_root {
                    leaves(&path, 0)
                } else {
                    path.starts_with('/') || leaves(&path, folders)
                };
                if outside {
                    found.push((at, path));
                }
            }
        }
    }
    found.sort();
    found
        .into_iter()
        .map(|(_, path)| format!("{file}: {path}"))
        .collect()
}

/// Every `.rs` file below `dir`, as its path below `root` and its text, sorted.
pub fn rust_files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(root, &path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let relative = path
                .strip_prefix(root)
                .expect("a path below the root")
                .to_string_lossy()
                .replace('\\', "/");
            // A file that is no UTF-8 is no Rust source the compiler takes.
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push((relative, text));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The check finds an include that leaves the package, of each kind and from each depth, and
    /// no include that stays inside. The sources are written with `ATTR` and `INC` for `#[path`
    /// and `include`, so that the walk over this file does not find them.
    #[test]
    fn an_include_that_leaves_the_package_is_found() {
        let rust = |text: &str| text.replace("ATTR", "#[path").replace("INC", "include");
        let source = rust(
            r##"
    ATTR = "common/responder.rs"]
    mod responder;
    ATTR = "../../addon_std/tests/common/collections.rs"]
    mod collections;
        ATTR="../src/lib.rs"] mod inside;
    ATTR
        = "../../xtask/src/two_lines.rs"]
    mod two_lines;
    const A: &str = INC_str!("fixtures/a.json");
    const B: &[u8] = INC_bytes!("../../addon_core/b.bin");
    const C: &str = INC_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"));
    const D: &str = INC_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../addon_core", "/d.rs"));
    const E: &str = INC_str!(
        concat!(
            "../../",
            "addon_core/e.rs"
        )
    );
    INC!("../tests/inside.rs");
    let text = r#"ATTR = "../../block/load.rs"] mod x;"#;
    "##,
        );
        assert_eq!(
            includes_outside_the_package("tests/determinism.rs", &source),
            [
                "tests/determinism.rs: ../../addon_std/tests/common/collections.rs",
                "tests/determinism.rs: ../../xtask/src/two_lines.rs",
                "tests/determinism.rs: ../../addon_core/b.bin",
                "tests/determinism.rs: /../addon_core/d.rs",
                "tests/determinism.rs: ../../addon_core/e.rs",
            ]
        );
        // One folder further down the relative paths end inside the package; the one from the
        // package root leaves from anywhere. One more step up leaves.
        assert_eq!(
            includes_outside_the_package("tests/link/rules.rs", &source),
            ["tests/link/rules.rs: /../addon_core/d.rs"]
        );
        assert_eq!(
            includes_outside_the_package("tests/link/rules.rs", &rust("ATTR = \"../../../x.rs\"]")),
            ["tests/link/rules.rs: ../../../x.rs"]
        );
        assert_eq!(
            includes_outside_the_package("tests/a.rs", &rust("ATTR = \"/etc/passwd\"] mod m;")),
            ["tests/a.rs: /etc/passwd"]
        );
        // The limit: a path that is no literal of the include is not seen.
        assert_eq!(
            includes_outside_the_package(
                "tests/a.rs",
                &rust(
                    "const P: &str = \"../../x\"; let text = std::fs::read(P); INC_str!(some_macro!());"
                )
            ),
            Vec::<String>::new()
        );
    }
}

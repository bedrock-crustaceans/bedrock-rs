//! What one load of a world came to, as text that two loads can be compared by: the report and
//! every collection of both stacks, collection by collection: two loads of one world give equal
//! `Debug` output of the `StackReport` and of every collection's `iter()`.
//!
//! Used by `crates/addon/addon/tests/determinism.rs` (the five completion orders) and by the driver
//! crate's `full_strategies.rs`, `full_strategies_client_install_a.rs` and
//! `full_strategies_client_install_b.rs` (`Strategy::Sequential` against `Strategy::Rayon`); it
//! names `bedrock_addon` alone.
//!
//! This file is in both packages, as `tests/common/collections.rs` of `bedrock_addon` and of
//! `bedrock_addon_std`, because a package builds from its own files alone and neither crate
//! can hand the other a test helper. The two are one text:
//! `the_copies_of_the_shared_test_files_are_the_same` fails when they differ, so a change is
//! made in one and copied to the other. [`SHARED`] lists the files kept so.
//!
//! The collections are enumerated from the stacks themselves: a `PackStack`'s `Debug` lists
//! every accessor of its side whose cache is filled, as `(Some(<Collection variant>), "<the
//! collection's Debug>")`, in accessor-list order. After `load_all` or a `StackLoader` over
//! `CollectionSet::ALL` that is every accessor compiled in, so nothing here is a list kept by
//! hand.
//!
//! Why the `Debug` of a collection is its iteration order, kind by kind:
//!
//! - `Registry<T>` and `Files<T>` print `entries` as `debug_map().entries(self.iter())`: the
//!   very iterator, in its order (`bedrock_addon_core` `document/by_id.rs`, `by_path.rs`).
//!   `rejected` and `diagnostics` are vectors in walk order; a registry's `beta` is the beta
//!   registry's `iter()`.
//! - `Merged<T, L>` prints `value` (the family's merged struct), `origins` (an `IndexMap`,
//!   insertion order, which is what `Merged::origin` walks) and `layers` (a vector, lowest
//!   pack first: what `layers()` iterates).
//! - The group types (`Jigsaw`, `Recipes`, `FeatureRules`, `Poi`, `Cameras`, `AimAssist`,
//!   `SoundFiles`, `DeferredSettings`, `DataDrivenUi`, `TextureAtlases`, `TextureLists`,
//!   `TextureSets`, `TextureFiles`, `Fonts`, `Materials`) derive
//!   `Debug` (`Recipes` writes the same by hand) over fields that are registries, files,
//!   merged values, vectors and `IndexMap`s: declaration order, each field as above.
//! - A keyed accessor (`translations`) prints its keys as a list of `(key, collection)` in
//!   walk order; a slice accessor (`scripts`) prints its vector.
//!
//! No collection sorts for `Debug` and none prints a hash map, so a difference in iteration
//! order is a difference in the text.
//!
//! What the text does not show. Some values inside the compared collections have a `Debug`
//! that prints less than they hold, so two loads that differ only there have the same text.
//! Five of single families:
//!
//! - a material's `NameSet` prints "N names", the count and not the names
//!   (`families/material/set.rs`);
//! - a geometry's `Inherited` prints its identifier, the number of bones and the chain's depth,
//!   not the bones (`geometry/mod.rs`);
//! - `JigsawMetadata` prints `body_len`, the length of the cache document and not its bytes
//!   (`families/jigsaw/metadata.rs`);
//! - a structure's `Layout` prints `kept`, whether the source is held, and nothing of it
//!   (`structure/mod.rs`);
//! - a UI definition's `Inherited` prints the namespace and name it inherits from, not what it
//!   resolved to (`ui/chain.rs`).
//!
//! And of every registry and path map everything but the effective documents: their `Debug`
//! prints `entries` (the effective document of each identifier or path), `rejected` and
//! `diagnostics`, and of a registry's beta registry the identifiers of its effective documents.
//! It prints no document that is not effective (a shadowed copy kept with
//! `LoadOptions::keep_shadowed`; a copy under a refused or ignored one; a stable definition a
//! beta one replaced, and the reverse), nothing of a beta document, and not the beta
//! registry's rejected files and findings (`bedrock_addon_core` `document/by_id.rs`,
//! `by_path.rs`). `Registry::all_layers` and `Files::all_layers` list every identifier and
//! path anything is kept of with all of its layers; `unprinted.rs` takes the whole table from
//! them.
//!
//! The first two are results of a combine step. Each is the type's own `Debug`, written so
//! for a reason of its own (a chain printed once, a cache not dumped), and stays so. The
//! comparison reads what they leave out through the public accessors instead:
//! `unprinted.rs` gives each such value as an [`Unprinted`], a snapshot carries them
//! ([`Snapshot::with_unprinted`]) and [`differences`] compares them value by value, by name.
//!
//! Outside the collections: a stack's packs are printed with the length of their file
//! listing, and the bytes a pack holds are in no `Debug`. `StackSet::link()` is the public
//! view of both (a rule reads a file no collection holds; another resolves a name by the
//! listing), so a snapshot carries what linking says.
//!
//! The tests of this file are about the file itself: that each of the two packages builds
//! from its own files, and that the copies are one text.

#![allow(dead_code, reason = "each test file uses part of it")]

use bedrock_addon::pack::{StackReport, StackSet};

/// One load, as text.
pub struct Snapshot {
    /// `Debug` of the `StackReport`: every finding in load order (`StackReport::iter`).
    pub report: String,
    /// `Display` of the `StackReport`: one line per finding, sorted by pack, path and offset.
    pub display: String,
    /// `{:#?}` of the behaviour stack: its packs, the hidden ones and its collections.
    pub behavior: String,
    /// `{:#?}` of the resource stack.
    pub resource: String,
    /// `Debug` of what `StackSet::link()` says of the loaded set, the lints included: every
    /// finding of the link pass in its order. The pass reads what no collection holds (a
    /// pack's resident bytes, the file listing of a behaviour stack's packs).
    pub link: String,
    /// What the `Debug` of the collections leaves out, value by value (`unprinted.rs`); empty
    /// until [`with_unprinted`](Self::with_unprinted) gives it.
    pub unprinted: Vec<Unprinted>,
}

/// A value of a loaded collection whose `Debug` prints less than the value holds: the part
/// the text leaves out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unprinted {
    /// The value and the part, as a difference names them (`the body of <path>`).
    pub what: String,
    /// The part, as bytes: a text, or the bytes themselves.
    pub content: Vec<u8>,
}

impl Unprinted {
    /// The part `content` of the value `what`.
    pub fn new(what: String, content: impl Into<Vec<u8>>) -> Unprinted {
        Unprinted {
            what,
            content: content.into(),
        }
    }
}

/// One collection of a stack: an accessor whose cache is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    /// `"behavior"` or `"resource"`.
    pub side: &'static str,
    /// The accessor's `Collection` variant, as its `Debug` prints it.
    pub name: &'a str,
    /// The collection's `Debug`, as the stack's `Debug` holds it (a string literal's inside,
    /// still escaped).
    pub text: &'a str,
}

impl Snapshot {
    /// The texts of `set` and of `report`, the report of that set (`set.load_all()`: the link
    /// pass is run after it, over everything loaded).
    pub fn of(set: &StackSet, report: &StackReport<'_>) -> Snapshot {
        Snapshot {
            report: format!("{report:?}"),
            display: report.to_string(),
            behavior: format!("{:#?}", set.behavior),
            resource: format!("{:#?}", set.resource),
            link: format!("{:?}", set.link()),
            unprinted: Vec::new(),
        }
    }

    /// The snapshot with `unprinted`, what `unprinted::of` reads from the same set.
    pub fn with_unprinted(mut self, unprinted: Vec<Unprinted>) -> Snapshot {
        self.unprinted = unprinted;
        self
    }

    /// Every loaded collection: the behaviour stack's in accessor-list order, then the
    /// resource stack's.
    pub fn collections(&self) -> Vec<Entry<'_>> {
        let mut entries = collections_of("behavior", &self.behavior);
        entries.extend(collections_of("resource", &self.resource));
        entries
    }
}

/// The collections a stack's pretty `Debug` lists.
///
/// The text is `PackStack { side, packs, hidden, collections: [(Some(Variant), "…"), …], .. }`
/// printed with `{:#?}`: the field's line is indented by four spaces, a variant's by sixteen
/// and a collection's text, a string literal on one line (a line break inside it is written
/// `\n`), by twelve. Any other line inside the field is a change of that format and panics.
fn collections_of<'a>(side: &'static str, stack: &'a str) -> Vec<Entry<'a>> {
    let mut lines = stack.lines();
    let mut entries = Vec::new();
    loop {
        match lines.next() {
            Some("    collections: [],") => return entries,
            Some("    collections: [") => break,
            Some(_) => {}
            None => panic!("the {side} stack's Debug has no `collections` field"),
        }
    }
    let mut name = None;
    for line in lines {
        match line {
            "    ]," => return entries,
            "        (" | "            Some(" | "            )," | "        )," => {}
            _ => {
                if let Some(text) = line
                    .strip_prefix("            \"")
                    .and_then(|rest| rest.strip_suffix("\","))
                {
                    let name = name.take().expect("a collection's variant before its text");
                    entries.push(Entry { side, name, text });
                } else if let Some(variant) = line
                    .strip_prefix("                ")
                    .and_then(|rest| rest.strip_suffix(','))
                    .filter(|variant| variant.chars().all(char::is_alphanumeric))
                {
                    name = Some(variant);
                } else {
                    panic!("the {side} stack's `collections` field has the line {line:?}");
                }
            }
        }
    }
    panic!("the {side} stack's `collections` field does not end");
}

/// Where two texts first differ, with what each holds there; `None` when they are equal.
fn difference(found: &str, reference: &str) -> Option<String> {
    difference_of_bytes(found.as_bytes(), reference.as_bytes())
}

/// [`difference`] for bytes: the excerpts are shown as text, a byte that is none escaped.
fn difference_of_bytes(found: &[u8], reference: &[u8]) -> Option<String> {
    if found == reference {
        return None;
    }
    let at = found
        .iter()
        .zip(reference)
        .position(|(a, b)| a != b)
        .unwrap_or(found.len().min(reference.len()));
    let excerpt = |bytes: &[u8]| {
        let from = at.saturating_sub(120);
        let to = (at + 120).min(bytes.len());
        match std::str::from_utf8(bytes) {
            // A text is cut at characters.
            Ok(text) => {
                let (mut from, mut to) = (from, to);
                while !text.is_char_boundary(from) {
                    from -= 1;
                }
                while !text.is_char_boundary(to) {
                    to += 1;
                }
                text[from..to].to_owned()
            }
            Err(_) => bytes[from..to].escape_ascii().to_string(),
        }
    };
    Some(format!(
        "first difference at byte {at} (lengths {} and {}):\n  found:     …{}…\n  reference: …{}…",
        found.len(),
        reference.len(),
        excerpt(found),
        excerpt(reference)
    ))
}

/// Every part of `found` that differs from `reference`, named: the report in load order, its
/// `Display`, each collection of each stack, what else a stack's `Debug` holds (its packs
/// and the hidden ones), what linking says, and each value whose `Debug` leaves a part out, by
/// that part. Empty when the two loads are the same.
pub fn differences(found: &Snapshot, reference: &Snapshot) -> Vec<String> {
    let mut differences = Vec::new();
    let mut compare = |what: String, found: &str, reference: &str| {
        if let Some(difference) = difference(found, reference) {
            differences.push(format!("{what}: {difference}"));
        }
    };
    compare(
        String::from("the report (findings in load order)"),
        &found.report,
        &reference.report,
    );
    compare(
        String::from("the report's Display"),
        &found.display,
        &reference.display,
    );
    let (ours, theirs) = (found.collections(), reference.collections());
    let names = |entries: &[Entry<'_>]| -> Vec<String> {
        entries
            .iter()
            .map(|entry| format!("{} {}", entry.side, entry.name))
            .collect()
    };
    if names(&ours) == names(&theirs) {
        for (ours, theirs) in ours.iter().zip(&theirs) {
            compare(
                format!("collection {} of the {} stack", ours.name, ours.side),
                ours.text,
                theirs.text,
            );
        }
    } else {
        compare(
            String::from("the loaded collections"),
            &names(&ours).join(", "),
            &names(&theirs).join(", "),
        );
    }
    // The packs, the hidden packs and anything a collection's text did not show.
    compare(
        String::from("the behaviour stack"),
        &found.behavior,
        &reference.behavior,
    );
    compare(
        String::from("the resource stack"),
        &found.resource,
        &reference.resource,
    );
    compare(
        String::from("what linking says"),
        &found.link,
        &reference.link,
    );
    // What the texts above leave out, value by value.
    let whats = |snapshot: &Snapshot| -> Vec<String> {
        snapshot
            .unprinted
            .iter()
            .map(|value| value.what.clone())
            .collect()
    };
    if whats(found) == whats(reference) {
        for (ours, theirs) in found.unprinted.iter().zip(&reference.unprinted) {
            if let Some(difference) = difference_of_bytes(&ours.content, &theirs.content) {
                differences.push(format!("{}: {difference}", ours.what));
            }
        }
    } else {
        compare(
            String::from("the values whose Debug leaves a part out"),
            &whats(found).join("\n"),
            &whats(reference).join("\n"),
        );
    }
    differences
}

/// Requires `found` to be the load `reference` is: the same report, in load order and as
/// displayed, the same `Debug` of every collection of both stacks and of the stacks, and the
/// same content where a `Debug` leaves a part out.
#[track_caller]
pub fn assert_same(what: &str, found: &Snapshot, reference: &Snapshot) {
    let differences = differences(found, reference);
    assert!(
        differences.is_empty(),
        "{what}: the loads differ in {} places:\n{}",
        differences.len(),
        differences.join("\n")
    );
}

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

/// The test files of this package that include a file outside it, and are known to, as
/// `(package, "<file>: <path>")`. Every family is the crate `bedrock_addon_family_<name>`
/// under `crates/addon/families/`, and its test helpers (`common.rs`, `generate.rs`,
/// `compare.rs`) and its component list live there with it. The tests of `bedrock_addon` that
/// build a `PackStack` of the family include those helpers by `#[path]` rather than keep a
/// second copy, so `bedrock_addon`'s tests build in the workspace, where every family is a
/// sibling. `bedrock_addon_std` includes nothing outside itself. A new include joins the list
/// only when it reaches a family's `tests/<name>/` helper or its component list.
const KNOWN_OUTSIDE: &[(&str, &str)] = &[
    (
        "bedrock_addon",
        "tests/atomic_client_entity.rs: ../../families/client_entity/tests/client_entity/generate.rs",
    ),
    (
        "bedrock_addon",
        "tests/attachable.rs: ../../families/attachable/tests/attachable/generate.rs",
    ),
    (
        "bedrock_addon",
        "tests/block_culling.rs: ../../families/block_culling/tests/block_culling/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/client_entity.rs: ../../families/client_entity/tests/client_entity/generate.rs",
    ),
    (
        "bedrock_addon",
        "tests/crafting_catalog.rs: ../../families/crafting_catalog/tests/crafting_catalog/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/deferred_settings.rs: ../../families/deferred_settings/tests/deferred_settings/compare.rs",
    ),
    (
        "bedrock_addon",
        "tests/dimension.rs: ../../families/dimension/tests/dimension/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/entity/descriptors.rs: ../../../families/entity/src/component/mod.rs",
    ),
    (
        "bedrock_addon",
        "tests/feature.rs: ../../families/feature/tests/feature/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/feature_rules.rs: ../../families/feature_rules/tests/feature_rules/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/item.rs: ../../families/item/tests/item/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/loot.rs: ../../families/loot/tests/loot/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/server_sound_definitions.rs: ../../families/server_sound_definitions/tests/server_sound_definitions/common.rs",
    ),
    (
        "bedrock_addon",
        "tests/structure.rs: ../../families/structure/tests/structure/common.rs",
    ),
];

/// Every `.rs` file below `dir`, as its path below `root` and its text, sorted.
fn rust_files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, String)>) {
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

/// The package of the test that includes this file builds from its own files: no test file
/// includes one of another crate. This file is in both `bedrock_addon` and `bedrock_addon_std`
/// (`tests/common/`), so the check runs for each.
#[test]
fn no_test_file_of_this_package_includes_a_file_outside_it() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(root, &root.join("tests"), &mut files);
    let mut found: Vec<String> = files
        .iter()
        .flat_map(|(file, text)| includes_outside_the_package(file, text))
        .collect();
    found.sort();
    let mut known: Vec<&str> = KNOWN_OUTSIDE
        .iter()
        .filter(|(package, _)| *package == env!("CARGO_PKG_NAME"))
        .map(|(_, include)| *include)
        .collect();
    known.sort_unstable();
    assert_eq!(
        found,
        known,
        "includes that leave {}",
        env!("CARGO_PKG_NAME")
    );
    // The file this test is in is one of the package's.
    assert!(
        files
            .iter()
            .any(|(file, _)| file == "tests/common/collections.rs"),
        "the walk finds this file"
    );
}

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

/// The files of `tests/common/` that `bedrock_addon` and `bedrock_addon_std` both hold.
const SHARED: &[&str] = &["collections.rs", "unprinted.rs"];

/// What is wrong with the copy `other` of the shared file `name`, whose text here is `ours`:
/// it is absent, or where its text first differs. `None` when the two are one text.
fn copy_difference(name: &str, ours: &str, other: Option<&str>) -> Option<String> {
    match other {
        None => Some(format!("{name}: the other crate has no copy")),
        Some(theirs) => difference(ours, theirs)
            .map(|difference| format!("{name}: the two copies differ: {difference}")),
    }
}

/// The copy of each shared file in the other crate is this crate's, byte for byte. A package
/// built outside the workspace has no neighbour's tests to compare with: the test says
/// `skipped:`. In the workspace the other crate's `tests/common/` is there, and a copy missing
/// from it fails.
#[test]
fn the_copies_of_the_shared_test_files_are_the_same() {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let other = match env!("CARGO_PKG_NAME") {
        "bedrock_addon" => "std",
        "bedrock_addon_std" => "addon",
        package => panic!("{package} is neither of the two crates that share these files"),
    };
    let Some(other) = here.parent().map(|crates| crates.join(other)) else {
        panic!("{} has no parent folder", here.display());
    };
    // A package holds its own tests alone: where the other crate is absent, or is there as a
    // dependency's sources without its tests, there is nothing to compare with.
    if !other.join("tests/common").is_dir() {
        eprintln!(
            "skipped: {} has no tests/common (a package built alone), so the shared test files have no copy to compare with",
            other.display()
        );
        return;
    }
    let differences: Vec<String> = SHARED
        .iter()
        .filter_map(|name| {
            let ours = std::fs::read_to_string(here.join("tests/common").join(name))
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            let theirs = std::fs::read_to_string(other.join("tests/common").join(name)).ok();
            copy_difference(name, &ours, theirs.as_deref())
        })
        .collect();
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

/// Two copies that differ, and a copy that is missing, are told with the file's name.
#[test]
fn copies_that_differ_are_told_with_the_file_s_name() {
    assert_eq!(
        copy_difference("collections.rs", "fn a() {}", Some("fn a() {}")),
        None
    );
    let differs = copy_difference("collections.rs", "fn a() {}", Some("fn b() {}"));
    assert!(
        differs.as_deref().is_some_and(|line| line
            .starts_with("collections.rs: the two copies differ: first difference at byte 3 ")),
        "{differs:?}"
    );
    assert_eq!(
        copy_difference("collections.rs", "fn a() {}", None).as_deref(),
        Some("collections.rs: the other crate has no copy")
    );
}

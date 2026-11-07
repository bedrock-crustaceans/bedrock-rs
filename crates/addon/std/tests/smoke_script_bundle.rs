//! Script-bundle smoke over the game's own files: every behaviour and resource pack of the
//! sample packs, the server install, the two client installs and the split client pack tree
//! loads through `Driver` into `scripts()` with zero error-level findings, every `.js` file
//! below a pack's `scripts/` is a member of its bundle, and the counts (bundles, plugins,
//! script files, the files and bytes the load read, import sites by what they resolve to) are
//! the ones pinned below. With the `script-syntax` feature of `bedrock_addon` on, the files the
//! parser reads have no syntax error either.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_script_bundle`, and once more with `--features bedrock_addon/script-syntax`; without
//! the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::script::{Resolution, ScriptBundle, ScriptSyntaxKind};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, PackType, StackSet};
use bedrock_addon_std::{Driver, Options};

use common::game_files;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.join("manifest.json").is_file());
    roots.sort();
    roots
}

/// The files below `dir` whose name ends in `.js`, recursively.
fn js_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .map(|path| {
            if path.is_dir() {
                js_files(&path)
            } else {
                usize::from(
                    path.file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".js")),
                )
            }
        })
        .sum()
}

/// The bundles of the pack in `dir`, loaded alone on the stack of its type as a pack the game
/// ships.
fn bundles_of(dir: &Path) -> Vec<ScriptBundle> {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    let options = LoadOptions::default();
    let resource = pack.pack_type() == PackType::Resource;
    let set = if resource {
        StackSet {
            behavior: PackStack::of([], options.clone()).unwrap(),
            resource: PackStack::of([pack], options).unwrap(),
        }
    } else {
        StackSet {
            behavior: PackStack::of([pack], options.clone()).unwrap(),
            resource: PackStack::of([], options).unwrap(),
        }
    };
    let set = driver.load_stacks(set, StackOptions::default()).unwrap();
    // Nothing of the load itself may fail: a script file that could not be read is no member.
    let scripts = if resource {
        set.resource.scripts()
    } else {
        set.behavior.scripts()
    };
    scripts.to_vec()
}

/// What the packs of one source hold.
#[derive(Debug, Default, PartialEq, Eq)]
struct Counts {
    /// Packs read.
    packs: usize,
    /// Packs with a bundle: a script module or a file below `scripts/`.
    bundles: usize,
    /// Bundles whose script module vanilla starts and whose main file is there.
    plugins: usize,
    /// Script files.
    files: usize,
    /// Script files the load read: those of the packs whose script module starts a plugin.
    read: usize,
    /// The bytes of the script files the load read.
    read_bytes: u64,
    /// Other files below `scripts/`.
    others: usize,
    /// Import sites.
    imports: usize,
    /// Sites that resolve to a file of the bundle.
    to_files: usize,
    /// Sites that resolve to a native module.
    to_natives: usize,
    /// Sites not looked at: a non-literal `import()`, or a file the entry does not reach.
    not_checked: usize,
    /// The one error vanilla's own packs have ([`is_known_missing_main`]).
    known_missing_main: usize,
    /// Script files the parser of `script-syntax` read (none without the feature): the files
    /// an entry reaches, which in vanilla's packs is each plugin's main file alone.
    parsed: usize,
}

/// The `experimental_spawn_rules` behaviour pack of client install A and of the split client
/// pack tree names `scripts/ExperimentalSpawnRules.js` as its entry and ships `scripts/scripts.js`
/// alone: vanilla's `does not contain main file [ExperimentalSpawnRules.js].`, raised for it
/// as for any pack, so this one pack cannot run.
fn is_known_missing_main(root: &Path, finding: &Diagnostic) -> bool {
    root.ends_with("behavior_packs/experimental_spawn_rules")
        && finding.code.id() == "script/plugin_missing_main_file"
        && finding
            .code
            .to_string()
            .ends_with("does not contain main file [ExperimentalSpawnRules.js].")
}

/// Loads every pack below `dirs` and checks its bundle: no error, every `.js` file on disk a
/// member, no lexical error anywhere, and with `script-syntax` no parser error in a file the
/// parser read. Returns the counts.
fn check(source: &str, dirs: &[PathBuf]) -> Counts {
    let mut counts = Counts::default();
    let mut errors = Vec::new();
    let mut by_code: BTreeMap<(&str, String), usize> = BTreeMap::new();
    let (mut parsed, mut not_parsed) = (0, 0);
    for root in dirs.iter().flat_map(|dir| pack_roots(dir)) {
        counts.packs += 1;
        let on_disk = js_files(&root.join("scripts"));
        let bundles = bundles_of(&root);
        assert!(
            bundles.len() <= 1,
            "{}: one pack, one bundle",
            root.display()
        );
        let members: usize = bundles.iter().map(|bundle| bundle.files.len()).sum();
        assert_eq!(members, on_disk, "{}: script files", root.display());
        for bundle in &bundles {
            counts.bundles += 1;
            counts.plugins += usize::from(bundle.graph.is_some());
            counts.files += bundle.files.len();
            counts.others += bundle.others.len();
            for (name, file) in &bundle.files {
                // Only a pack whose script module starts a plugin has its scripts read: the
                // others are listed by name and no byte of them is asked for.
                let Some(content) = &file.content else {
                    assert!(bundle.plugin.is_none(), "{}: {name}", root.display());
                    continue;
                };
                counts.read += 1;
                counts.read_bytes += content.size;
                // The loose file's content, never a zero-size archive entry's.
                assert!(content.size > 0, "{}: {name} is empty", root.display());
                assert_eq!(content.utf8, Ok(()), "{}: {name}", root.display());
                match &content.syntax {
                    None => {}
                    Some(syntax) => errors.push(format!(
                        "{}: {name}:{}:{}: {:?}",
                        root.display(),
                        syntax.at.line,
                        syntax.at.column,
                        syntax.kind
                    )),
                }
                assert!(
                    !matches!(
                        content.syntax.as_ref().map(|syntax| &syntax.kind),
                        Some(ScriptSyntaxKind::Parser { .. })
                    ) || content.parsed
                );
                if content.parsed {
                    parsed += 1;
                } else {
                    not_parsed += 1;
                }
            }
            for (_, site) in bundle.imports() {
                counts.imports += 1;
                match &site.resolution {
                    Resolution::File(_) => counts.to_files += 1,
                    Resolution::Native { .. } => counts.to_natives += 1,
                    Resolution::NotChecked => counts.not_checked += 1,
                    other => errors.push(format!("{}: {} -> {other:?}", root.display(), site.raw)),
                }
            }
            for finding in bundle.findings() {
                *by_code
                    .entry((finding.code.id(), finding.severity.to_string()))
                    .or_default() += 1;
                if finding.severity >= Severity::Error {
                    if is_known_missing_main(&root, finding) {
                        counts.known_missing_main += 1;
                    } else {
                        errors.push(format!("{}: {finding}", root.display()));
                    }
                }
            }
        }
    }
    eprintln!(
        "{source}: {counts:?}; parser read {parsed} files and was kept from {not_parsed}; findings {by_code:?}"
    );
    for error in errors.iter().take(50) {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // The parser is kept from no file a vanilla entry reaches.
    assert!(
        by_code
            .keys()
            .all(|(code, _)| *code != "core/script_syntax_not_checked"),
        "{source}: {by_code:?}"
    );
    counts.parsed = parsed;
    counts
}

/// `behavior_packs/` and `resource_packs/` below `root`.
fn both(root: &Path) -> Vec<PathBuf> {
    vec![root.join("behavior_packs"), root.join("resource_packs")]
}

#[test]
fn sample_packs_scripts_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The two packs are folders of the checkout itself.
    let counts = check("sample packs", &[samples]);
    // Neither pack of the samples has a script.
    assert_eq!(
        counts,
        Counts {
            packs: 2,
            ..Counts::default()
        }
    );
}

#[test]
fn server_install_scripts_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let counts = check("server install", &both(&server));
    // The three library packs (data modules: one note each, their fourteen wrapper files
    // listed and not read) and `editor`, whose `Main.js` is the one file read and imports
    // four native modules. Only a pack whose script module starts a plugin has its scripts
    // read, so the other fifteen files (3.59 MB) are listed and not read.
    assert_eq!(
        counts,
        Counts {
            packs: 71,
            bundles: 4,
            plugins: 1,
            files: 15,
            read: 1,
            read_bytes: 2_551_502,
            others: 0,
            imports: 4,
            to_files: 0,
            to_natives: 4,
            not_checked: 0,
            known_missing_main: 0,
            parsed: usize::from(ScriptBundle::SYNTAX_CHECKED),
        }
    );
}

#[test]
fn client_install_a_scripts_load_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    let counts = check("client install A", &both(&install_a));
    // As the server install, with `editor-internal`, `editor-samples` and `experimental_spawn_rules`, whose
    // one script is read (its module gets as far as the main file) and not looked at.
    assert_eq!(
        counts,
        Counts {
            packs: 153,
            bundles: 7,
            plugins: 3,
            files: 18,
            read: 4,
            read_bytes: 4_263_864,
            others: 0,
            imports: 10,
            to_files: 0,
            to_natives: 9,
            not_checked: 1,
            known_missing_main: 1,
            parsed: if ScriptBundle::SYNTAX_CHECKED { 3 } else { 0 },
        }
    );
}

#[test]
fn client_install_b_scripts_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let counts = check("client install B", &both(&install_b));
    // The three library packs and `editor`.
    assert_eq!(
        counts,
        Counts {
            packs: 143,
            bundles: 4,
            plugins: 1,
            files: 15,
            read: 1,
            read_bytes: 2_535_300,
            others: 0,
            imports: 4,
            to_files: 0,
            to_natives: 4,
            not_checked: 0,
            known_missing_main: 0,
            parsed: usize::from(ScriptBundle::SYNTAX_CHECKED),
        }
    );
}

#[test]
fn client_pack_tree_scripts_load_without_errors() {
    let packs = skip!(game_files::SPLIT_TREE);
    let mut dirs = both(&packs.join("server"));
    dirs.extend(both(&packs.join("client")));
    let counts = check("packs", &dirs);
    // The server and the client tree, each with the seven script packs of client install A.
    assert_eq!(
        counts,
        Counts {
            packs: 231,
            bundles: 14,
            plugins: 6,
            files: 36,
            read: 8,
            read_bytes: 8_527_727,
            others: 0,
            imports: 20,
            to_files: 0,
            to_natives: 18,
            not_checked: 2,
            known_missing_main: 2,
            parsed: if ScriptBundle::SYNTAX_CHECKED { 6 } else { 0 },
        }
    );
}

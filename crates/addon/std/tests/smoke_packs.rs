//! Pack-opening smoke over the game's own files: every pack of the server install, both client
//! installs, the sample packs and the split client pack tree opens through `Driver::pack_dir` /
//! `Driver::pack_archive` with no `OpenError`, and `Strategy::Sequential` and
//! `Strategy::Rayon` give the same identity and report for each pack.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_packs`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::pack::{Pack, PackOrigin};
use bedrock_addon_std::{DirSource, Driver, LoadError, Options, Strategy};

use common::game_files;

/// The children of `dir`, sorted: each one pack candidate.
fn children(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut children: Vec<PathBuf> = entries.map(|entry| entry.unwrap().path()).collect();
    children.sort();
    children
}

/// The packs directories (`*_packs`) directly below `entry` whose names are in `names`.
fn packs_dirs(entry: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| entry.join(name))
        .filter(|dir| dir.is_dir())
        .collect()
}

/// Every directory named `*_packs` at any depth below `root` (the client pack tree).
fn packs_dirs_below(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for child in children(&dir) {
            if !child.is_dir() {
                continue;
            }
            let is_packs_dir = child
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_packs"));
            if is_packs_dir {
                found.push(child);
            } else {
                pending.push(child);
            }
        }
    }
    found.sort();
    found
}

/// Opens one candidate: a folder as a pack folder, a file as an archive.
fn open(driver: &mut Driver, candidate: &Path) -> Result<Pack, LoadError> {
    if candidate.is_dir() {
        return driver.pack_dir(candidate);
    }
    #[cfg(feature = "zip")]
    {
        driver.pack_archive(candidate)
    }
    #[cfg(not(feature = "zip"))]
    {
        driver.pack_dir(candidate)
    }
}

/// What a strategy produced for each candidate: the pack's identity and report, or the error.
type Opened = Vec<(PathBuf, Result<String, String>)>;

/// What a strategy produced for each candidate, and the warnings of the reports of the packs
/// that opened, by code.
fn open_all(strategy: Strategy, candidates: &[PathBuf]) -> (Opened, BTreeMap<&'static str, usize>) {
    let mut driver = Driver::new(Options::default().with_strategy(strategy)).unwrap();
    let mut warnings = BTreeMap::new();
    let opened = candidates
        .iter()
        .map(|candidate| {
            let outcome = open(&mut driver, candidate)
                .map(|pack| {
                    let found = game_files::warnings_by_code(pack.report().diagnostics().iter());
                    for (code, count) in found {
                        *warnings.entry(code).or_insert(0) += count;
                    }
                    format!("{:?} {:?} {:?}", pack.id(), pack.pack_type(), pack.report())
                })
                .map_err(|error| format!("{error:?}"));
            (candidate.clone(), outcome)
        })
        .collect();
    (opened, warnings)
}

/// Opens every candidate sequentially and on rayon, prints the source's report line (packs
/// opened, open errors with their paths, MCB documents), then requires no open error and both
/// strategies to agree; the warnings of the pack reports, by code, are `warnings` exactly, so a
/// game file that gains one fails the smoke.
fn assert_every_pack_opens(source: &str, candidates: &[PathBuf], warnings: &[(&str, usize)]) {
    assert!(
        !candidates.is_empty(),
        "{source}: the game-files entry holds no packs"
    );
    let (sequential, found) = open_all(Strategy::Sequential, candidates);
    let failures: Vec<_> = sequential
        .iter()
        .filter_map(|(path, outcome)| outcome.as_ref().err().map(|error| (path, error)))
        .collect();
    let mcb = candidates
        .iter()
        .map(|candidate| game_files::mcb_on_disk(candidate))
        .sum::<game_files::McbCount>();
    eprintln!(
        "{source}: {} packs opened, {} open errors, {} MCB documents ({} loose, {} in {} brarchives)",
        sequential.len() - failures.len(),
        failures.len(),
        mcb.total(),
        mcb.loose,
        mcb.archived,
        mcb.brarchives,
    );
    for (path, error) in &failures {
        eprintln!("{source}: open error at {}: {error}", path.display());
    }
    assert!(
        failures.is_empty(),
        "{source}: {} packs did not open: {failures:#?}",
        failures.len()
    );
    eprintln!("{source}: warnings of the pack reports {found:?}");
    game_files::assert_warnings(source, &found, warnings);
    #[cfg(feature = "rayon")]
    {
        let (parallel, _) = open_all(Strategy::Rayon { threads: None }, candidates);
        assert_eq!(sequential.len(), parallel.len());
        for (sequential, parallel) in sequential.iter().zip(&parallel) {
            assert_eq!(sequential, parallel, "{source}: the strategies disagree");
        }
        eprintln!(
            "{source}: Sequential and Rayon agree on {} packs",
            parallel.len()
        );
    }
}

fn candidates_in(dirs: &[PathBuf]) -> Vec<PathBuf> {
    dirs.iter().flat_map(|dir| children(dir)).collect()
}

#[test]
fn the_game_files_layout_is_reported() {
    let Some(root) = game_files::root() else {
        eprintln!("skipped: {} is not set", game_files::VARIABLE);
        return;
    };
    assert!(
        root.is_dir(),
        "{} is not a directory: {}",
        game_files::VARIABLE,
        root.display()
    );
    for entry in game_files::ENTRIES {
        if game_files::source(entry).is_none() {
            eprintln!("{}", game_files::missing(entry));
        }
    }
}

#[test]
fn server_install_packs_open() {
    let server = skip!(game_files::SERVER_INSTALL);
    let dirs = packs_dirs(
        &server,
        &[
            "behavior_packs",
            "resource_packs",
            "development_behavior_packs",
            "development_resource_packs",
            "development_skin_packs",
        ],
    );
    assert_every_pack_opens(
        "server install",
        &candidates_in(&dirs),
        &[], // a refused capability is an information of the crate's own (the game is silent)
    );
}

#[test]
fn client_install_a_packs_open() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dirs = packs_dirs(&client, &["behavior_packs", "resource_packs", "skin_packs"]);
    assert_every_pack_opens(
        "client install A",
        &candidates_in(&dirs),
        // A `contents.json` without `version` 1 is an information: the game says nothing about
        // its `version`.
        &[], // a refused capability is an information of the crate's own (the game is silent)
    );
}

#[test]
fn client_install_b_packs_open() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dirs = packs_dirs(&client, &["behavior_packs", "resource_packs", "skin_packs"]);
    assert_every_pack_opens(
        "client install B",
        &candidates_in(&dirs),
        // A `contents.json` without `version` 1 is an information: the game says nothing about
        // its `version`.
        &[], // a refused capability is an information of the crate's own (the game is silent)
    );
}

#[test]
fn sample_packs_packs_open() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_every_pack_opens(
        "sample packs",
        &[samples.join("behavior_pack"), samples.join("resource_pack")],
        &[],
    );
}

#[test]
fn client_pack_tree_opens() {
    let packs = skip!(game_files::SPLIT_TREE);
    let dirs = packs_dirs_below(&packs);
    assert!(
        !dirs.is_empty(),
        "no *_packs directory below {}",
        packs.display()
    );
    assert_every_pack_opens(
        "client pack tree",
        &candidates_in(&dirs),
        // A `contents.json` without `version` 1 is an information: the game says nothing about
        // its `version`.
        &[], // a refused capability is an information of the crate's own (the game is silent)
    );
}

#[test]
fn server_install_repository_registers_every_pack() {
    let server = skip!(game_files::SERVER_INSTALL);
    let sources = || {
        packs_dirs(&server, &["behavior_packs", "resource_packs"])
            .into_iter()
            .map(|dir| DirSource::packs(dir, PackOrigin::Package))
            .collect::<Vec<_>>()
    };
    let expected = candidates_in(&packs_dirs(&server, &["behavior_packs", "resource_packs"])).len();
    let identities = |strategy| {
        let mut driver = Driver::new(Options::default().with_strategy(strategy)).unwrap();
        let repository = driver.repository(sources()).unwrap();
        assert!(
            repository.invalid().is_empty(),
            "{:#?}",
            repository.invalid()
        );
        repository
            .packs()
            .map(|pack| format!("{:?} {:?}", pack.id(), pack.report()))
            .collect::<Vec<_>>()
    };
    let sequential = identities(Strategy::Sequential);
    assert_eq!(sequential.len(), expected);
    #[cfg(feature = "rayon")]
    assert_eq!(identities(Strategy::Rayon { threads: None }), sequential);
}

/// The MCB helpers of `common::game_files` against `bedrock_addon`'s brarchive fixture holding
/// one MCB entry, on disk and inside a pack opened from memory.
mod mcb_helpers {
    use bedrock_addon::pack::{OpenOptions, Pack};
    use bedrock_addon::storage::PackPath;

    use super::game_files;

    fn mcb_brarchive() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../storage/tests/fixtures/brarchive/mcb.brarchive")
    }

    const MANIFEST: &[u8] = br#"{
        "format_version": 2,
        "header": { "name": "mcb", "description": "", "uuid": "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21",
                    "version": [1, 0, 0], "min_engine_version": [1, 26, 0] },
        "modules": [{ "type": "data", "uuid": "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40", "version": [1, 0, 0] }]
    }"#;

    #[test]
    fn mcb_on_disk_counts_loose_files_and_brarchive_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::copy(mcb_brarchive(), dir.path().join("cameras.brarchive")).unwrap();
        std::fs::write(dir.path().join("loose.json"), b"\x7FMCB\x01\x00").unwrap();
        std::fs::write(dir.path().join("plain.json"), b"{}").unwrap();
        let count = game_files::mcb_on_disk(dir.path());
        assert_eq!(
            count,
            game_files::McbCount {
                loose: 1,
                archived: 1,
                brarchives: 1,
            }
        );
        assert_eq!(count.total(), 2);
    }

    #[test]
    fn mcb_documents_counts_the_resident_mcb_files_of_a_pack() {
        let pack = Pack::from_files(
            [
                (PackPath::from_static("manifest.json"), MANIFEST.to_vec()),
                (
                    PackPath::from_static("loose.json"),
                    b"\x7FMCB\x01\x00".to_vec(),
                ),
                (PackPath::from_static("plain.json"), b"{}".to_vec()),
                (
                    PackPath::from_static("__brarchive/cameras.brarchive"),
                    std::fs::read(mcb_brarchive()).unwrap(),
                ),
            ],
            &OpenOptions::default(),
        )
        .unwrap();
        assert!(game_files::is_mcb_document(
            &pack,
            &PackPath::from_static("loose.json")
        ));
        assert!(!game_files::is_mcb_document(
            &pack,
            &PackPath::from_static("plain.json")
        ));
        assert_eq!(
            game_files::mcb_documents(&pack),
            2,
            "the loose file and the brarchive entry"
        );
    }
}

//! Data-driven UI smoke over the game's own files: the `ddui/root/` files of the client installs'
//! resource packs load through `Driver` and `rp.data_driven_ui()`. Neither install ships a
//! `ddui/composition/` folder and the sample packs ship no `ddui/` at all, so those trees are
//! skipped with a message; compositions are covered by the fixtures of
//! `crates/addon/addon/tests/ddui/`. The documents hold no Molang (`attribs` is opaque to the
//! loader), so the Molang pairs file has no occurrence to check.
//!
//! Each install is loaded as one stack: its packs registered as the app's own (the only kind
//! that may use `minecraft` in an identifier), the world's lists built by
//! `common::world::base_game_world`.
//!
//! The control catalogue is the engine's, not the file's (every file says 1.21.130). The
//! stack reads with the engine the crate tracks, whose catalogue is 1.26.45's: the files of
//! client install B (1.26.45) load without a finding, and the files of client install A
//! (1.26.20) that use `Container.Item` are refused, as a 1.26.45 client refuses them. Read with
//! a 1.26.20 engine, every file of install A loads.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_ddui`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::ddui::UiRoot;
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

const ROOTS: &str = "ddui/root";
const COMPOSITIONS: &str = "ddui/composition";

/// A driver that opens packs as the app's own.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The `.json` files directly in `dir`, sorted.
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
}

/// Every directory of a packs directory, sorted.
fn packs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// What one install reported.
#[derive(Debug, Default)]
struct Tally {
    /// Packs with `ddui/root`.
    packs: usize,
    /// `.json` files in `ddui/root` of those packs.
    root_files: usize,
    /// `.json` files in `ddui/composition`.
    composition_files: usize,
    roots: usize,
    compositions: usize,
    /// Files refused, by file name.
    rejected: Vec<String>,
    /// Error-level findings other than the loader's closing line, as `code: text spanned`.
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
    infos: BTreeMap<&'static str, usize>,
}

/// Loads the resource stack of the install at `root` and counts what it reports.
fn load(root: &Path, source: &str) -> Tally {
    let mut driver = base_game_driver();
    let repository: PackRepository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    let mut tally = Tally::default();
    let mut mcb_on_disk = 0;
    for dir in packs(&root.join("resource_packs")) {
        if !dir.join("manifest.json").is_file() {
            continue;
        }
        tally.composition_files += json_files(&dir.join(COMPOSITIONS)).len();
        if !dir.join(ROOTS).is_dir() {
            continue;
        }
        tally.packs += 1;
        tally.root_files += json_files(&dir.join(ROOTS)).len();
        mcb_on_disk += game_files::mcb_on_disk(&dir.join(ROOTS)).total();
    }
    let options = LoadOptions::default();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(&repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let ui = stacks.resource.data_driven_ui();
    tally.roots = ui.roots.len();
    tally.compositions = ui.compositions.len();
    tally.rejected = ui
        .roots
        .rejected()
        .iter()
        .chain(ui.compositions.rejected())
        .map(|rejected| {
            let path = rejected.origin.path.as_str();
            path.rsplit('/').next().unwrap_or(path).to_owned()
        })
        .collect();
    tally.rejected.sort();
    let mut unsupported = 0;
    for diagnostic in ui.diagnostics() {
        let name = diagnostic.code.name();
        if name == "UnsupportedFileFormat" {
            unsupported += 1;
        } else if diagnostic.severity >= Severity::Error {
            if name != "DocumentRejected" {
                tally.errors.push(format!("{name}: {diagnostic}"));
            }
        } else if diagnostic.severity == Severity::Warning {
            *tally.warnings.entry(name).or_insert(0) += 1;
        } else {
            *tally.infos.entry(name).or_insert(0) += 1;
        }
    }
    tally.mcb = unsupported;
    assert!(
        unsupported <= mcb_on_disk,
        "{source}: {unsupported} MCB findings"
    );
    eprintln!(
        "{source}: {} packs with {ROOTS}, {} root files, {} composition files, {} roots, {} compositions, {} rejected {:?}, {} errors, {} MCB documents, warnings {:?}, infos {:?}",
        tally.packs,
        tally.root_files,
        tally.composition_files,
        tally.roots,
        tally.compositions,
        tally.rejected.len(),
        tally.rejected,
        tally.errors.len(),
        tally.mcb,
        tally.warnings,
        tally.infos
    );
    if tally.composition_files == 0 {
        eprintln!("skip: {source} ships no {COMPOSITIONS}/ (compositions are fixture-only)");
    }
    tally
}

/// Client install B (1.26.45): every root loads on the engine the crate tracks, with no
/// finding at all.
#[test]
fn client_install_b_roots_load_without_a_finding() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = load(&client, "client install B resource_packs");
    assert!(tally.errors.is_empty(), "{:#?}", tally.errors);
    assert!(tally.rejected.is_empty(), "{:?}", tally.rejected);
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    assert!(tally.infos.is_empty(), "{:?}", tally.infos);
    assert_eq!(
        (
            tally.packs,
            tally.root_files,
            tally.roots,
            tally.compositions
        ),
        (1, 8, 8, 0)
    );
    assert_eq!(tally.composition_files, 0);
}

/// Client install A (1.26.20) on the engine the crate tracks (catalogue 1.26.45): the six
/// container screens use `Container.Item`, which that catalogue does not have, and are
/// refused; the two forms load. Nothing else is reported.
#[test]
fn client_install_a_roots_that_use_container_item_are_refused_by_the_later_catalogue() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let tally = load(&client, "client install A resource_packs");
    assert_eq!(
        (
            tally.packs,
            tally.root_files,
            tally.roots,
            tally.compositions
        ),
        (1, 8, 2, 0)
    );
    assert_eq!(
        tally.rejected,
        [
            "barrel_screen.json",
            "chest_screen.json",
            "copper_chest_screen.json",
            "default_chest_screen.json",
            "ender_chest_screen.json",
            "shulker_box_screen.json"
        ]
    );
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    // Each refused root has the hint that an earlier engine had the type (`FieldUntilEngine`).
    let infos: Vec<(&str, usize)> = tally
        .infos
        .iter()
        .map(|(code, count)| (&**code, *count))
        .collect();
    assert_eq!(infos, [("FieldUntilEngine", 6)]);
    assert!(
        tally
            .errors
            .iter()
            .all(|error| error.starts_with("InvalidValue: ")),
        "{:#?}",
        tally.errors
    );
    // `component` that names `Container.Item`, beside the six hints.
    assert_eq!(tally.errors.len(), 6, "{:#?}", tally.errors);
}

/// The same files of client install A read by the engine they ship with: all eight load without a finding.
#[test]
fn client_install_a_roots_load_on_their_own_engine() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dir = client.join("resource_packs/vanilla_1.21.130").join(ROOTS);
    let files = json_files(&dir);
    assert_eq!(files.len(), 8, "{}", dir.display());
    let mut cx = LoadContext::detached();
    cx.engine_version = FormatVersion::new(1, 26, 20);
    cx.is_base_game_pack = true;
    for file in files {
        let bytes = std::fs::read(&file).unwrap();
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let path = PackPath::new(&format!("{ROOTS}/{name}")).expect("a pack path");
        match UiRoot::parse(&path, &bytes, &cx) {
            Ok(document) => {
                assert!(document.diagnostics().is_empty(), "{name}");
                assert!(
                    document
                        .description
                        .identifier
                        .as_str()
                        .starts_with("minecraft:")
                );
            }
            Err(rejected) => panic!("{name}: {:#?}", rejected.diagnostics),
        }
    }
}

/// The sample packs ship no `ddui/`: nothing to load, said aloud.
#[test]
fn sample_packs_ships_no_ddui() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let dir = samples.join("resource_pack").join("ddui");
    assert!(
        !dir.exists(),
        "{} appeared: add it to the smoke",
        dir.display()
    );
    eprintln!(
        "skip: the sample packs ship no ddui/ (roots are checked against the client installs)"
    );
}

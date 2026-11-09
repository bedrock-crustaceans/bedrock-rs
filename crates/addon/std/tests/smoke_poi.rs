//! Vanilla smoke for the POI documents: the POI directories of the sample packs'
//! `behavior_pack/`, of the server install's behaviour packs and of the behaviour packs of
//! client install A (which include `experimental_poi`) load through the `Driver` with the `poi`
//! experiment on and zero error-level diagnostics; MCB documents (baked documents the loaders
//! do not decode) are counted, not failed.
//!
//! No vanilla pack ships a POI document yet (the 1.26.20 `experimental_poi` pack holds a
//! manifest and `contents.json` only; the sample packs have no POI folder), so today every
//! source reports zero documents; the tests keep the walk honest for the day one does.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_poi`;
//! without the variable every test skips.

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::load::{Experiment, Experiments, LoadOptions, StackOptions};
use bedrock_addon::pack::{Pack, PackOrigin, PackRepository, PackType};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// The directories the POI walk reads: types from `poi/blocks`, tags from `poi/tags`, no
/// sub-folder.
const DIRECTORIES: [&str; 2] = ["poi/blocks", "poi/tags"];

/// What a load found.
struct Outcome {
    types: usize,
    tags: usize,
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    /// Warnings and infos by code, for the report.
    findings: Vec<String>,
}

/// The MCB documents on disk below the pack roots `roots`: the POI directories and their
/// brarchives.
fn poi_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            DIRECTORIES.iter().flat_map(move |directory| {
                [
                    root.join(directory),
                    root.join("__brarchive").join(directory),
                    root.join("__brarchive")
                        .join(format!("{directory}.brarchive")),
                ]
            })
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path))
        .sum()
}

/// The POI files on disk below the pack roots `roots` (loose files directly in a POI
/// directory only: the walks read no sub-folder).
fn poi_files(roots: &[PathBuf]) -> usize {
    fn count(dir: &Path) -> usize {
        std::fs::read_dir(dir).map_or(0, |entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| {
                    let path = entry.path();
                    usize::from(path.is_file() && path.extension().is_some_and(|e| e == "json"))
                })
                .sum()
        })
    }
    roots
        .iter()
        .flat_map(|root| DIRECTORIES.iter().map(move |d| root.join(d)))
        .filter(|path| path.is_dir())
        .map(|path| count(&path))
        .sum()
}

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The world options, with or without the `poi` experiment.
fn options(poi: bool) -> LoadOptions {
    let mut experiments = Experiments::new();
    if poi {
        experiments.insert(Experiment::Poi);
    }
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

/// Loads the behaviour stack of `repository` with `driver` and collects the POI findings; an
/// `UnsupportedFileFormat` finding is an MCB document as long as the helper counted as many on
/// disk below `roots`, an error otherwise, so an unsupported non-MCB file cannot pass silently.
fn load_poi(driver: &Driver, repository: &PackRepository, roots: &[PathBuf], poi: bool) -> Outcome {
    let on_disk = poi_mcb(roots).total();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(repository, &options(poi)), options(poi)),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let group = stacks.behavior.poi();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut findings = Vec::new();
    for finding in group.diagnostics() {
        let at = format!(
            "{} {}",
            finding
                .location
                .path
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
            finding.location.json_path
        );
        if finding.severity < Severity::Error {
            findings.push(format!("{} {}", finding.severity, finding.code.id()));
            continue;
        }
        if finding.is(&codes::UNSUPPORTED_FILE_FORMAT) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(format!("{at} {} {:?}", finding.code, finding.code));
    }
    Outcome {
        types: group.types.len(),
        tags: group.tags.len(),
        rejected: group.rejected().count(),
        mcb,
        errors,
        findings,
    }
}

/// Prints the source's line and requires no error outside MCB documents, and every loose POI
/// file on disk accounted for.
fn report(source: &str, outcome: &Outcome, roots: &[PathBuf]) {
    let on_disk = poi_mcb(roots).total();
    let files = poi_files(roots);
    eprintln!(
        "{source}: {files} POI files on disk, {} types, {} tags, {} rejected, {} MCB documents \
         ({on_disk} on disk), {} errors",
        outcome.types,
        outcome.tags,
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
    );
    let mut kinds: Vec<(&str, usize)> = Vec::new();
    for finding in &outcome.findings {
        match kinds.iter_mut().find(|(k, _)| *k == finding.as_str()) {
            Some((_, count)) => *count += 1,
            None => kinds.push((finding.as_str(), 1)),
        }
    }
    for (kind, count) in &kinds {
        eprintln!("  {count} × {kind}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    // No warning, with or without a POI file on disk: the pin is exact.
    let warnings: BTreeMap<&str, usize> = kinds
        .iter()
        .filter_map(|(kind, count)| Some((kind.strip_prefix("warning ")?, *count)))
        .collect();
    game_files::assert_warnings(source, &warnings, &[]);
    assert!(
        outcome.types + outcome.tags + outcome.rejected + outcome.mcb <= files + on_disk,
        "{source}: more documents than files"
    );
    if files == 0 && on_disk == 0 {
        assert_eq!(
            (outcome.types, outcome.tags, outcome.rejected),
            (0, 0, 0),
            "{source}: nothing on disk, nothing loaded"
        );
        assert!(
            outcome.findings.is_empty(),
            "{source}: no POI file, no finding: {:?}",
            outcome.findings
        );
    }
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// Copies the directory `from` to `to`, recursively.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// No game tree has a POI document, so the driver's path is exercised with the 1.26.20
/// hard-coded set written as documents (the fixture `poi30_vanilla_1_26_20_set`): a pack
/// directory on disk, read through the `Driver`. Needs no game files.
#[test]
fn the_vanilla_1_26_20_set_loads_from_disk_through_the_driver() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../addon/tests/fixtures/poi/poi/poi30_vanilla_1_26_20_set");
    let dir = tempfile::tempdir().expect("a temp dir");
    let root = dir.path().join("vanilla_poi");
    copy_tree(&fixture, &root);
    std::fs::write(
        root.join("manifest.json"),
        r#"{"format_version": 2, "header": {"name": "vanilla poi", "uuid": "32dd7cc5-e6fb-46f3-ace9-15aa250989a6", "version": [0, 0, 1], "min_engine_version": [1, 26, 40]}, "modules": [{"type": "data", "uuid": "4a0bba2c-0a1d-4765-a64a-1fde28c6b110", "version": [0, 0, 1]}]}"#,
    )
    .unwrap();
    let mut driver = base_game_driver();
    let pack: Pack = driver.pack_dir(&root).expect("the pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let roots = [root];
    let outcome = load_poi(&driver, &repository, &roots, true);
    report("poi30 on disk (poi on)", &outcome, &roots);
    assert_eq!((outcome.types, outcome.tags), (19, 3));
    assert_eq!(outcome.rejected, 0);
    assert!(outcome.findings.is_empty(), "{:?}", outcome.findings);
    // Without the experiment the same pack loads nothing and says so once.
    let outcome = load_poi(&driver, &repository, &roots, false);
    assert_eq!((outcome.types, outcome.tags, outcome.rejected), (0, 0, 0));
    assert_eq!(outcome.findings, ["info core/missing_toggles"]);
    assert!(outcome.errors.is_empty());
}

#[test]
fn sample_packs_poi_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack: Pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let roots = [samples.join("behavior_pack")];
    for poi in [true, false] {
        let outcome = load_poi(&driver, &repository, &roots, poi);
        report(
            if poi {
                "sample packs (poi on)"
            } else {
                "sample packs (poi off)"
            },
            &outcome,
            &roots,
        );
    }
}

#[test]
fn server_install_behavior_packs_poi_loads_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            server.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let roots = pack_roots(&server.join("behavior_packs"));
    let outcome = load_poi(&driver, &repository, &roots, true);
    report("server install", &outcome, &roots);
}

#[test]
fn client_install_a_behavior_packs_poi_loads_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            client.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let roots = pack_roots(&client.join("behavior_packs"));
    // Client install A ships the experiment's behaviour pack.
    assert!(
        roots.iter().any(|root| root.ends_with("experimental_poi")),
        "client install A has the experimental_poi behaviour pack"
    );
    // And it is one of the packs that are loaded: the repository opened it, as a behaviour
    // pack, under vanilla's UUID, and the world list holds it.
    let experimental_poi = "32dd7cc5-e6fb-46f3-ace9-15aa250989a6";
    assert!(
        repository.packs().any(|pack| {
            pack.id().uuid.to_string() == experimental_poi && pack.pack_type() == PackType::Behavior
        }),
        "the repository holds the experimental_poi behaviour pack"
    );
    let stacks = repository.stacks(&base_game_world(&repository, &options(true)), options(true));
    assert!(
        stacks
            .behavior
            .packs()
            .any(|pack| pack.id().uuid.to_string() == experimental_poi),
        "the behaviour stack holds the experimental_poi pack"
    );
    let experimental: Vec<PathBuf> = roots
        .iter()
        .filter(|root| root.ends_with("experimental_poi"))
        .cloned()
        .collect();
    eprintln!(
        "client install A experimental_poi: {} POI files on disk",
        poi_files(&experimental)
    );
    for poi in [true, false] {
        let outcome = load_poi(&driver, &repository, &roots, poi);
        report(
            if poi {
                "client install A (poi on)"
            } else {
                "client install A (poi off)"
            },
            &outcome,
            &roots,
        );
    }
}

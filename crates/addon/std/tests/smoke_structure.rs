//! Vanilla smoke for structures: the `.mcstructure` files of the server install's behaviour
//! packs, of the packs of client installs B and A, and of the sample packs' `behavior_pack/`
//! (which ships none) load through the `Driver` with zero error-level diagnostics and are
//! written back byte for byte. The Java `.nbt` templates of the base-game packs are vanilla's
//! jigsaw data, not read and not reported. MCB documents (baked documents the loaders do not
//! decode) are counted, not failed.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_structure`; without the variable every test skips.

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::structure::Structure;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{Pack, PackOrigin, PackRef, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// What a load found.
struct Outcome {
    structures: usize,
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    infos: Vec<String>,
    /// Structures whose pack folder has no loose file at their path.
    no_loose_file: Vec<String>,
    /// Structures whose written bytes differ from their pack's loose file.
    not_exact: Vec<String>,
}

/// The MCB documents on disk below the pack roots `roots`' `structures/`.
fn structure_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join("structures"),
                root.join("__brarchive").join("structures.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path))
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

/// The folder of the behaviour pack `pack` of `repository`: `packs_dir` joined with the pack's
/// place below its source (empty for a pack opened from its own folder).
fn pack_folder(repository: &PackRepository, pack: &PackRef, packs_dir: &Path) -> Option<PathBuf> {
    repository
        .packs()
        .find(|candidate| candidate.id() == pack.id())
        .map(|found| packs_dir.join(found.location().path.as_str()))
}

/// Loads the behaviour stack of `repository` and collects the structure findings; writes
/// every effective structure back and compares it with the loose file of its own pack, whose
/// folder is below `packs_dir` (the directory the behaviour packs were opened from).
fn load_structures(driver: &Driver, repository: &PackRepository, packs_dir: &Path) -> Outcome {
    let roots = if packs_dir.join("manifest.json").exists() {
        vec![packs_dir.to_path_buf()]
    } else {
        pack_roots(packs_dir)
    };
    let on_disk = structure_mcb(&roots).total();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let structures = stacks.behavior.structures();
    let mut outcome = Outcome {
        structures: structures.iter().count(),
        rejected: structures.rejected().len(),
        mcb: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
        infos: Vec::new(),
        no_loose_file: Vec::new(),
        not_exact: Vec::new(),
    };
    for finding in structures.diagnostics() {
        match finding.severity {
            Severity::Info => outcome.infos.push(finding.code.id().to_owned()),
            Severity::Warning => outcome.warnings.push(finding.code.id().to_owned()),
            _ => {}
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && outcome.mcb < on_disk {
            outcome.mcb += 1;
            continue;
        }
        outcome.errors.push(format!(
            "{} {} {}",
            finding
                .location
                .path
                .as_ref()
                .map(std::string::ToString::to_string)
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
        ));
    }
    for (path, document) in structures {
        assert!(
            Structure::id_of(path).is_some(),
            "{path} is a listed structure"
        );
        // The file as stored in the structure's own pack folder.
        let written = document.to_bytes().expect("writes");
        let origin = document.origin();
        let loose = pack_folder(repository, &origin.pack, packs_dir)
            .map(|folder| folder.join(origin.path.as_str()))
            .and_then(|file| std::fs::read(file).ok());
        match loose {
            None => outcome
                .no_loose_file
                .push(format!("{} {path}", origin.pack.name())),
            Some(bytes) if bytes != written => outcome
                .not_exact
                .push(format!("{} {path}", origin.pack.name())),
            Some(_) => {}
        }
    }
    outcome
}

/// Prints the source's line and its findings by code; requires no error outside MCB documents
/// and every structure written back as read.
fn report(source: &str, outcome: &Outcome) {
    eprintln!(
        "{source}: {} structures, {} rejected, {} MCB documents, {} errors, {} warnings, {} infos",
        outcome.structures,
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
        outcome.warnings.len(),
        outcome.infos.len()
    );
    for (severity, list) in [("warning", &outcome.warnings), ("info", &outcome.infos)] {
        let mut kinds: Vec<(&String, usize)> = Vec::new();
        for code in list {
            match kinds.iter_mut().find(|(kind, _)| *kind == code) {
                Some((_, count)) => *count += 1,
                None => kinds.push((code, 1)),
            }
        }
        for (code, count) in kinds {
            eprintln!("  {count} × {severity}: {code}");
        }
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    // No tree has a structure warning: the pin is exact.
    game_files::assert_warnings(source, &game_files::count_names(&outcome.warnings), &[]);
    assert!(
        outcome.no_loose_file.is_empty(),
        "{source}: no loose file in the structure's pack: {:#?}",
        outcome.no_loose_file
    );
    assert!(
        outcome.not_exact.is_empty(),
        "{source}: not written back byte for byte: {:#?}",
        outcome.not_exact
    );
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The behaviour packs of an install at `root` (with its resource packs, which the unversioned
/// `vanilla` behaviour pack, holder of the bastion mob structures, depends on); returns how many
/// structures loaded.
fn install(root: &Path, source: &str) -> usize {
    let mut driver = base_game_driver();
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    let outcome = load_structures(&driver, &repository, &root.join("behavior_packs"));
    report(source, &outcome);
    outcome.structures
}

#[test]
fn server_install_structures_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    // `vanilla` (five bastion mobs) and `vanilla_1.26.30` (ten sulfur springs).
    assert_eq!(install(&server, "server install"), 15);
}

#[test]
fn client_install_b_structures_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_eq!(install(&client, "client install B"), 15);
}

#[test]
fn client_install_a_structures_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    // `vanilla` (five bastion mobs) and `experimental_y_2026_drop_2` (ten sulfur springs).
    assert_eq!(install(&client, "client install A"), 15);
}

#[test]
fn sample_packs_structures_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack: Pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let outcome = load_structures(&driver, &repository, &samples.join("behavior_pack"));
    report("sample packs", &outcome);
}

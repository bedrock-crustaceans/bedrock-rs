#![allow(clippy::pedantic, missing_docs)]
//! Spawn smoke over the game's own files: the vanilla spawn rules of the sample packs'
//! `behavior_pack/` and of the server install's behaviour packs load through `Driver` into
//! `bp.spawn_rules()` with zero error-level diagnostics, with the Simplified Spawn Rules
//! experiment off and on; the raid spawn groups of the client and server app data
//! (`definitions/spawn_groups`, read with `read_app_data`) load into `bp.spawn_groups()`, all
//! seven, without a finding. MCB documents (baked documents the loaders do not decode) are
//! counted apart.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_spawn`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::spawn_rules::SpawnRules;
use bedrock_addon::id::SpawnGroupId;
use bedrock_addon::load::{Experiment, Experiments, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy, read_app_data};

use common::game_files;
use common::world::world_list;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The spawn-rules MCB documents on disk in the packs below `dirs` (`spawn_rules/` and the
/// `spawn_rules` brarchive).
fn spawn_mcb(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join("spawn_rules"),
                root.join("__brarchive").join("spawn_rules.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// The world options: the experiment on or off, the app data when given.
fn options(experiment: bool, app_data: Option<&Path>) -> LoadOptions {
    let mut experiments = Experiments::new();
    if experiment {
        experiments.insert(Experiment::SimplifiedSpawnRules);
    }
    LoadOptions {
        experiments,
        app_data: app_data.map(|dir| read_app_data(dir).expect("the app data reads")),
        ..LoadOptions::default()
    }
}

/// The stacks of the packs directories `dirs` (all `Package` origin) for the world's behaviour
/// list `world_behavior`.
fn stacks(dirs: &[&Path], world_behavior: &[u8], options: LoadOptions) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    driver
        .load_stacks(
            repository.stacks(&world_list(&repository, world_behavior, &options), options),
            StackOptions::default(),
        )
        .unwrap()
}

/// Loads the spawn rules of `dirs` with the experiment off and on: how many loaded and in which
/// shape, warnings by code, and no error but MCB documents.
fn check_rules(source: &str, dirs: &[&Path], world_behavior: &[u8]) {
    let on_disk = spawn_mcb(dirs);
    for experiment in [false, true] {
        let set = stacks(dirs, world_behavior, options(experiment, None));
        let rules = set.behavior.spawn_rules();
        let mut mcb = 0;
        let mut errors = Vec::new();
        let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
        for finding in rules.diagnostics() {
            if finding.severity == Severity::Warning {
                *warnings.entry(finding.code.name()).or_default() += 1;
            }
            if finding.severity < Severity::Error {
                continue;
            }
            if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb < on_disk {
                mcb += 1;
                continue;
            }
            errors.push(finding.to_string());
        }
        let simplified = rules
            .iter()
            .filter(|(_, d)| matches!(d.value(), SpawnRules::Simplified(_)))
            .count();
        eprintln!(
            "{source} (experiment {}): {} spawn rules ({simplified} simplified) over {} packs, {} rejected files, {mcb} MCB documents ({on_disk} on disk), {} errors, warnings {warnings:?}",
            if experiment { "on" } else { "off" },
            rules.len(),
            set.behavior.packs().len(),
            rules.rejected().len(),
            errors.len(),
        );
        for error in &errors {
            eprintln!("{source}: {error}");
        }
        assert!(errors.is_empty(), "{source}: {} errors", errors.len());
        // Neither tree has a spawn rule warning, with the experiment off or on: the pin is exact.
        game_files::assert_warnings(source, &warnings, &[]);
        assert_eq!(
            rules.rejected().len(),
            mcb,
            "{source}: only MCB documents are rejected"
        );
    }
}

#[test]
fn sample_packs_spawn_rules_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The sample pack has its own UUID: the world selects it from the vanilla location.
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    check_rules("sample packs", &[&samples], world);
}

#[test]
fn server_install_spawn_rules_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    check_rules("server install", &[&server.join("behavior_packs")], b"[]");
}

/// Loads the app data at `root` (the directory holding `definitions/`) under an empty
/// behaviour stack and checks the seven raid groups.
fn check_groups(source: &str, root: &Path, packs: &Path) {
    let set = stacks(&[packs], b"[]", options(false, Some(root)));
    let groups = set.behavior.spawn_groups();
    let findings: Vec<String> = groups.diagnostics().map(ToString::to_string).collect();
    eprintln!(
        "{source}: {} spawn groups, {} rejected files, {} findings",
        groups.len(),
        groups.rejected().len(),
        findings.len()
    );
    for finding in &findings {
        eprintln!("{source}: {finding}");
    }
    assert!(findings.is_empty(), "{source}: {} findings", findings.len());
    assert!(groups.rejected().is_empty());
    assert_eq!(groups.len(), 7);
    for wave in 0..7 {
        let id = SpawnGroupId::parse(format!("minecraft:raid_group_{wave}"));
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("{source}: {id:?}"));
        assert!(!group.spawn_rules().is_empty());
    }
}

#[test]
fn client_spawn_groups_load_without_findings() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    check_groups(
        "client install B",
        &install_b,
        &install_b.join("behavior_packs"),
    );
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check_groups(
        "client install A",
        &install_a,
        &install_a.join("behavior_packs"),
    );
}

#[test]
fn server_install_spawn_groups_load_without_findings() {
    let server = skip!(game_files::SERVER_INSTALL);
    check_groups("server install", &server, &server.join("behavior_packs"));
}

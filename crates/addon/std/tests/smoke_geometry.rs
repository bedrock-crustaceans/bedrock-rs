//! Geometry smoke over the game's own files: every geometry file of the resource packs of
//! the sample packs and of the client installs (`models/mobs.json`, `models/entity/**`,
//! `models/blocks/**`) loads through `Driver` into `rp.geometries()` with no error-level
//! diagnostic, the Molang compile check on (MCB documents, baked documents the loaders do not
//! decode, are counted apart),
//! and every `models` occurrence of the Molang pairs file (bone `binding`) loads with its text
//! and version.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_geometry`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The geometry files on disk in the packs below `dir`: JSON files of `models/mobs.json`,
/// `models/entity/**` and `models/blocks/**`, and the MCB documents among them and in the
/// `models` brarchive.
fn on_disk(dir: &Path) -> (usize, usize) {
    fn json_below(dir: &Path, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                json_below(&path, count);
            } else if path.extension().is_some_and(|e| e == "json") {
                *count += 1;
            }
        }
    }
    let (mut files, mut mcb) = (0, 0);
    for root in pack_roots(dir) {
        let models = root.join("models");
        files += usize::from(models.join("mobs.json").is_file());
        json_below(&models.join("entity"), &mut files);
        json_below(&models.join("blocks"), &mut files);
        for candidate in [
            models.clone(),
            root.join("__brarchive").join("models.brarchive"),
        ] {
            if candidate.exists() {
                mcb += game_files::mcb_on_disk(&candidate).total();
            }
        }
    }
    (files, mcb)
}

/// The stacks of every pack below `dir` (all `Package` origin), each resource pack listed.
fn stacks(dir: &Path) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::Package)])
        .unwrap();
    // The Molang compile check on: every `binding` of the vanilla files compiles at its pack's
    // version.
    let options = LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        ..LoadOptions::default()
    };
    driver
        .load_stacks(
            repository.stacks(&base_game_world(&repository, &options), options),
            StackOptions::default(),
        )
        .unwrap()
}

/// Loads the geometry of every resource pack below `dir` and requires no error but MCB
/// documents, and the warnings by code to be `expected` exactly; the count of registered
/// geometries.
fn check(source: &str, dir: &Path, expected: &[(&str, usize)]) -> usize {
    let (files, mcb_on_disk) = on_disk(dir);
    let set = stacks(dir);
    let geometries = set.resource.geometries();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    let mut notes: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in geometries.diagnostics() {
        match finding.severity {
            Severity::Warning => *warnings.entry(finding.code.id()).or_default() += 1,
            severity if severity < Severity::Warning => {
                *notes.entry(finding.code.id()).or_default() += 1;
            }
            _ => {}
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&codes::UNSUPPORTED_FILE_FORMAT) && mcb < mcb_on_disk {
            mcb += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    let legacy = geometries
        .iter()
        .filter(|(_, d)| d.value().parent().is_some())
        .count();
    let upgraded = geometries
        .iter()
        .filter(|(_, d)| d.version().upgraded)
        .count();
    eprintln!(
        "{source}: {} geometries ({upgraded} from an older format, {legacy} with a legacy parent) of {files} files in {} resource packs, {} rejected, {mcb} MCB documents ({mcb_on_disk} on disk), {} errors, warnings {warnings:?}, notes {notes:?}",
        geometries.len(),
        set.resource.packs().len(),
        geometries.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    game_files::assert_warnings(source, &warnings, expected);
    assert_eq!(
        geometries.rejected().len(),
        mcb,
        "{source}: only MCB documents are rejected"
    );
    assert!(files > 0, "{source}: no geometry file on disk");
    assert!(!geometries.is_empty(), "{source}: no geometry loaded");
    // The geometries that name a legacy parent (`geometry.child:geometry.parent`, the files
    // older than 1.12.0): 24 in each of the three trees, since they are the base pack's own
    // legacy models and no later pack adds one. The number holds only when the stack is
    // walked in vanilla's order (the lowest pack first, `mobs.json` before `models/entity`):
    // a child read before the definition of its parent that stands would be dropped, or be
    // built on a shadowed parent, and the count would move.
    assert_eq!(legacy, 24, "{source}: geometries with a legacy parent");
    // Every legacy child found its parent in the stack.
    for (id, document) in geometries {
        let geometry = document.value();
        assert_eq!(
            geometry.parent().is_some(),
            geometry.inherited.is_some(),
            "{source}: {id:?}"
        );
    }
    geometries.len()
}

/// Every file of the sample packs `resource_pack/models/**` (198 files) loads with no error and
/// no warning, though two files carry a finding each: `zombie_villager.geo.json` defines the
/// bone `hat` twice (`DuplicateBone`, the crate's own) and `geometry.pufferfish.mid` of
/// `pufferfish_v1.0.geo.json` makes `body` its own parent (vanilla's "Error: parenting model
/// part to itself!"). Both are informations: the game's bone merge is silent, and the
/// self-parent text is a development-time assertion.
#[test]
fn s05_sample_packs_geometry_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let (files, _) = on_disk(&samples);
    assert_eq!(files, 198, "sample packs models");
    check("sample packs", &samples, &[]);
    let set = stacks(&samples);
    let mut warnings: Vec<String> = set
        .resource
        .geometries()
        .diagnostics()
        .filter(|d| {
            matches!(
                d.code.id(),
                "core/duplicate_bone"
                    | "core/reference_cycle"
                    | "core/reference_cycle_worded"
                    | "geometry/parent_is_self"
            )
        })
        .map(|d| {
            assert_eq!(d.severity, Severity::Info, "{d}");
            format!(
                "{} {}",
                d.location.path.as_ref().map_or("", |p| p.as_str()),
                d.code
            )
        })
        .collect();
    warnings.sort();
    assert_eq!(
        warnings,
        [
            "models/entity/pufferfish_v1.0.geo.json Error: parenting model part to itself!",
            "models/entity/zombie_villager.geo.json two bones collide after lower-casing",
        ]
    );
}

#[test]
fn client_install_b_geometry_loads_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    check(
        "client install B",
        &install_b.join("resource_packs"),
        // The duplicate bone, the lenient removal and the bone-parent cycle on vanilla's files
        // are informations.
        &[],
    );
}

/// Client install A is a smoke tree like client install B; skipped with a message when absent.
#[test]
fn client_install_a_geometry_loads_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check(
        "client install A",
        &install_a.join("resource_packs"),
        // The duplicate bone, the lenient removal and the bone-parent cycle on vanilla's files
        // are informations.
        &[],
    );
}

/// Every `models` occurrence of the Molang pairs file (the packs of client install B): the bone
/// `binding` at the occurrence's pointer reads to the pair's text at the pack's Molang version.
#[test]
fn molang_pairs_binding_loads_with_its_text_and_version() {
    use bedrock_addon::families::geometry::load_file;

    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let found = pairs::check(&pairs_path, &install_b, "models", |file| {
        let document = load_file(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = document.diagnostics().clone();
        let mut members = pairs::Members::new();
        pairs::members(&mut document.into_value(), None, &mut members);
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs models: {} occurrences checked in {} files, {} failures",
        found.compared,
        found.files,
        found.failures.len()
    );
    assert!(found.failures.is_empty(), "{:#?}", found.failures);
    assert_eq!(found.compared, 10, "the pairs' `models` occurrences");
    assert_eq!(found.files, 5, "the five held-item geometries");
}

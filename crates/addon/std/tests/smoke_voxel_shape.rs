//! Vanilla smoke for voxel shapes: the `shapes/` documents of the sample packs'
//! `behavior_pack/`, of the server install's and of the client installs' behaviour packs load
//! through the `Driver` with zero error-level findings; MCB documents are decoded and read like
//! their JSON twins.
//! Vanilla ships its shapes in `behavior_packs/vanilla/shapes` or in the experimental
//! pack `experimental_vanilla_shapes`, which a world has only with the `voxel_shapes`
//! experiment, so the stacks are composed with it.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_voxel_shape`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::document::Content;
use bedrock_addon::families::voxel_shape::VoxelShape;
use bedrock_addon::load::{Experiment, LoadContext, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon::storage::PackPath;
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

/// The shape MCB documents on disk in the packs below `dir` (the loose directory and its
/// brarchive).
fn mcb(dir: &Path) -> usize {
    pack_roots(dir)
        .into_iter()
        .flat_map(|root| {
            [
                root.join("shapes"),
                root.join("__brarchive").join("shapes.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// The stacks of a world that lists every pack below `root`'s `behavior_packs/` and
/// `resource_packs/` (all `Package` origin; a vanilla behaviour pack depends on its resource
/// pack), or the packs directly below `root` when it has no `behavior_packs/`, with the
/// `voxel_shapes` experiment on.
fn stacks(root: &Path) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            pack_dirs(root)
                .into_iter()
                .map(|dir| DirSource::packs(dir, PackOrigin::Package)),
        )
        .unwrap();
    let options = LoadOptions {
        experiments: [Experiment::VoxelShapes].into_iter().collect(),
        ..LoadOptions::default()
    };
    let world = base_game_world(&repository, &options);
    driver
        .load_stacks(repository.stacks(&world, options), StackOptions::default())
        .unwrap()
}

/// The packs directories of `root`: its `behavior_packs/` and `resource_packs/`, or `root`
/// itself (sample packs).
fn pack_dirs(root: &Path) -> Vec<PathBuf> {
    let behavior = root.join("behavior_packs");
    if !behavior.is_dir() {
        return vec![root.to_path_buf()];
    }
    let mut dirs = vec![behavior];
    let resource = root.join("resource_packs");
    if resource.is_dir() {
        dirs.push(resource);
    }
    dirs
}

/// Loads the shapes of `dir`, prints the counts and the warnings by code, and requires no
/// error (an MCB document the loader could not decode would be one); returns how many shapes
/// loaded and how many MCB documents are on disk.
fn check(source: &str, dir: &Path) -> (usize, usize) {
    let set = stacks(dir);
    let shapes = set.behavior.voxel_shapes();
    let on_disk: usize = pack_dirs(dir).iter().map(|dir| mcb(dir)).sum();
    let mut counted = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    // The registry's own findings and the loader's about a `shapes/` file (an MCB document
    // never reaches the parser: the loader reports it, on the stack).
    let report = set.behavior.report();
    let about_shapes = report.iter().filter(|finding| {
        finding
            .location
            .path
            .as_ref()
            .is_some_and(|path| path.as_str().starts_with("shapes/"))
    });
    for finding in about_shapes {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.name()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&codes::UNSUPPORTED_FILE_FORMAT) {
            counted += 1;
        }
        errors.push(finding.to_string());
    }
    eprintln!(
        "{source}: {} voxel shapes, {} rejected files, {counted} MCB documents refused ({on_disk} on disk), {} errors, warnings {warnings:?}",
        shapes.len(),
        shapes.rejected().len(),
        errors.len()
    );
    assert!(errors.is_empty(), "{source}: {errors:#?}");
    // No tree has a warning about a `shapes/` file: the pin is exact.
    game_files::assert_warnings(source, &warnings, &[]);
    // Every vanilla shape is `minecraft:<file stem>`, 1 to 7 boxes within the block.
    for (id, shape) in shapes {
        let stem = shape
            .origin()
            .path
            .as_str()
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".json"))
            .unwrap_or_default()
            .to_owned();
        assert_eq!(id.as_str(), format!("minecraft:{stem}"), "{source}");
        assert!(
            (1..=7).contains(&shape.shape.boxes.len()),
            "{source}: {id:?}"
        );
    }
    (shapes.len(), on_disk)
}

#[test]
fn sample_packs_shapes_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let (loaded, mcb) = check("sample packs", &samples);
    assert_eq!((loaded, mcb), (218, 0));
}

#[test]
fn server_install_shapes_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    // Every shape of the install is an MCB document in the brarchive, which shadows the loose
    // JSON, and is decoded.
    assert_eq!(check("server install", &server), (57, 57));
}

#[test]
fn client_shapes_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    assert_eq!(check("client install B", &install_b), (218, 218));
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    assert_eq!(check("client install A", &install_a), (28, 28));
}

/// Parses every loose `shapes/*.json` below the packs of `root` on its own, as a base-game
/// pack's file, and requires no finding at all and an equal value after a write and a re-read;
/// returns how many there are. In the server and client packs the stack reads the baked copy of each
/// (the MCB documents counted above), so this is where their JSON is read.
fn loose_files_load(source: &str, root: &Path) -> usize {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let mut count = 0;
    for dir in pack_dirs(root) {
        for pack in pack_roots(&dir) {
            let Ok(entries) = std::fs::read_dir(pack.join("shapes")) else {
                continue;
            };
            let mut files: Vec<PathBuf> =
                entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            files.sort();
            for file in files {
                let bytes = std::fs::read(&file).unwrap();
                if game_files::is_mcb(&bytes) || file.extension().is_none_or(|e| e != "json") {
                    continue;
                }
                let name = file.file_name().unwrap().to_string_lossy();
                let path = PackPath::new(&format!("shapes/{name}")).unwrap();
                let shape = VoxelShape::parse(&path, &bytes, &cx)
                    .unwrap_or_else(|rejected| panic!("{source} {}: {rejected}", file.display()));
                assert!(
                    shape.diagnostics().is_empty(),
                    "{source} {}",
                    file.display()
                );
                let written = shape.to_json().unwrap();
                let again = VoxelShape::parse(&path, written.as_bytes(), &cx).unwrap();
                assert_eq!(again.value(), shape.value(), "{source} {}", file.display());
                count += 1;
            }
        }
    }
    eprintln!("{source}: {count} loose voxel shape files parse without a finding");
    count
}

#[test]
fn the_loose_shape_files_of_server_install_and_client_parse_without_a_finding() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_eq!(loose_files_load("server install", &server), 57);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    assert_eq!(loose_files_load("client install B", &install_b), 218);
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    assert_eq!(loose_files_load("client install A", &install_a), 28);
}

//! Vanilla smoke for cameras and aim assist: the `cameras/` and `aim_assist/`
//! documents of the sample packs' `behavior_pack/`, of the server install's behaviour packs and of
//! the client installs' packs load through the `Driver` into `bp.cameras()` and
//! `bp.aim_assist()` with zero error-level diagnostics. The sample packs ship neither folder,
//! so their test pins the empty groups. A brarchive entry that is an MCB document (a baked
//! document the loaders do not decode) shadows its loose file and is not parsed: such
//! entries are counted from the stack report, not failed, and every loose JSON of an install
//! is read a second time, file by file, so each vanilla document is parsed whatever shadows
//! it. The camera entity documents of the client resource packs (`cameras/<name>.json`) are
//! read the same way.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_camera`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::aim_assist::{AimAssistCategories, AimAssistPreset};
use bedrock_addon::families::camera::{CameraEntity, CameraList, CameraPreset, CameraSpline};
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

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

/// The MCB documents on disk in the `cameras` and `aim_assist` folders and brarchives of the
/// packs below `dirs`.
fn mcb_on_disk(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            ["cameras", "aim_assist"]
                .into_iter()
                .flat_map(move |folder| {
                    [
                        root.join(folder),
                        root.join("__brarchive").join(folder),
                        root.join("__brarchive").join(format!("{folder}.brarchive")),
                    ]
                })
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// Every `.json` file below `dir` (directly in it when `recursive` is off), sorted.
fn json_files(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if recursive {
                json_files(&path, recursive, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
}

/// Counts by name, for the printed line.
fn count(counts: &mut BTreeMap<String, usize>, name: &str) {
    *counts.entry(name.to_owned()).or_default() += 1;
}

/// What a stack loaded.
#[derive(Debug, Default, PartialEq, Eq)]
struct Loaded {
    presets: usize,
    lists: usize,
    splines: usize,
    aim_presets: usize,
    aim_categories: usize,
    mcb: usize,
}

/// Loads the base-game behaviour stack composed from the packs directories `dirs` and checks
/// its camera and aim-assist groups: no error but MCB documents, nothing rejected, and one MCB
/// finding per MCB document on disk.
fn check_stack(source: &str, dirs: &[&Path], world_behavior: &[u8]) -> Loaded {
    let on_disk = mcb_on_disk(dirs);
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap();
    let cameras = stacks.behavior.cameras();
    let aim = stacks.behavior.aim_assist();
    // The stack report has the collections' findings and the loader's own about their files
    // (an MCB document is `UnsupportedFileFormat` there and never reaches the parser).
    let report = stacks.behavior.report();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings = BTreeMap::new();
    for finding in report.iter() {
        let ours = finding.location.path.as_ref().is_some_and(|path| {
            path.as_str().starts_with("cameras/") || path.as_str().starts_with("aim_assist/")
        });
        if !ours {
            continue;
        }
        if finding.severity == Severity::Warning {
            count(&mut warnings, finding.code.name());
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) {
            mcb += 1;
            continue;
        }
        if finding.severity >= Severity::Error {
            errors.push(finding.to_string());
        }
    }
    let loaded = Loaded {
        presets: cameras.presets.len(),
        lists: cameras.lists.len(),
        splines: cameras.splines.len(),
        aim_presets: aim.presets.len(),
        aim_categories: aim.categories.len(),
        mcb,
    };
    eprintln!(
        "{source}: {loaded:?} over {} packs ({on_disk} MCB documents on disk), {} errors, warnings {warnings:?}",
        stacks.behavior.packs().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No tree has a camera or aim-assist warning in its stack: the pin is exact, so a game
    // file that gains one fails the smoke.
    game_files::assert_warnings(source, &warnings, &[]);
    let rejected = cameras.presets.rejected().len()
        + cameras.lists.rejected().len()
        + cameras.splines.rejected().len()
        + aim.presets.rejected().len()
        + aim.categories.rejected().len();
    assert_eq!(rejected, 0, "{source}: a document was rejected");
    assert_eq!(
        mcb, on_disk,
        "{source}: every MCB document on disk is one finding of the stack"
    );
    loaded
}

/// Parses every loose JSON of `folder` in the packs below `dir` as a base-game file of `T`,
/// one by one: zero errors, and what was read is written back and reads the same. Returns the
/// number of files.
fn check_files<T: Content + PartialEq + std::fmt::Debug>(
    source: &str,
    dir: &Path,
    folder: &str,
    recursive: bool,
) -> usize {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let mut files = 0;
    let mut warnings = BTreeMap::new();
    let mut errors = Vec::new();
    for root in pack_roots(dir) {
        let mut found = Vec::new();
        json_files(&root.join(folder), recursive, &mut found);
        for file in found {
            let relative = file.strip_prefix(&root).unwrap().to_string_lossy();
            let path = PackPath::new(&relative.replace('\\', "/")).unwrap();
            let bytes = std::fs::read(&file).unwrap();
            files += 1;
            match T::parse(&path, &bytes, &cx) {
                Ok(document) => {
                    for finding in document.diagnostics() {
                        if finding.severity == Severity::Warning {
                            count(&mut warnings, finding.code.name());
                        }
                    }
                    let written = document.to_json().unwrap();
                    let again = T::parse(&path, written.as_bytes(), &cx)
                        .unwrap_or_else(|r| panic!("{}: {:#?}", file.display(), r.diagnostics));
                    assert_eq!(again.value(), document.value(), "{}", file.display());
                    assert_eq!(again.to_json().unwrap(), written, "{}", file.display());
                }
                Err(rejected) => {
                    for finding in &rejected.diagnostics {
                        errors.push(format!("{}: {finding}", file.display()));
                    }
                }
            }
        }
    }
    eprintln!(
        "{source}: {files} loose {} files in {folder}/, {} errors, warnings {warnings:?}",
        T::NAME,
        errors.len()
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No loose file of a tree has a warning: the pin is exact, so a game file that gains one
    // fails the smoke.
    game_files::assert_warnings(source, &warnings, &[]);
    files
}

/// The loose files of the five folders of the behaviour packs below `dir`:
/// `(presets, lists, splines, aim presets, aim category files)`.
fn check_behavior_files(source: &str, dir: &Path) -> (usize, usize, usize, usize, usize) {
    (
        check_files::<CameraPreset>(source, dir, "cameras/presets", true),
        check_files::<CameraList>(source, dir, "cameras/lists", true),
        check_files::<CameraSpline>(source, dir, "cameras/splines", true),
        check_files::<AimAssistPreset>(source, dir, "aim_assist/presets", false),
        check_files::<AimAssistCategories>(source, dir, "aim_assist/categories", false),
    )
}

#[test]
fn sample_packs_has_no_camera_or_aim_assist_documents() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    let loaded = check_stack("sample packs", &[&samples], world);
    // The sample packs ship no `cameras/` and no `aim_assist/` folder.
    assert_eq!(loaded, Loaded::default());
    assert_eq!(
        check_behavior_files("sample packs", &samples),
        (0, 0, 0, 0, 0)
    );
}

#[test]
fn server_install_cameras_and_aim_assist_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let dir = server.join("behavior_packs");
    let loaded = check_stack("server install", &[&dir], b"[]");
    let files = check_behavior_files("server install", &dir);
    // 6 presets of `vanilla`, 1 of `experimental_creator_cameras`; one aim-assist preset and
    // one categories file.
    assert_eq!(files, (7, 0, 0, 1, 1));
    // Without the experiment the stack has the vanilla pack's six presets; the three
    // categories are those of its one file. The brarchive entries are JSON: no MCB document.
    assert_eq!(
        loaded,
        Loaded {
            presets: 6,
            aim_presets: 1,
            aim_categories: 3,
            ..Loaded::default()
        }
    );
}

#[test]
fn client_install_b_cameras_and_aim_assist_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dir = client.join("behavior_packs");
    let loaded = check_stack("client install B", &[&dir], b"[]");
    assert_eq!(
        check_behavior_files("client install B", &dir),
        (7, 0, 0, 1, 1)
    );
    assert_eq!(
        (loaded.presets, loaded.aim_presets, loaded.aim_categories),
        (6, 1, 3)
    );
    assert_eq!(loaded.mcb, 0);
}

#[test]
fn client_install_a_cameras_and_aim_assist_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dir = client.join("behavior_packs");
    let loaded = check_stack("client install A", &[&dir], b"[]");
    let files = check_behavior_files("client install A", &dir);
    assert_eq!(files, (7, 0, 0, 1, 1));
    assert_eq!(
        (loaded.presets, loaded.aim_presets, loaded.aim_categories),
        (6, 1, 3)
    );
    assert_eq!(loaded.mcb, 0);
}

#[test]
fn client_camera_entity_documents_parse_without_errors() {
    // On the client installs: the seven `resource_packs/vanilla/cameras/<name>.json` of each
    // install that has them as loose files.
    let mut checked = 0;
    for name in [game_files::CLIENT_INSTALL_A, game_files::CLIENT_INSTALL_B] {
        let Some(client) = game_files::source(name) else {
            eprintln!("skipped: {}", game_files::missing(name));
            continue;
        };
        let packs = client.join("resource_packs");
        let mut loose = Vec::new();
        json_files(&packs.join("vanilla").join("cameras"), false, &mut loose);
        if loose.is_empty() {
            eprintln!("skipped: {name} has no loose resource_packs/vanilla/cameras/*.json");
            continue;
        }
        assert_eq!(
            check_files::<CameraEntity>(name, &packs, "cameras", false),
            7,
            "{name}"
        );
        checked += 1;
    }
    if checked == 0 {
        eprintln!("skipped: no client install with loose camera entity documents");
    }
}

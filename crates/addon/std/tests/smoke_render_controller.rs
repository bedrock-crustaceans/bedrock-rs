//! Render controller smoke over the game's own files: every `render_controllers/` file of the
//! resource packs of the sample packs, the server install and the client installs loads
//! through `Driver` and `rp.render_controllers()` with no error-level diagnostic, with the
//! Molang compile check on (MCB documents, baked documents the loaders do not decode, are
//! counted apart), and every render controller occurrence of the Molang pairs file loads with
//! its text and version.
//!
//! Each tree is loaded as one stack: the install's packs registered as the app's own, the
//! world's lists built by `common::world::base_game_world`, so every pack of the tree is on the
//! resource stack and every file is read; a controller a higher pack replaces is counted by its
//! `RenderControllerShadowed` info. The counts are pinned for the game's files.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_render_controller`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
// The pairs loop is written out below (key occurrences, folded pointers); the shared helper
// gives the member walk and the error list.
#[allow(dead_code)]
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

const DIRECTORY: &str = "render_controllers";

/// A driver that opens packs as the app's own.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The `.json` files below `dir`, sub-folders included.
fn json_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .map(|entry| entry.unwrap().path())
        .map(|path| {
            if path.is_dir() {
                json_files(&path)
            } else {
                usize::from(path.extension().is_some_and(|e| e == "json"))
            }
        })
        .sum()
}

/// What one tree reported.
#[derive(Debug, Default)]
struct Tally {
    /// Packs with the directory.
    packs: usize,
    /// `.json` files in those directories.
    files: usize,
    /// Controllers the registry holds (one per name).
    controllers: usize,
    /// Controllers a higher pack, or a later file of their pack, replaced.
    shadowed: usize,
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
    infos: BTreeMap<&'static str, usize>,
}

/// Loads the resource stack of `repository` (every pack of it, by the shared world list) and
/// requires no error; `roots` are the pack folders, for the counts on disk.
fn assert_loads(
    source: &str,
    driver: &Driver,
    repository: &PackRepository,
    roots: &[PathBuf],
) -> Tally {
    let mut tally = Tally::default();
    let mut on_disk = 0;
    for dir in roots {
        if !dir.join("manifest.json").is_file() || !dir.join(DIRECTORY).is_dir() {
            continue;
        }
        tally.packs += 1;
        tally.files += json_files(&dir.join(DIRECTORY));
        on_disk += game_files::mcb_on_disk(&dir.join(DIRECTORY)).total()
            + game_files::mcb_on_disk(
                &dir.join("__brarchive")
                    .join(format!("{DIRECTORY}.brarchive")),
            )
            .total();
    }
    let options = LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        ..LoadOptions::default()
    };
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let registry = stacks.resource.render_controllers();
    tally.controllers = registry.len();
    let mut unsupported = Vec::new();
    for diagnostic in registry.diagnostics() {
        if diagnostic.code.id() == "core/unsupported_file_format" {
            unsupported.push(diagnostic.to_string());
        } else if diagnostic.severity >= Severity::Error {
            tally.errors.push(diagnostic.to_string());
        } else if diagnostic.severity == Severity::Warning {
            *tally.warnings.entry(diagnostic.code.id()).or_insert(0) += 1;
        } else {
            *tally.infos.entry(diagnostic.code.id()).or_insert(0) += 1;
        }
    }
    tally.shadowed = tally
        .infos
        .get("core/render_controller_shadowed")
        .copied()
        .unwrap_or(0);
    tally.mcb = unsupported.len();
    if unsupported.len() > on_disk {
        tally.errors.extend(unsupported);
    }
    eprintln!(
        "{source}: {} packs, {} files, {} controllers ({} more shadowed), {} errors, {} MCB documents, warnings {:?}, infos {:?}",
        tally.packs,
        tally.files,
        tally.controllers,
        tally.shadowed,
        tally.errors.len(),
        tally.mcb,
        tally.warnings,
        tally.infos
    );
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert!(registry.rejected().is_empty(), "{source}: rejected files");
    tally
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

/// The tree of an install: its behaviour and resource packs as one repository.
fn install(root: &Path, source: &str) -> Tally {
    let mut driver = base_game_driver();
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    assert_loads(
        source,
        &driver,
        &repository,
        &packs(&root.join("resource_packs")),
    )
}

/// The vanilla resource pack of the sample packs loads without an error or a warning (the
/// pinned counts are the sample packs' own).
#[test]
fn sample_packs_resource_pack_loads() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack = driver
        .pack_dir(samples.join("resource_pack"))
        .expect("the resource pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let tally = assert_loads(
        "sample packs resource_pack",
        &driver,
        &repository,
        &[samples.join("resource_pack")],
    );
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    assert_eq!(
        (tally.packs, tally.files, tally.controllers, tally.shadowed),
        (1, 173, 213, 0)
    );
}

#[test]
fn server_install_resource_packs_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    let tally = install(&server, "server install resource_packs");
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    assert_eq!(
        (tally.packs, tally.files, tally.controllers, tally.shadowed),
        SERVER_INSTALL_COUNTS
    );
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = install(&client, "client install B resource_packs");
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    assert_eq!(
        (tally.packs, tally.files, tally.controllers, tally.shadowed),
        CLIENT_INSTALL_B_COUNTS
    );
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let tally = install(&client, "client install A resource_packs");
    assert!(tally.warnings.is_empty(), "{:?}", tally.warnings);
    assert_eq!(
        (tally.packs, tally.files, tally.controllers, tally.shadowed),
        CLIENT_INSTALL_A_COUNTS
    );
}

/// Packs with the directory, files, controllers by name, controllers replaced.
const SERVER_INSTALL_COUNTS: (usize, usize, usize, usize) = (1, 1, 1, 0);
const CLIENT_INSTALL_B_COUNTS: (usize, usize, usize, usize) = (31, 211, 216, 80);
const CLIENT_INSTALL_A_COUNTS: (usize, usize, usize, usize) = (32, 212, 217, 76);

/// The text of a JSON pointer segment (`~1` is `/`, `~0` is `~`).
fn segment(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Every string leaf of `value` with its pointer as the file writes it, and the pointer the
/// loaded file gives the same member: lower case (the game folds the file), and for
/// `materials` / `part_visibility` the pair's position in the game's order (elements in array
/// order, the keys of one element sorted) in place of the element's index.
fn leaves(
    value: &bedrock_addon::json::Value,
    written: &str,
    loaded: &str,
    out: &mut Vec<(String, String, String)>,
) {
    if let Some(text) = value.as_string() {
        out.push((
            written.to_owned(),
            loaded.to_owned(),
            text.as_str().to_owned(),
        ));
    } else if let Some(array) = value.as_array() {
        let bones = loaded.ends_with("/materials") || loaded.ends_with("/part_visibility");
        let is_controller_member = loaded.matches('/').count() == 3;
        let mut position = 0_usize;
        for (index, element) in array.iter().enumerate() {
            let written = format!("{written}/{index}");
            if let (true, true, Some(object)) = (bones, is_controller_member, element.as_object()) {
                let mut keys: Vec<&str> = object.iter().map(|(key, _)| key.as_str()).collect();
                keys.sort_by_key(|k| k.to_ascii_lowercase().into_bytes());
                for key in keys {
                    let member = object.get(key).unwrap();
                    leaves(
                        member,
                        &format!("{written}/{}", segment(key)),
                        &format!("{loaded}/{position}/{}", segment(&key.to_ascii_lowercase())),
                        out,
                    );
                    position += 1;
                }
            } else {
                leaves(element, &written, &format!("{loaded}/{index}"), out);
            }
        }
    } else if let Some(object) = value.as_object() {
        for (key, member) in object {
            leaves(
                member,
                &format!("{written}/{}", segment(key.as_str())),
                &format!("{loaded}/{}", segment(&key.as_str().to_ascii_lowercase())),
                out,
            );
        }
    }
}

/// Every render controller occurrence of the Molang pairs file (the packs of client install B) loads:
/// a value with the pair's text, compared as the game reads it (case folded outside string
/// literals), and the pack's Molang version; a key (the name of an array) as a name the loaded
/// controller declares.
///
/// The shared check compares pointers lower-cased and texts case-folded; this family
/// hands it each member at the pointer the file writes it at (the loaded file orders the
/// pairs of `materials` / `part_visibility`), and each declared array name as a key.
#[test]
fn every_render_controller_molang_occurrence_loads_with_its_text_and_version() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::render_controller::{RenderControllerFile, fold_case};
    use bedrock_addon::json::{JsonSlice, Value, parse_at};

    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let rules = pairs::Rules {
        keys: pairs::Keys::Members,
        pointer: |pointer| pointer.to_ascii_lowercase(),
        text: fold_case,
        foreign: |_| false,
    };
    let found = pairs::check_with_keys(&pairs_path, &install_b, DIRECTORY, &rules, |file| {
        let document = RenderControllerFile::parse(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = document.diagnostics().clone();
        let mut loaded = document.into_value();
        let mut members = pairs::Members::new();
        // The key of an array is its name; the loaded controller declares it, folded.
        for (key, controller) in &loaded.render_controllers {
            let Some(declared) = &controller.arrays else {
                continue;
            };
            for (kind, map) in [
                ("geometries", &declared.geometries),
                ("materials", &declared.materials),
                ("textures", &declared.textures),
            ] {
                for name in map.keys() {
                    members.insert(
                        format!("/render_controllers/{key}/arrays/{kind}/{name}"),
                        (name.clone(), None),
                    );
                }
            }
        }
        // By lower-cased pointer: an apostrophe can leave capitals in the loaded file.
        let mut by_loaded = pairs::Members::new();
        pairs::members(&mut loaded, None, &mut by_loaded);
        let by_loaded: pairs::Members = by_loaded
            .into_iter()
            .map(|(pointer, member)| (pointer.to_ascii_lowercase(), member))
            .collect();
        let root: Value = parse_at(JsonSlice::new(file.text)).expect("JSON");
        let mut leaves_of = Vec::new();
        leaves(&root, "", "", &mut leaves_of);
        for (written_at, loaded_at, written) in leaves_of {
            // The file's text and the loaded text agree as the game reads them.
            if let Some((source, raw)) = by_loaded.get(&loaded_at)
                && fold_case(source) == fold_case(&written)
            {
                members.insert(written_at, (written, *raw));
            }
        }
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs render_controllers: {} values and {} keys in {} files, {} failures",
        found.compared,
        found.keys,
        found.files,
        found.failures.len()
    );
    assert!(
        found.failures.is_empty(),
        "{} failures, first: {:#?}",
        found.failures.len(),
        found.failures.iter().take(10).collect::<Vec<_>>()
    );
    // The pairs of client install B: 2745 occurrences (2598 values, 147 array names) in 211 files.
    assert_eq!((found.compared, found.keys, found.files), (2598, 147, 211));
}

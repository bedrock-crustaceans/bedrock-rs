//! Material smoke over the game's own files: the `materials/` folders of the resource packs
//! of the client installs (`vanilla`, `vanilla_base`, `chemistry`) and of the server install load
//! through `Driver` and `rp.materials()` with no error-level diagnostic. The sample packs ship
//! no `materials/` folder: that tree is skipped with a message. The compiled
//! `renderer/materials/*.material.bin` files are no file of this family.
//!
//! Each tree is loaded as one stack: the install's packs registered as the app's own, the
//! world's lists built by `common::world::base_game_world` (which orders the resource side as
//! the client's base-game stack: `vanilla_base` below `vanilla` below `chemistry`), so every
//! pack of the tree is on the resource stack. Material files hold no Molang, so
//! the Molang pairs file has no occurrence to check. The counts are pinned for the game's files.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_materials`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::material::{
    MaterialFile, MaterialGroupKind, MaterialList, MaterialListKind,
};
use bedrock_addon::id::MaterialId;
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

const DIRECTORY: &str = "materials";

/// A driver that opens packs as the app's own.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The files directly in `dir` whose name ends with `suffix`.
fn files_ending(dir: &Path, suffix: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .filter(|path| path.to_string_lossy().ends_with(suffix))
        .count()
}

/// What one tree reported.
#[derive(Debug, Default, PartialEq)]
struct Tally {
    /// Packs with the directory.
    packs: usize,
    /// `.material` files in those directories.
    material_files: usize,
    /// The four list files in those directories.
    list_files: usize,
    /// List documents the load holds.
    lists: usize,
    /// Merged material files (one per path).
    merged: usize,
    /// Materials of the common, the fancy and the sad group, variants not counted.
    groups: [usize; 3],
    /// Names of the three groups, variants counted.
    names: [usize; 3],
    errors: Vec<String>,
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
    for dir in roots {
        if !dir.join("manifest.json").is_file() || !dir.join(DIRECTORY).is_dir() {
            continue;
        }
        tally.packs += 1;
        tally.material_files += files_ending(&dir.join(DIRECTORY), ".material");
        tally.list_files += MaterialListKind::ALL
            .iter()
            .filter(|kind| dir.join(kind.path()).is_file())
            .count();
    }
    let options = LoadOptions::default();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let materials = stacks.resource.materials();
    tally.lists = materials.lists.len();
    tally.merged = materials.files.len();
    for (slot, kind) in MaterialGroupKind::ALL.into_iter().enumerate() {
        tally.groups[slot] = materials.group(kind).len();
        tally.names[slot] = materials.group(kind).names().count();
    }
    for diagnostic in materials.diagnostics() {
        if diagnostic.severity >= Severity::Error {
            tally.errors.push(diagnostic.to_string());
        } else if diagnostic.severity == Severity::Warning {
            *tally.warnings.entry(diagnostic.code.id()).or_insert(0) += 1;
        } else {
            *tally.infos.entry(diagnostic.code.id()).or_insert(0) += 1;
        }
    }
    eprintln!(
        "{source}: {} packs, {} .material files, {} list files; {} lists, {} merged files, materials {:?} (names {:?}) in common/fancy/sad, {} errors, warnings {:?}, infos {:?}",
        tally.packs,
        tally.material_files,
        tally.list_files,
        tally.lists,
        tally.merged,
        tally.groups,
        tally.names,
        tally.errors.len(),
        tally.warnings,
        tally.infos
    );
    for diagnostic in materials.diagnostics() {
        eprintln!("  {diagnostic}");
    }
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    // The names every client entity of the vanilla packs is drawn with.
    if tally.list_files > 0 {
        for name in [
            "entity",
            "entity_alphatest",
            "entity.skinning",
            "entity_static.skinning_color",
        ] {
            assert!(
                materials.contains(&MaterialId::parse(name)),
                "{source}: {name}"
            );
        }
    }
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

#[test]
fn sample_packs_has_no_materials_folder() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The sample packs ship no `materials/` folder; nothing to load.
    assert!(!samples.join("resource_pack").join(DIRECTORY).exists());
    eprintln!("skipped: the sample packs ship no {DIRECTORY}/ folder");
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = install(&client, "client install B resource_packs");
    assert_eq!(tally, expected_client([99, 274, 252], [131, 981, 959]));
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let tally = install(&client, "client install A resource_packs");
    // Client install A's entity file has one material less.
    assert_eq!(tally, expected_client([99, 273, 251], [131, 977, 955]));
}

#[test]
fn server_install_resource_packs_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    let tally = install(&server, "server install resource_packs");
    // The server ships the chemistry pack's one file and no list: it is merged, and creates
    // no material.
    assert_eq!(
        (tally.packs, tally.material_files, tally.list_files),
        (1, 1, 0)
    );
    assert_eq!((tally.lists, tally.merged), (0, 1));
    assert_eq!(tally.groups, [0, 0, 0]);
    assert!(
        tally.warnings.is_empty() && tally.infos.is_empty(),
        "{tally:?}"
    );
}

/// What a client install tree gives: three packs, 19 `.material` files of 14 different paths
/// and 8 lists. The two kinds of finding are vanilla's own: `vanilla_base` lists
/// `terrain.material` three times in `fancy.json` and in `sad.json` (two infos each for the
/// repeated path), and `terrain_blend` adds two variants it inherits (two infos, the client's
/// engine log).
fn expected_client(groups: [usize; 3], names: [usize; 3]) -> Tally {
    Tally {
        packs: 3,
        material_files: 19,
        list_files: 8,
        lists: 8,
        merged: 14,
        groups,
        names,
        errors: Vec::new(),
        warnings: BTreeMap::new(),
        infos: BTreeMap::from([("core/no_effect", 4), ("material/variant_replaced", 2)]),
    }
}

/// A JSON value in a form that compares member for member: object members sorted by key,
/// numbers by their value.
fn canonical(value: &Value, out: &mut String) {
    use std::fmt::Write;
    if let Some(object) = value.as_object() {
        let mut members: Vec<_> = object.iter().collect();
        members.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        out.push('{');
        for (key, member) in members {
            let _ = write!(out, "{:?}:", key.as_str());
            canonical(member, out);
            out.push(',');
        }
        out.push('}');
    } else if let Some(array) = value.as_array() {
        out.push('[');
        for element in array.as_slice() {
            canonical(element, out);
            out.push(',');
        }
        out.push(']');
    } else if let Some(number) = value.as_number() {
        let _ = write!(out, "{}", number.to_f64_lossy());
    } else if let Some(text) = value.as_string() {
        let _ = write!(out, "{:?}", text.as_str());
    } else if let Some(flag) = value.as_bool() {
        let _ = write!(out, "{flag}");
    } else {
        out.push_str("null");
    }
}

/// Reads every `.material` file and every list file of the packs of `root` on its own, writes
/// it and requires the written text to be the file's own JSON value, member for member:
/// vanilla's files are written back as authored. Gives (material files, list files).
fn assert_written_as_authored(root: &Path, source: &str) -> (usize, usize) {
    let (mut files, mut lists) = (0, 0);
    for pack in packs(&root.join("resource_packs")) {
        let Ok(entries) = std::fs::read_dir(pack.join(DIRECTORY)) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries.map(|entry| entry.unwrap().path()).collect();
        paths.sort();
        for path in paths {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let key = PackPath::new(&format!("{DIRECTORY}/{name}")).expect("a pack path");
            let is_list = MaterialListKind::ALL
                .iter()
                .any(|kind| kind.path() == key.as_str());
            if !is_list && !name.ends_with(".material") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            let context = LoadContext::detached();
            let written = if is_list {
                lists += 1;
                let list = MaterialList::parse(&key, &bytes, &context)
                    .unwrap_or_else(|rejected| panic!("{}: {rejected:?}", path.display()));
                list.to_json()
            } else {
                files += 1;
                let file = MaterialFile::parse(&key, &bytes, &context)
                    .unwrap_or_else(|rejected| panic!("{}: {rejected:?}", path.display()));
                file.to_json()
            };
            let written = written.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let text = String::from_utf8(bytes).expect("UTF-8");
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
            let input = bedrock_addon::json::from_str_jsonc::<Value>(text).expect("the input");
            let output = bedrock_addon::json::from_str::<Value>(&written).expect("the output");
            let (mut before, mut after) = (String::new(), String::new());
            canonical(&input, &mut before);
            canonical(&output, &mut after);
            assert_eq!(before, after, "{}: not written as authored", path.display());
        }
    }
    eprintln!("{source}: {files} .material files and {lists} list files written as authored");
    (files, lists)
}

#[test]
fn every_vanilla_material_file_and_list_is_written_back_as_authored() {
    // The counts are the trees' own; client install B's `ui.material` files write
    // `"stencilRefOverride": 0`, which is written back as the number.
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    assert_eq!(
        assert_written_as_authored(&install_b, "client install B"),
        (19, 8)
    );
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    assert_eq!(
        assert_written_as_authored(&install_a, "client install A"),
        (19, 8)
    );
    let server = skip!(game_files::SERVER_INSTALL);
    assert_eq!(
        assert_written_as_authored(&server, "server install"),
        (1, 0)
    );
}

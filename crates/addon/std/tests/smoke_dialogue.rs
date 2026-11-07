//! Dialogue smoke over the game's own files: the NPC dialogue files of
//! the sample packs' `behavior_pack/` and of the server install's behaviour packs load through the
//! `Driver` into `bp.dialogue()` with zero error-level diagnostics. Vanilla ships no dialogue
//! file today (neither tree has a `dialogue/` folder), so the tests also count what is on disk
//! and require the registry to agree with it; MCB documents (baked documents the
//! loaders do not decode) are counted, not failed. One more test needs no game files: two packs
//! written to a temporary directory load through the `Driver` from disk, the lower-priority
//! pack's scene winning.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_dialogue`;
//! without the variable the game-file tests skip.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::dialogue::DialogueText;
use bedrock_addon::id::DialogueSceneTag;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
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

/// The `.json` files below `dir`, at any depth.
fn json_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .map(|path| {
            if path.is_dir() {
                json_files(&path)
            } else {
                usize::from(path.extension().is_some_and(|e| e == "json"))
            }
        })
        .sum()
}

/// The dialogue files on disk in the packs below `dirs`: loose `.json` files and MCB documents
/// (`dialogue/` and a `dialogue` brarchive).
fn on_disk(dirs: &[&Path]) -> (usize, usize) {
    let roots: Vec<PathBuf> = dirs.iter().flat_map(|dir| pack_roots(dir)).collect();
    let loose = roots
        .iter()
        .map(|root| json_files(&root.join("dialogue")))
        .sum();
    let mcb = roots
        .iter()
        .flat_map(|root| {
            [
                root.join("dialogue"),
                root.join("__brarchive").join("dialogue"),
                root.join("__brarchive").join("dialogue.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum();
    (loose, mcb)
}

/// The stacks of the packs directories `dirs` (of `origin`) for the world's behaviour list.
fn stacks(dirs: &[&Path], origin: PackOrigin, world_behavior: &[u8]) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), origin)),
        )
        .unwrap();
    driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap()
}

/// Loads the dialogue of `dirs`: how many scenes, warnings by code, and no error but MCB
/// documents.
fn check(source: &str, dirs: &[&Path], world_behavior: &[u8]) {
    let (loose, mcb_on_disk) = on_disk(dirs);
    let set = stacks(dirs, PackOrigin::Package, world_behavior);
    let dialogue = set.behavior.dialogue();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in dialogue.diagnostics() {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.name()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && mcb < mcb_on_disk {
            mcb += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    eprintln!(
        "{source}: {} scenes over {} packs ({loose} dialogue files on disk), {} rejected, {mcb} MCB documents ({mcb_on_disk} on disk), {} errors, warnings {warnings:?}",
        dialogue.len(),
        set.behavior.packs().len(),
        dialogue.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // Neither tree has a dialogue warning: the pin is exact, so a game file that gains one
    // fails the smoke.
    game_files::assert_warnings(source, &warnings, &[]);
    assert_eq!(
        dialogue.rejected().len(),
        mcb,
        "{source}: only MCB documents are rejected"
    );
    // Every file on disk gives at least one scene; none on disk, none loaded.
    assert_eq!(
        dialogue.is_empty(),
        loose == 0,
        "{source}: scenes loaded and files on disk disagree"
    );
}

#[test]
fn sample_packs_dialogue_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The sample pack has its own UUID: the world selects it from the vanilla location.
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    check("sample packs", &[&samples], world);
}

#[test]
fn server_install_dialogue_loads_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    check("server install", &[&server.join("behavior_packs")], b"[]");
}

/// A behaviour pack folder `name` below `root` with one dialogue file.
fn write_pack(root: &Path, name: &str, uuid: u8, file: &str, text: &str) {
    let pack = root.join(name);
    std::fs::create_dir_all(pack.join("dialogue").join("npcs")).unwrap();
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "00000000-0000-0000-0000-0000000000{uuid:02x}", "version": [1, 0, 0], "min_engine_version": [1, 19, 0]}}, "modules": [{{"type": "data", "uuid": "00000000-0000-0000-0001-0000000000{uuid:02x}", "version": [1, 0, 0]}}]}}"#
    );
    std::fs::write(pack.join("manifest.json"), manifest).unwrap();
    std::fs::write(pack.join("dialogue").join(file), text).unwrap();
}

#[test]
fn two_packs_on_disk_load_through_the_driver_and_the_lower_one_keeps_a_shared_scene() {
    let root = tempfile::tempdir().unwrap();
    let scene = |text: &str, own: &str| {
        format!(
            r#"{{"format_version": "1.17.0", "minecraft:npc_dialogue": {{"scenes": [
                {{"scene_tag": "shared", "text": "{text}", "on_open_commands": ["/say {text}"]}},
                {{"scene_tag": "{own}", "text": "{text}"}}]}}}}"#
        )
    };
    write_pack(
        root.path(),
        "first",
        1,
        "npcs/a.json",
        &scene("first", "only_first"),
    );
    write_pack(
        root.path(),
        "second",
        2,
        "b.json",
        &scene("second", "only_second"),
    );
    let world = br#"[
        {"pack_id": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0]},
        {"pack_id": "00000000-0000-0000-0000-000000000002", "version": [1, 0, 0]}]"#;
    let set = stacks(&[root.path()], PackOrigin::User, world);
    assert_eq!(set.behavior.packs().len(), 2);
    let dialogue = set.behavior.dialogue();
    assert_eq!(dialogue.len(), 3);
    let shared = dialogue
        .get(&DialogueSceneTag::parse("shared"))
        .expect("the shared scene");
    // The lowest-priority pack (the last of the stack) is visited first and keeps the tag.
    assert_eq!(shared.origin().pack.index(), Some(1));
    let kept = shared.origin().pack.name().to_owned();
    assert_eq!(shared.text, DialogueText::Plain(kept.clone()));
    // 1.19.0 → command version 20.
    assert_eq!(shared.on_open_commands.as_ref().unwrap()[0].version, 20);
    let errors: Vec<_> = dialogue
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].code,
        DiagnosticCode::DuplicateIdentifier { text: Some(_) }
    ));
    assert_eq!(
        errors[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(0)
    );
    let stem = if kept == "first" { "b" } else { "a" };
    assert_eq!(
        errors[0].code.to_string(),
        format!(
            "JSON: {stem}.json has an error: \n A scene with a scene name of 'shared' already exists. Skipping scene..."
        )
    );
}

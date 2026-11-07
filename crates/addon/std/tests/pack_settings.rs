//! `read_saved_settings`: the saved pack-settings files read from disk, and the smoke test over
//! every such file the game's own files hold.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::pack::settings::{
    FileState, GLOBAL_FILE, SavedSettings, SettingValue, SettingsFile, WORLD_FILE,
};
use bedrock_addon_std::{ReadError, read_saved_settings, read_saved_settings_document};

/// The family's fixtures in the `bedrock_addon` crate.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../addon/tests/fixtures/pack/pack_settings")
        .join(name)
}

const DOCUMENT: &str = r#"{
  "format_version": "1.21.100",
  "minecraft:pack_settings": {
    "settings": [
      {
        "pack_id": "d3349771-6782-4420-875a-fcd6179a8915",
        "values": { "banana:slider": 5, "banana:toggle": false, "banana:dropdown": "apple" }
      }
    ]
  }
}"#;

#[test]
fn a_world_file_is_read_and_parsed() {
    let world = tempfile::tempdir().unwrap();
    let path = world.path().join(WORLD_FILE);
    fs::write(&path, DOCUMENT).unwrap();
    let settings = read_saved_settings(&path).unwrap();
    assert_eq!(settings.settings.len(), 1);
    assert_eq!(
        settings.settings[0].values["banana:slider"],
        SettingValue::Float(5.0)
    );
    assert_eq!(
        settings,
        SavedSettings::parse(DOCUMENT.as_bytes())
            .unwrap()
            .into_value()
    );
}

#[test]
fn d1_a_missing_file_holds_no_values() {
    let world = tempfile::tempdir().unwrap();
    for file in [SettingsFile::World, SettingsFile::Global] {
        let settings = read_saved_settings(world.path().join(file.name())).unwrap();
        assert_eq!(settings, SavedSettings::default());
    }
}

#[test]
fn a_file_that_cannot_be_read_is_an_io_error() {
    let world = tempfile::tempdir().unwrap();
    let path = world.path().join(WORLD_FILE);
    fs::create_dir(&path).unwrap();
    match read_saved_settings(&path) {
        Err(ReadError::Io { path: reported, .. }) => assert_eq!(reported, path),
        other => panic!("{other:?}"),
    }
}

#[test]
fn d9_a_rejected_file_closes_with_its_path() {
    let path = fixture("ps09_unknown_member.json");
    let Err(ReadError::Rejected(rejected)) = read_saved_settings(&path) else {
        panic!("the file is rejected");
    };
    let last = rejected
        .diagnostics
        .iter()
        .rfind(|d| d.severity == Severity::Error)
        .unwrap();
    assert_eq!(
        last.code,
        bedrock_addon::pack::settings::codes::LOAD_FAILED.with([path.display().to_string()])
    );
    assert_eq!(rejected.origin.path.as_str(), "ps09_unknown_member.json");
}

#[test]
fn a_saved_document_written_to_disk_reads_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(GLOBAL_FILE);
    let first = read_saved_settings(fixture("ps03_v1_26_40_string_list.json")).unwrap();
    fs::write(&path, first.to_json().unwrap()).unwrap();
    assert_eq!(read_saved_settings(&path).unwrap(), first);
}

#[test]
fn the_state_of_a_file_for_a_save_follows_the_read() {
    let world = tempfile::tempdir().unwrap();
    let path = world.path().join(WORLD_FILE);
    // What a caller builds from the three outcomes of a read.
    let state = |path: &Path| match read_saved_settings(path) {
        Ok(settings) if path.exists() => FileState::Read(settings),
        Ok(_) => FileState::Missing,
        Err(ReadError::Rejected(_)) => FileState::Failed,
        Err(other) => panic!("{other:?}"),
    };
    assert_eq!(state(&path), FileState::Missing);
    fs::write(&path, DOCUMENT).unwrap();
    assert!(matches!(state(&path), FileState::Read(_)));
    fs::write(&path, "{").unwrap();
    assert_eq!(state(&path), FileState::Failed);
}

/// Every file named like a saved-settings file below `dir` (symlinks followed, at most 12
/// levels deep).
fn settings_files(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            if depth < 12 {
                settings_files(&path, depth + 1, found);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == WORLD_FILE || name == GLOBAL_FILE)
        {
            found.push(path);
        }
    }
}

/// The vanilla smoke test: every `world_behavior_pack_settings.json` and
/// `global_resource_pack_settings.json` of the game's own files loads without an error. The
/// files are written by a client into worlds and its settings directory, which the game files
/// (packs and a server install) do not hold, so the count is expected to be 0; the test then
/// only says so.
#[test]
fn smoke_saved_settings_of_the_game_files_load() {
    let mut found = Vec::new();
    let mut searched = 0;
    for entry in [
        common::game_files::SAMPLE_PACKS,
        common::game_files::SERVER_INSTALL,
        common::game_files::CLIENT_INSTALL_A,
        common::game_files::CLIENT_INSTALL_B,
        common::game_files::SPLIT_TREE,
    ] {
        let root = skip!(entry);
        searched += 1;
        settings_files(&root, 0, &mut found);
    }
    found.sort();
    eprintln!(
        "pack_settings smoke: {} saved-settings file(s) in {searched} game-file entries",
        found.len()
    );
    for path in &found {
        match read_saved_settings(path) {
            Ok(settings) => eprintln!("  {}: {} entries", path.display(), settings.settings.len()),
            Err(error) => panic!("{}: {error:?}", path.display()),
        }
    }
}

// ---- read_saved_settings_document: the reader that keeps the findings -------------------------

/// A file that loads with a finding: a slider value beyond `f32`, an information. A number
/// beyond `f32` is read as the largest finite float with an `InvalidValue` note and the file
/// still loads; refusing it would drop every pack's saved values for one number.
const WARNED: &str = r#"{
  "format_version": "1.21.100",
  "minecraft:pack_settings": {
    "settings": [
      {
        "pack_id": "d3349771-6782-4420-875a-fcd6179a8915",
        "values": { "banana:slider": 1e39 }
      }
    ]
  }
}"#;

#[test]
fn the_document_reader_returns_the_findings_of_a_file_that_loads() {
    let world = tempfile::tempdir().unwrap();
    let path = world.path().join(WORLD_FILE);
    fs::write(&path, WARNED).unwrap();
    let document = read_saved_settings_document(&path)
        .unwrap()
        .expect("the file exists");
    let warnings: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .map(|d| (d.code.name(), d.location.json_path.to_string()))
        .collect();
    assert_eq!(
        warnings,
        [(
            "InvalidValue",
            "minecraft:pack_settings/settings/0/values/banana:slider".to_owned()
        )]
    );
    assert_eq!(document.origin().path.as_str(), WORLD_FILE);
    // The value is the one the plain reader gives, which drops the findings.
    assert_eq!(document.value(), &read_saved_settings(&path).unwrap());
    assert_eq!(
        document.value().settings[0].values["banana:slider"],
        SettingValue::Float(f32::MAX)
    );
}

#[test]
fn the_document_reader_tells_a_missing_file_from_an_empty_one() {
    let world = tempfile::tempdir().unwrap();
    for file in [SettingsFile::World, SettingsFile::Global] {
        let path = world.path().join(file.name());
        assert!(read_saved_settings_document(&path).unwrap().is_none());
        // The plain reader's empty document for the same path.
        assert_eq!(
            read_saved_settings(&path).unwrap(),
            SavedSettings::default()
        );
    }
    let path = world.path().join(WORLD_FILE);
    fs::write(&path, DOCUMENT).unwrap();
    let document = read_saved_settings_document(&path).unwrap().unwrap();
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.value().settings.len(), 1);
}

#[test]
fn the_document_reader_fails_as_the_plain_reader_does() {
    let world = tempfile::tempdir().unwrap();
    let path = world.path().join(WORLD_FILE);
    fs::write(&path, "{}").unwrap();
    let Err(ReadError::Rejected(rejected)) = read_saved_settings_document(&path) else {
        panic!("a rejected file");
    };
    let Err(ReadError::Rejected(plain)) = read_saved_settings(&path) else {
        panic!("a rejected file");
    };
    assert_eq!(rejected.diagnostics, plain.diagnostics);
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    match read_saved_settings_document(&path) {
        Err(ReadError::Io { path: reported, .. }) => assert_eq!(reported, path),
        other => panic!("{other:?}"),
    }
}

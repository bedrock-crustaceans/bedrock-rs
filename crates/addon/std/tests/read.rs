//! The readers of server-side files around packs: `read_world_packs` and `read_app_data`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use bedrock_addon::load::AppData;
use bedrock_addon::pack::WorldPacks;
use bedrock_addon_std::{ReadError, read_app_data, read_world_packs};

fn write(dir: &Path, path: &str, bytes: &[u8]) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// Every path and its bytes, in the `AppData`'s own (bytewise) order.
fn contents(app_data: &AppData) -> Vec<(String, Vec<u8>)> {
    app_data
        .iter()
        .map(|(path, bytes)| (path.as_str().to_owned(), bytes.to_vec()))
        .collect()
}

mod world_packs {
    use super::*;

    const BEHAVIOR: &[u8] =
        br#"[{ "pack_id": "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21", "version": [1, 0, 0] }]"#;

    #[test]
    fn both_lists_are_read_and_parsed() {
        let world = tempfile::tempdir().unwrap();
        write(world.path(), WorldPacks::BEHAVIOR_FILE, BEHAVIOR);
        write(world.path(), WorldPacks::RESOURCE_FILE, b"[]");
        let packs = read_world_packs(world.path()).unwrap();
        assert_eq!(packs.behavior.len(), 1);
        assert_eq!(
            packs.behavior[0].pack_id.to_string(),
            "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21"
        );
        assert!(packs.resource.is_empty());
    }

    #[test]
    fn a_world_without_lists_selects_no_packs() {
        let world = tempfile::tempdir().unwrap();
        let packs = read_world_packs(world.path()).unwrap();
        assert!(packs.behavior.is_empty());
        assert!(packs.resource.is_empty());
    }

    #[test]
    fn a_rejected_list_is_a_rejected_error() {
        let world = tempfile::tempdir().unwrap();
        write(world.path(), WorldPacks::BEHAVIOR_FILE, b"{ not json");
        assert!(matches!(
            read_world_packs(world.path()),
            Err(ReadError::Rejected(_))
        ));
    }

    #[test]
    fn a_list_that_cannot_be_read_is_an_io_error_naming_it() {
        let world = tempfile::tempdir().unwrap();
        fs::create_dir(world.path().join(WorldPacks::RESOURCE_FILE)).unwrap();
        match read_world_packs(world.path()) {
            Err(ReadError::Io { path, source }) => {
                assert_eq!(path, world.path().join(WorldPacks::RESOURCE_FILE));
                assert_eq!(source.kind(), ErrorKind::IsADirectory);
            }
            other => panic!("expected an I/O error, got {other:?}"),
        }
    }
}

mod app_data {
    use super::*;

    #[test]
    fn the_four_directories_are_read_at_any_depth() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "definitions/attachables/bow.json", b"bow");
        write(root.path(), "definitions/features/trees/oak.json", b"oak");
        write(
            root.path(),
            "definitions/feature_rules/oak_rule.json",
            b"rule",
        );
        write(
            root.path(),
            "definitions/spawn_groups/raid_group_1.json",
            b"raid",
        );
        write(
            root.path(),
            "definitions/persona/skip.json",
            b"not app data",
        );
        write(root.path(), "server.properties", b"not app data");
        // The credits are one file of the game's own data, not a directory.
        write(root.path(), "credits/credits.json", b"[]");
        write(root.path(), "credits/end.txt", b"not app data");
        let app_data = read_app_data(root.path()).unwrap();
        assert_eq!(
            contents(&app_data),
            vec![
                ("credits/credits.json".to_owned(), b"[]".to_vec()),
                (
                    "definitions/attachables/bow.json".to_owned(),
                    b"bow".to_vec()
                ),
                (
                    "definitions/feature_rules/oak_rule.json".to_owned(),
                    b"rule".to_vec()
                ),
                (
                    "definitions/features/trees/oak.json".to_owned(),
                    b"oak".to_vec()
                ),
                (
                    "definitions/spawn_groups/raid_group_1.json".to_owned(),
                    b"raid".to_vec()
                ),
            ]
        );
    }

    #[test]
    fn a_missing_directory_is_skipped() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "definitions/spawn_groups/raid_group_1.json",
            b"raid",
        );
        let app_data = read_app_data(root.path()).unwrap();
        assert_eq!(app_data.len(), 1);
    }

    #[test]
    fn an_empty_root_is_empty_app_data() {
        let root = tempfile::tempdir().unwrap();
        assert!(read_app_data(root.path()).unwrap().is_empty());
    }

    #[test]
    fn a_missing_root_is_an_io_error() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("no_such_install");
        match read_app_data(&missing) {
            Err(ReadError::Io { path, source }) => {
                assert_eq!(path, missing);
                assert_eq!(source.kind(), ErrorKind::NotFound);
            }
            other => panic!("expected an I/O error, got {other:?}"),
        }
    }

    #[test]
    fn a_root_that_is_a_file_is_an_io_error() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "file", b"");
        assert!(matches!(
            read_app_data(root.path().join("file")),
            Err(ReadError::Io { source, .. }) if source.kind() == ErrorKind::NotADirectory
        ));
    }

    #[test]
    fn a_name_that_is_not_a_pack_path_is_an_invalid_data_error() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "definitions/features/c:oak.json", b"oak");
        assert!(matches!(
            read_app_data(root.path()),
            Err(ReadError::Io { source, .. }) if source.kind() == ErrorKind::InvalidData
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_followed_and_cycles_end() {
        let root = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        write(elsewhere.path(), "shared/oak.json", b"oak");
        write(root.path(), "definitions/features/own.json", b"own");
        std::os::unix::fs::symlink(
            elsewhere.path().join("shared"),
            root.path().join("definitions/features/shared"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            root.path().join("definitions/features"),
            root.path().join("definitions/features/loop"),
        )
        .unwrap();
        let app_data = read_app_data(root.path()).unwrap();
        assert_eq!(
            contents(&app_data),
            vec![
                ("definitions/features/own.json".to_owned(), b"own".to_vec()),
                (
                    "definitions/features/shared/oak.json".to_owned(),
                    b"oak".to_vec()
                ),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_is_an_io_error_naming_it() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        write(root.path(), "definitions/features/oak.json", b"oak");
        let file = root.path().join("definitions/features/oak.json");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&file).is_ok() {
            eprintln!("skipped: permissions are not enforced for this user");
            return;
        }
        let result = read_app_data(root.path());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        match result {
            Err(ReadError::Io { path, source }) => {
                assert_eq!(path, file);
                assert_eq!(source.kind(), ErrorKind::PermissionDenied);
            }
            other => panic!("expected an I/O error, got {other:?}"),
        }
    }
}

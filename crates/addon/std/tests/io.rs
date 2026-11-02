//! `io::Roots` and `io::respond`: the root table and the blocking answer to one loader request,
//! against a real directory tree built in a temporary directory.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use bedrock_addon::load::{
    DirEntry, EntryKind, Input, ListDepth, Place, ReadRange, Request, RootId,
};
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::io::{Roots, respond};

/// A temporary directory registered as the only root of a fresh table.
fn rooted_temp_dir() -> (tempfile::TempDir, Roots, RootId) {
    let dir = tempfile::tempdir().unwrap();
    let mut roots = Roots::new();
    let root = roots.add(dir.path());
    (dir, roots, root)
}

#[test]
fn roots_respond_is_what_the_free_function_answers() {
    let (dir, roots, root) = rooted_temp_dir();
    write(dir.path(), "a.txt", b"abc");
    let request = || Request::ReadFile {
        location: place(root, "a.txt"),
        range: ReadRange::Whole,
        len_hint: None,
    };
    let method = roots.respond(request());
    assert!(matches!(&method, Input::File { bytes, .. } if bytes.as_ref() == b"abc"));
    assert_eq!(
        format!("{method:?}"),
        format!("{:?}", respond(&roots, request()))
    );
}

fn place(root: RootId, path: &str) -> Place {
    Place::root(root).join(&PackPath::new(path).unwrap())
}

fn write(dir: &Path, path: &str, bytes: &[u8]) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn read(roots: &Roots, location: &Place, range: ReadRange) -> Input {
    respond(
        roots,
        Request::ReadFile {
            location: location.clone(),
            range,
            len_hint: None,
        },
    )
}

fn list(roots: &Roots, location: &Place, depth: ListDepth) -> Input {
    respond(
        roots,
        Request::ListDir {
            location: location.clone(),
            depth,
        },
    )
}

/// The entries of a `Dir` answer for `location`, sorted by name.
fn entries(input: Input, location: &Place) -> Vec<DirEntry> {
    match input {
        Input::Dir {
            location: answered,
            mut entries,
        } => {
            assert_eq!(
                &answered, location,
                "the answer names the requested location"
            );
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            entries
        }
        other => panic!("expected a listing, got {other:?}"),
    }
}

fn file(name: &str, len: u64) -> DirEntry {
    DirEntry {
        name: name.into(),
        lossy: false,
        kind: EntryKind::File { len },
    }
}

fn dir(name: &str) -> DirEntry {
    DirEntry {
        name: name.into(),
        lossy: false,
        kind: EntryKind::Dir,
    }
}

/// Permission bits do not stop the superuser; tests that rely on them check first.
#[cfg(unix)]
fn permissions_are_enforced(probe: &Path) -> bool {
    fs::read_dir(probe).is_err() || fs::read(probe).is_err()
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
fn make_fifo(path: &Path) -> bool {
    std::process::Command::new("mkfifo")
        .arg(path)
        .status()
        .is_ok_and(|status| status.success())
}

mod roots {
    use super::*;

    #[test]
    fn host_path_joins_the_segments_onto_the_root() {
        let mut roots = Roots::new();
        let first = roots.add("/srv/packs");
        let second = roots.add(PathBuf::from("/srv/world.mcpack"));
        assert_ne!(first, second, "each registration has its own handle");
        assert_eq!(
            roots.host_path(&place(first, "blocks/ruby.json")),
            Some(PathBuf::from("/srv/packs/blocks/ruby.json"))
        );
        assert_eq!(
            roots.host_path(&Place::root(second)),
            Some(PathBuf::from("/srv/world.mcpack"))
        );
    }

    #[test]
    fn host_path_is_none_for_an_unknown_root() {
        let mut roots = Roots::new();
        let known = roots.add("/srv/packs");
        let unknown = RootId(known.0 + 1);
        assert_eq!(roots.host_path(&Place::root(unknown)), None);
        assert_eq!(Roots::default().host_path(&Place::root(known)), None);
    }

    #[test]
    fn host_path_is_none_for_memory() {
        let mut roots = Roots::new();
        roots.add("/srv/packs");
        assert_eq!(roots.host_path(&Place::root(RootId::MEMORY)), None);
        assert_eq!(
            roots.host_path(&place(RootId::MEMORY, "manifest.json")),
            None
        );
    }
}

mod read_file {
    use super::*;

    #[test]
    fn whole_answers_every_byte() {
        let (temp, roots, root) = rooted_temp_dir();
        write(
            temp.path(),
            "blocks/ruby.json",
            b"{\"format_version\":\"1.26.20\"}",
        );
        let location = place(root, "blocks/ruby.json");
        match read(&roots, &location, ReadRange::Whole) {
            Input::File {
                location: answered,
                bytes,
            } => {
                assert_eq!(answered, location);
                assert_eq!(&bytes[..], b"{\"format_version\":\"1.26.20\"}");
            }
            other => panic!("expected the file, got {other:?}"),
        }
    }

    #[test]
    fn prefix_answers_at_most_n_bytes() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "pack.mcpack", &[7u8; 1000]);
        match read(&roots, &place(root, "pack.mcpack"), ReadRange::Prefix(256)) {
            Input::File { bytes, .. } => assert_eq!(&bytes[..], &[7u8; 256][..]),
            other => panic!("expected the prefix, got {other:?}"),
        }
    }

    #[test]
    fn prefix_of_a_shorter_file_answers_the_whole_file() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "short", b"PK");
        match read(&roots, &place(root, "short"), ReadRange::Prefix(256)) {
            Input::File { bytes, .. } => assert_eq!(&bytes[..], b"PK"),
            other => panic!("expected the whole short file, got {other:?}"),
        }
    }

    #[test]
    fn the_root_itself_can_be_a_file() {
        let temp = tempfile::tempdir().unwrap();
        write(temp.path(), "pack.mcpack", b"PK\x03\x04");
        let mut roots = Roots::new();
        let root = roots.add(temp.path().join("pack.mcpack"));
        match read(&roots, &Place::root(root), ReadRange::Whole) {
            Input::File { bytes, .. } => assert_eq!(&bytes[..], b"PK\x03\x04"),
            other => panic!("expected the archive, got {other:?}"),
        }
    }

    #[test]
    fn a_missing_file_is_missing() {
        let (_temp, roots, root) = rooted_temp_dir();
        let location = place(root, "blocks/ruby.json");
        assert!(matches!(
            read(&roots, &location, ReadRange::Whole),
            Input::Missing { location: answered } if answered == location
        ));
    }

    #[test]
    fn a_directory_is_failed_as_a_directory() {
        let (temp, roots, root) = rooted_temp_dir();
        fs::create_dir(temp.path().join("blocks")).unwrap();
        let location = place(root, "blocks");
        assert!(matches!(
            read(&roots, &location, ReadRange::Whole),
            Input::Failed { location: answered, kind: ErrorKind::IsADirectory } if answered == location
        ));
    }

    #[test]
    fn a_file_below_a_file_is_failed() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "manifest.json", b"{}");
        assert!(matches!(
            read(
                &roots,
                &place(root, "manifest.json/inner"),
                ReadRange::Whole
            ),
            Input::Failed {
                kind: ErrorKind::NotADirectory,
                ..
            }
        ));
    }

    #[test]
    fn an_unknown_root_is_failed_as_invalid_input() {
        let (_temp, roots, root) = rooted_temp_dir();
        let location = place(RootId(root.0 + 1), "manifest.json");
        assert!(matches!(
            read(&roots, &location, ReadRange::Whole),
            Input::Failed { location: answered, kind: ErrorKind::InvalidInput } if answered == location
        ));
        assert!(matches!(
            read(&roots, &Place::root(RootId::MEMORY), ReadRange::Whole),
            Input::Failed {
                kind: ErrorKind::InvalidInput,
                ..
            }
        ));
    }

    #[test]
    fn a_symlinked_file_is_read_through_the_link() {
        let (temp, roots, root) = rooted_temp_dir();
        let target = tempfile::tempdir().unwrap();
        write(target.path(), "shared.json", b"{}");
        symlink(
            &target.path().join("shared.json"),
            &temp.path().join("linked.json"),
        );
        match read(&roots, &place(root, "linked.json"), ReadRange::Whole) {
            Input::File { bytes, .. } => assert_eq!(&bytes[..], b"{}"),
            other => panic!("expected the link target, got {other:?}"),
        }
    }

    #[test]
    fn a_dangling_symlink_is_missing() {
        let (temp, roots, root) = rooted_temp_dir();
        symlink(
            &temp.path().join("gone.json"),
            &temp.path().join("linked.json"),
        );
        assert!(matches!(
            read(&roots, &place(root, "linked.json"), ReadRange::Whole),
            Input::Missing { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_is_failed_with_permission_denied() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "secret.json", b"{}");
        let path = temp.path().join("secret.json");
        set_mode(&path, 0o000);
        if !permissions_are_enforced(&path) {
            eprintln!("skipped: permissions are not enforced for this user");
            return;
        }
        let input = read(&roots, &place(root, "secret.json"), ReadRange::Whole);
        set_mode(&path, 0o644);
        assert!(matches!(
            input,
            Input::Failed {
                kind: ErrorKind::PermissionDenied,
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_fifo_is_failed_without_blocking() {
        let (temp, roots, root) = rooted_temp_dir();
        if !make_fifo(&temp.path().join("pipe")) {
            eprintln!("skipped: mkfifo unavailable");
            return;
        }
        assert!(matches!(
            read(&roots, &place(root, "pipe"), ReadRange::Whole),
            Input::Failed {
                kind: ErrorKind::InvalidInput,
                ..
            }
        ));
    }
}

mod list_dir {
    use super::*;

    #[test]
    fn children_lists_files_and_directories_one_level_deep() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "a_pack/manifest.json", b"{}");
        write(temp.path(), "b.mcpack", b"PK\x03\x04");
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Children), &location),
            vec![dir("a_pack"), file("b.mcpack", 4)]
        );
    }

    #[test]
    fn recursive_lists_every_entry_with_slash_separated_names() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "manifest.json", b"{}");
        write(temp.path(), "blocks/ores/ruby.json", b"{ }");
        fs::create_dir(temp.path().join("empty")).unwrap();
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![
                dir("blocks"),
                dir("blocks/ores"),
                file("blocks/ores/ruby.json", 3),
                dir("empty"),
                file("manifest.json", 2),
            ]
        );
    }

    #[test]
    fn a_listing_below_the_root_is_relative_to_the_listed_location() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "behavior_packs/ruby/manifest.json", b"{}");
        let location = place(root, "behavior_packs/ruby");
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![file("manifest.json", 2)]
        );
    }

    #[test]
    fn symlinks_are_followed_to_files_and_directories() {
        let (temp, roots, root) = rooted_temp_dir();
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "shared/texture.png", b"png");
        write(outside.path(), "icon.png", b"icon");
        symlink(
            &outside.path().join("shared"),
            &temp.path().join("textures"),
        );
        symlink(
            &outside.path().join("icon.png"),
            &temp.path().join("pack_icon.png"),
        );
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![
                file("pack_icon.png", 4),
                dir("textures"),
                file("textures/texture.png", 3),
            ]
        );
    }

    #[test]
    fn a_symlink_cycle_ends_the_walk() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "pack/manifest.json", b"{}");
        // `pack/loop` points back at `pack`, `pack/up` at the listed root itself.
        symlink(&temp.path().join("pack"), &temp.path().join("pack/loop"));
        symlink(temp.path(), &temp.path().join("pack/up"));
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![dir("pack"), file("pack/manifest.json", 2)],
            "a link back into an entered directory is skipped"
        );
    }

    #[test]
    fn two_links_to_one_directory_at_one_level_keep_the_first_by_name() {
        let (temp, roots, root) = rooted_temp_dir();
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "shared/a.json", b"{}");
        // Created in reverse name order, so the host's listing order cannot decide.
        symlink(&outside.path().join("shared"), &temp.path().join("second"));
        symlink(&outside.path().join("shared"), &temp.path().join("first"));
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![dir("first"), file("first/a.json", 2)]
        );
    }

    #[test]
    fn a_directory_and_a_sibling_link_to_it_keep_the_directory() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "real/a.json", b"{}");
        // `alias` sorts before `real`; the real directory still wins.
        symlink(&temp.path().join("real"), &temp.path().join("alias"));
        let location = Place::root(root);
        for _ in 0..3 {
            assert_eq!(
                entries(list(&roots, &location, ListDepth::Recursive), &location),
                vec![dir("real"), file("real/a.json", 2)]
            );
        }
    }

    #[test]
    fn a_link_to_a_directory_that_is_not_an_ancestor_is_listed_again() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "pack/textures/a.png", b"png");
        fs::create_dir(temp.path().join("other")).unwrap();
        symlink(
            &temp.path().join("pack/textures"),
            &temp.path().join("other/textures"),
        );
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![
                dir("other"),
                dir("other/textures"),
                file("other/textures/a.png", 3),
                dir("pack"),
                dir("pack/textures"),
                file("pack/textures/a.png", 3),
            ],
            "only a link back into an ancestor is a cycle"
        );
    }

    #[test]
    fn a_recursive_listing_is_sorted_by_name() {
        let (temp, roots, root) = rooted_temp_dir();
        for name in ["b/2.json", "a.json", "b/1.json", "c.json"] {
            write(temp.path(), name, b"{}");
        }
        let names: Vec<Box<str>> = match list(&roots, &Place::root(root), ListDepth::Recursive) {
            Input::Dir { entries, .. } => entries.into_iter().map(|entry| entry.name).collect(),
            other => panic!("expected a listing, got {other:?}"),
        };
        assert_eq!(
            names,
            ["a.json", "b", "b/1.json", "b/2.json", "c.json"].map(Box::<str>::from)
        );
    }

    #[test]
    fn children_does_not_follow_cycles_because_it_does_not_descend() {
        let (temp, roots, root) = rooted_temp_dir();
        symlink(temp.path(), &temp.path().join("self"));
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Children), &location),
            vec![dir("self")]
        );
    }

    #[test]
    fn dangling_symlinks_are_skipped() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "manifest.json", b"{}");
        symlink(&temp.path().join("gone"), &temp.path().join("dangling"));
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![file("manifest.json", 2)]
        );
    }

    #[cfg(unix)]
    #[test]
    fn entries_that_are_neither_file_nor_directory_are_skipped() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "manifest.json", b"{}");
        if !make_fifo(&temp.path().join("pipe")) {
            eprintln!("skipped: mkfifo unavailable");
            return;
        }
        let location = Place::root(root);
        assert_eq!(
            entries(list(&roots, &location, ListDepth::Recursive), &location),
            vec![file("manifest.json", 2)]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_non_utf8_name_is_converted_and_marked_lossy() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let (temp, roots, root) = rooted_temp_dir();
        let raw = OsStr::from_bytes(b"bad\xffname");
        fs::create_dir(temp.path().join(raw)).unwrap();
        fs::write(temp.path().join(raw).join("a.json"), b"{}").unwrap();
        let location = Place::root(root);
        let listed = entries(list(&roots, &location, ListDepth::Recursive), &location);
        assert_eq!(
            listed,
            vec![
                DirEntry {
                    name: "bad\u{fffd}name".into(),
                    lossy: true,
                    kind: EntryKind::Dir,
                },
                DirEntry {
                    name: "bad\u{fffd}name/a.json".into(),
                    lossy: true,
                    kind: EntryKind::File { len: 2 },
                },
            ],
            "an entry below a lossy directory is lossy too"
        );
    }

    #[test]
    fn a_missing_directory_is_missing() {
        let (_temp, roots, root) = rooted_temp_dir();
        let location = place(root, "behavior_packs");
        for depth in [ListDepth::Children, ListDepth::Recursive] {
            assert!(matches!(
                list(&roots, &location, depth),
                Input::Missing { location: ref answered } if *answered == location
            ));
        }
    }

    #[test]
    fn a_file_is_failed_as_not_a_directory() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "pack.mcpack", b"PK");
        for depth in [ListDepth::Children, ListDepth::Recursive] {
            assert!(matches!(
                list(&roots, &place(root, "pack.mcpack"), depth),
                Input::Failed {
                    kind: ErrorKind::NotADirectory,
                    ..
                }
            ));
        }
    }

    #[test]
    fn an_unknown_root_is_failed_as_invalid_input() {
        let roots = Roots::new();
        assert!(matches!(
            list(&roots, &Place::root(RootId(0)), ListDepth::Recursive),
            Input::Failed {
                kind: ErrorKind::InvalidInput,
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_nested_directory_fails_the_whole_listing() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "manifest.json", b"{}");
        write(temp.path(), "locked/a.json", b"{}");
        let locked = temp.path().join("locked");
        set_mode(&locked, 0o000);
        if !permissions_are_enforced(&locked) {
            set_mode(&locked, 0o755);
            eprintln!("skipped: permissions are not enforced for this user");
            return;
        }
        let input = list(&roots, &Place::root(root), ListDepth::Recursive);
        set_mode(&locked, 0o755);
        assert!(matches!(
            input,
            Input::Failed {
                kind: ErrorKind::PermissionDenied,
                ..
            }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_root_is_failed_with_permission_denied() {
        let (temp, roots, root) = rooted_temp_dir();
        write(temp.path(), "locked/a.json", b"{}");
        let locked = temp.path().join("locked");
        set_mode(&locked, 0o000);
        if !permissions_are_enforced(&locked) {
            set_mode(&locked, 0o755);
            eprintln!("skipped: permissions are not enforced for this user");
            return;
        }
        let input = list(&roots, &place(root, "locked"), ListDepth::Children);
        set_mode(&locked, 0o755);
        assert!(matches!(
            input,
            Input::Failed {
                kind: ErrorKind::PermissionDenied,
                ..
            }
        ));
    }
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(windows)]
fn symlink(target: &Path, link: &Path) {
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link).unwrap();
    } else {
        std::os::windows::fs::symlink_file(target, link).unwrap();
    }
}

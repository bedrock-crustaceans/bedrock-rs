//! `RepositoryLoader`: sources in priority order, each source's entries sorted
//! bytewise, first registration wins, unreadable roots and invalid packs in the source reports,
//! composite archives, packs in memory, one key request per identity.

use std::io::ErrorKind;

use bedrock_addon::diagnostic::DiagnosticCode;
use bedrock_addon::load::{LoadOptions, RepositoryLoader, RootId, Source};
use bedrock_addon::pack::{OpenError, OpenOptions, Pack, PackOrigin, PackRepository, WorldPacks};
use bedrock_addon::storage::{PackPath, StorageError, ZipLimit};
use bytes::Bytes;

use super::responder::{Order, Responder, manifest, place};

/// The names of the registered packs, in registration order.
fn names(repository: &PackRepository) -> Vec<String> {
    repository
        .packs()
        .map(|pack| pack.manifest().header.name.clone())
        .collect()
}

/// Two packs directories: root 1 holds packs 3 and 1 (as `b/`, `a/`), root 2 holds pack 2.
fn two_directories(order: Order) -> Responder {
    let mut responder = Responder::new(order);
    responder
        .file(1, "b/manifest.json", manifest(3, "data"))
        .file(1, "a/manifest.json", manifest(1, "data"))
        .file(1, "a/entities/pig.json", &b"{}"[..])
        .file(2, "c/manifest.json", manifest(2, "data"));
    responder
}

fn load(responder: &mut Responder, sources: Vec<Source>) -> PackRepository {
    let mut loader = RepositoryLoader::new(sources, OpenOptions::default());
    responder.drive(&mut loader)
}

fn directory(root: u32, origin: PackOrigin) -> Source {
    Source::Directory {
        root: RootId(root),
        origin,
    }
}

#[test]
fn packs_register_in_source_order_then_bytewise_within_a_source() {
    for order in Order::ALL {
        let mut responder = two_directories(order);
        let repository = load(
            &mut responder,
            vec![
                directory(2, PackOrigin::World),
                directory(1, PackOrigin::World),
            ],
        );
        assert_eq!(
            names(&repository),
            ["pack 2", "pack 1", "pack 3"],
            "{order:?}"
        );
        let reports = repository.source_reports();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].location, place(2, ""));
        assert_eq!(reports[1].packs.len(), 2);
        assert!(repository.invalid().is_empty());
        // Only manifests were read.
        let pack = repository.packs().nth(1).unwrap();
        assert!(!pack.is_resident(&PackPath::from_static("entities/pig.json")));
        assert_eq!(*pack.location(), place(1, "a"));
    }
}

#[test]
fn the_first_registration_of_an_identity_and_origin_wins_across_sources() {
    let mut responder = Responder::new(Order::Reverse);
    responder
        .file(1, "x/manifest.json", manifest(1, "data"))
        .file(1, "x/first.txt", &b"1"[..])
        .file(2, "y/manifest.json", manifest(1, "data"))
        .file(2, "y/second.txt", &b"2"[..]);
    let repository = load(
        &mut responder,
        vec![
            directory(1, PackOrigin::World),
            directory(2, PackOrigin::World),
        ],
    );
    assert_eq!(repository.packs().count(), 1);
    let kept = repository.packs().next().unwrap();
    assert!(kept.paths().any(|path| path.as_str() == "first.txt"));
    // Both sources list what they found; the drop is a warning of every stack set.
    assert_eq!(repository.source_reports()[1].packs.len(), 1);
    let set = repository.stacks(
        &WorldPacks::parse(None, None).unwrap(),
        LoadOptions::default(),
    );
    // The game dedups in silence: an information.
    let warnings: Vec<_> = set
        .selection()
        .iter()
        .filter(|d| d.severity == bedrock_addon::diagnostic::Severity::Info)
        .filter(|d| matches!(d.code, DiagnosticCode::DuplicateIdentifier { .. }))
        .map(|d| &d.code)
        .collect();
    assert!(
        matches!(warnings[..], [DiagnosticCode::DuplicateIdentifier { .. }]),
        "{warnings:?}"
    );
}

#[test]
fn the_same_identity_from_another_origin_registers_too() {
    let mut responder = Responder::new(Order::Issue);
    responder
        .file(1, "x/manifest.json", manifest(1, "data"))
        .file(2, "y/manifest.json", manifest(1, "data"));
    let repository = load(
        &mut responder,
        vec![
            directory(1, PackOrigin::World),
            directory(2, PackOrigin::User),
        ],
    );
    assert_eq!(repository.packs().count(), 2);
}

#[test]
fn an_unreadable_source_root_is_in_its_report() {
    let mut responder = two_directories(Order::Issue);
    responder
        .missing(7, "")
        .failed(1, "", ErrorKind::PermissionDenied);
    let repository = load(
        &mut responder,
        vec![
            directory(7, PackOrigin::World),
            directory(1, PackOrigin::World),
            directory(2, PackOrigin::World),
        ],
    );
    let unreadable: Vec<_> = repository
        .source_reports()
        .iter()
        .map(|report| report.unreadable)
        .collect();
    assert_eq!(
        unreadable,
        [
            Some(ErrorKind::NotFound),
            Some(ErrorKind::PermissionDenied),
            None
        ]
    );
    assert_eq!(names(&repository), ["pack 2"]);
}

#[test]
fn an_invalid_candidate_is_an_invalid_pack_of_its_source() {
    let mut responder = two_directories(Order::Issue);
    responder
        .file(1, "empty/readme.txt", &b"no manifest"[..])
        .file(1, "notes.txt", &b"not an archive"[..])
        .raw_name(1, "", "bad:name");
    let repository = load(&mut responder, vec![directory(1, PackOrigin::World)]);
    assert_eq!(names(&repository), ["pack 1", "pack 3"]);
    let invalid: Vec<_> = repository
        .invalid()
        .iter()
        .map(|pack| (pack.location.path.as_str().to_owned(), pack.origin))
        .collect();
    assert_eq!(
        invalid,
        [
            (String::new(), PackOrigin::World),
            ("empty".to_owned(), PackOrigin::World),
            ("notes.txt".to_owned(), PackOrigin::World),
        ]
    );
    assert!(matches!(
        repository.invalid()[0].error,
        OpenError::Storage(bedrock_addon::storage::StorageError::PathRejected { .. })
    ));
    assert!(matches!(
        repository.invalid()[1].error,
        OpenError::NoManifest { .. }
    ));
    assert_eq!(repository.source_reports()[0].invalid.len(), 3);
}

#[test]
fn a_lossy_candidate_name_is_an_unreadable_invalid_pack() {
    let mut responder = two_directories(Order::Issue);
    responder.raw_entry(1, "", "caf\u{fffd}.mcpack", true);
    let repository = load(&mut responder, vec![directory(1, PackOrigin::World)]);
    assert_eq!(names(&repository), ["pack 1", "pack 3"]);
    assert!(
        matches!(
            repository.invalid(),
            [pack] if matches!(pack.error, OpenError::Unreadable { kind: ErrorKind::InvalidData, .. })
        ),
        "{:?}",
        repository.invalid()
    );
}

#[test]
fn packs_in_memory_register_in_their_source_position() {
    let memory = Pack::from_files(
        [(
            PackPath::from_static("manifest.json"),
            Bytes::from(manifest(9, "resources")),
        )],
        &OpenOptions::default(),
    )
    .unwrap();
    let mut responder = two_directories(Order::Issue);
    let repository = load(
        &mut responder,
        vec![
            directory(2, PackOrigin::World),
            Source::Packs {
                packs: vec![memory],
                origin: PackOrigin::Package,
            },
        ],
    );
    assert_eq!(names(&repository), ["pack 2", "pack 9"]);
    assert_eq!(repository.source_reports()[1].location.root, RootId::MEMORY);
}

#[cfg(feature = "zip")]
mod archives {
    use std::io::{Cursor, Write};

    use super::*;

    /// A zip of `members` (name, bytes), stored.
    fn zip(members: &[(&str, Bytes)]) -> Bytes {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in members {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        Bytes::from(writer.finish().unwrap().into_inner())
    }

    fn mcpack(n: u128, module: &str) -> Bytes {
        zip(&[("manifest.json", Bytes::from(manifest(n, module)))])
    }

    fn archive_source(root: u32) -> Source {
        Source::Archive {
            root: RootId(root),
            origin: PackOrigin::World,
        }
    }

    /// A pack opened from an archive keeps the archive: `to_archive` hands back its bytes,
    /// which hold more than the members (here the zip's comment). Two archives with the same
    /// members and another comment are two different packs of a repository.
    #[test]
    fn archives_that_differ_outside_their_members_are_told_apart() {
        let zipped = |comment: &str| {
            let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
            writer.set_comment(comment).unwrap();
            for (name, bytes) in [
                ("manifest.json", manifest(1, "data").into_bytes()),
                ("entities/pig.json", b"{}".to_vec()),
            ] {
                writer
                    .start_file(name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(&bytes).unwrap();
            }
            Bytes::from(writer.finish().unwrap().into_inner())
        };
        let load_of = |comment: &str| {
            let mut responder = Responder::new(Order::Issue);
            responder.root_file(4, zipped(comment));
            load(&mut responder, vec![archive_source(4)])
        };
        let (one, other, same) = (load_of("one"), load_of("two"), load_of("one"));
        let archive = |repository: &PackRepository| {
            let pack = repository.packs().next().expect("the pack");
            pack.to_archive().expect("the archive")
        };
        assert_ne!(archive(&one), archive(&other));
        assert_eq!(names(&one), names(&other));
        assert_eq!(describe(&one), describe(&same));
        assert_ne!(describe(&one), describe(&other));
    }

    #[test]
    fn a_zipped_world_holds_its_behavior_and_resource_packs() {
        let bytes = Bytes::from(super::super::fixture(
            "storage/zip_packs/FlatWorldWithEditorPacksAndPlayerData.zip",
        ));
        let mut responder = Responder::new(Order::Issue);
        responder.root_file(4, bytes);
        let repository = load(&mut responder, vec![archive_source(4)]);
        let locations: Vec<String> = repository
            .packs()
            .map(|pack| pack.location().path.as_str().to_owned())
            .collect();
        assert_eq!(
            locations,
            [
                "behavior_packs/OpticEditorCommandBP",
                "behavior_packs/OpticVanillaCommandBP",
                "resource_packs/OpticEditorCoalReplaceRP",
                "resource_packs/OpticVanillaGoldReplaceRP",
            ]
        );
        assert!(repository.invalid().is_empty());
    }

    #[test]
    fn a_nested_mcaddon_opens_its_packs_from_the_outer_bytes() {
        let inner = zip(&[
            ("rp.mcpack", mcpack(2, "resources")),
            ("Loose/manifest.json", Bytes::from(manifest(3, "data"))),
        ]);
        let outer = zip(&[("bp.mcpack", mcpack(1, "data")), ("more.mcaddon", inner)]);
        let mut responder = Responder::new(Order::Seeded(3));
        responder.root_file(4, outer);
        let repository = load(&mut responder, vec![archive_source(4)]);
        assert_eq!(names(&repository), ["pack 1", "pack 3", "pack 2"]);
        // Two reads (prefix and whole): nested archives need no further requests.
        assert_eq!(responder.log.requests.len(), 2);
        let locations: Vec<String> = repository
            .packs()
            .map(|pack| pack.location().to_string())
            .collect();
        assert_eq!(
            locations,
            [
                "#4/bp.mcpack",
                "#4/more.mcaddon/Loose",
                "#4/more.mcaddon/rp.mcpack"
            ]
        );
    }

    #[test]
    fn archives_nested_deeper_than_four_are_refused() {
        let mut bytes = mcpack(1, "data");
        for depth in 0..4 {
            bytes = zip(&[
                (&format!("level{depth}.mcaddon"), bytes),
                ("x.mcpack", mcpack(10 + depth, "data")),
            ]);
        }
        let mut responder = Responder::new(Order::Issue);
        responder.root_file(4, bytes);
        let repository = load(&mut responder, vec![archive_source(4)]);
        // The outer archive is level 1; `x.mcpack` of levels 2 to 4 open, both members of the
        // fourth archive sit at level 5. Members sort bytewise by their path in the archive.
        assert_eq!(names(&repository), ["pack 11", "pack 12", "pack 13"]);
        assert_eq!(repository.invalid().len(), 2);
        for invalid in repository.invalid() {
            assert!(matches!(
                invalid.error,
                OpenError::Storage(StorageError::LimitExceeded {
                    limit: ZipLimit::Nesting
                })
            ));
        }
    }
}

#[cfg(feature = "encrypted")]
#[test]
fn two_packs_of_one_identity_ask_for_its_key_once() {
    let contents = super::fixture("storage/encrypted_synthetic/contents.json");
    let mut responder = Responder::new(Order::Seeded(9));
    responder
        .file(1, "a/manifest.json", manifest(1, "data"))
        .file(1, "a/contents.json", contents.clone())
        .file(1, "b/manifest.json", manifest(2, "data"))
        .file(1, "b/contents.json", contents);
    let repository = load(&mut responder, vec![directory(1, PackOrigin::World)]);
    assert_eq!(responder.log.keys.len(), 1);
    assert_eq!(repository.packs().count(), 2);
}

/// Every public method of `Pack` (the two `impl Pack` blocks, `src/pack/mod.rs` and
/// `src/pack/open.rs`), and how [`describe`] takes it. `the_description_of_a_pack_takes_every_public_method`
/// checks the names against the sources.
const PACK_METHODS: &[(&str, &str)] = &[
    ("manifest", "written in full (`Manifest` derives `Debug`)"),
    ("id", "written"),
    ("pack_type", "written"),
    ("report", "written in full"),
    ("icon", "written: the bytes"),
    ("icon_findings", "written"),
    (
        "check_icon",
        "no view of a pack: a function of the bytes it is given",
    ),
    ("location", "written"),
    ("is_resident", "written for every path"),
    ("paths", "walked: every path is written"),
    ("from_files", "a constructor"),
    ("from_archive", "a constructor"),
    (
        "read",
        "written for every path: the bytes, or why they cannot be read",
    ),
    (
        "to_archive",
        "written with `zip`: the archive's bytes, or the error",
    ),
];

/// A repository as text, for the comparison of two loads of it: the registered packs in
/// registration order, each by everything it holds, then the invalid ones and the source
/// reports.
///
/// A `Pack`'s `Debug` is its identity and location. Here a pack is every view its public
/// methods give ([`PACK_METHODS`]): its identity, type and location, its manifest and its
/// report in full, its icon and the icon's findings, every file it lists with whether the file
/// is resident and its bytes (or why they cannot be read), and, with the feature `zip`, the
/// pack as an archive: for a pack opened from one that is the archive's own bytes, which hold
/// more than the members.
fn describe(repository: &PackRepository) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for pack in repository.packs() {
        writeln!(
            text,
            "pack {:?} {:?} at {:?}\n  manifest: {:?}\n  report: {:?}\n  icon: {:?}\n  icon findings: {:?}",
            pack.id(),
            pack.pack_type(),
            pack.location(),
            pack.manifest(),
            pack.report(),
            pack.icon(),
            pack.icon_findings(),
        )
        .unwrap();
        for path in pack.paths() {
            writeln!(
                text,
                "  {path}: resident {}, {:?}",
                pack.is_resident(path),
                pack.read(path)
            )
            .unwrap();
        }
        #[cfg(feature = "zip")]
        writeln!(text, "  as an archive: {:?}", pack.to_archive()).unwrap();
    }
    write!(
        text,
        "{:?} {:?}",
        repository.invalid(),
        repository.source_reports()
    )
    .unwrap();
    text
}

/// The public methods of `Pack`, from the sources: every `pub fn` and `pub const fn` inside
/// an `impl Pack {` block of a file below `src/`. A scan of the text: it sees an inherent
/// method written out in such a block, and not one a macro makes, nor a trait's.
fn public_methods_of_pack() -> Vec<String> {
    fn walk(dir: &std::path::Path, found: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                let mut inside = false;
                for line in source.lines() {
                    if line == "impl Pack {" {
                        inside = true;
                    } else if line == "}" {
                        inside = false;
                    } else if inside {
                        let line = line.trim_start();
                        let rest = line
                            .strip_prefix("pub fn ")
                            .or_else(|| line.strip_prefix("pub const fn "));
                        if let Some(rest) = rest {
                            let name: String = rest
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            found.push(name);
                        }
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut found,
    );
    found.sort();
    found
}

/// `describe` says of each public method of `Pack` how it takes it: a method added to `Pack`
/// without a row fails here by name.
#[test]
fn the_description_of_a_pack_takes_every_public_method() {
    let mut listed: Vec<String> = PACK_METHODS
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    listed.sort();
    assert_eq!(public_methods_of_pack(), listed);
}

/// A zipped pack (7) with a comment, an icon and files in folders.
#[cfg(feature = "zip")]
fn archived_pack() -> Bytes {
    use std::io::Write as _;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    writer.set_comment("an archive of the order test").unwrap();
    for (name, bytes) in [
        ("manifest.json", manifest(7, "data").into_bytes()),
        ("pack_icon.png", b"\x89PNG\r\n\x1a\nicon".to_vec()),
        ("entities/pig.json", b"{}".to_vec()),
        ("loot_tables/chests/a.json", b"{ \"pools\": [] }".to_vec()),
    ] {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    Bytes::from(writer.finish().unwrap().into_inner())
}

#[test]
fn every_completion_order_gives_the_same_repository() {
    let mut reference = None;
    for order in Order::ALL {
        let mut responder = two_directories(order);
        // More than a manifest to a pack: files in folders, an icon, a subpack.
        responder
            .file(1, "z/readme.txt", &b"x"[..])
            .file(1, "a/pack_icon.png", &b"\x89PNG\r\n\x1a\nicon"[..])
            .file(1, "a/entities/cow.json", &b"{ \"cow\": 1 }"[..])
            .file(1, "a/subpacks/big/entities/pig.json", &b"[]"[..])
            .file(1, "b/texts/en_US.lang", &b"a=b\n"[..])
            .file(2, "c/functions/tick.json", &b"{ \"values\": [] }"[..]);
        // A pack opened from an archive, behind a third source.
        #[cfg(feature = "zip")]
        responder.root_file(3, archived_pack());
        let repository = load(
            &mut responder,
            vec![
                directory(1, PackOrigin::World),
                directory(2, PackOrigin::Dev),
                #[cfg(feature = "zip")]
                Source::Archive {
                    root: RootId(3),
                    origin: PackOrigin::World,
                },
            ],
        );
        #[cfg(feature = "zip")]
        assert!(
            names(&repository).contains(&String::from("pack 7")),
            "the archive's pack is registered: {:?}",
            names(&repository)
        );
        let text = describe(&repository);
        assert_eq!(
            reference.get_or_insert_with(|| text.clone()),
            &text,
            "{order:?}"
        );
    }
}

#[test]
fn the_read_window_bounds_outstanding_reads_and_an_oversized_read_goes_alone() {
    let mut responder = Responder::new(Order::Reverse);
    for n in 0..8 {
        responder.file(1, &format!("p{n}/manifest.json"), manifest(n + 1, "data"));
    }
    // A manifest larger than the whole byte window.
    let mut big = manifest(20, "data");
    big.push_str(&" ".repeat(4000));
    responder.file(1, "q/manifest.json", big.clone());
    let small = manifest(1, "data").len() as u64;
    let options = OpenOptions {
        limits: bedrock_addon::load::Limits {
            max_pending_reads: 2,
            max_pending_read_bytes: small * 3,
            ..bedrock_addon::load::Limits::default()
        },
        ..OpenOptions::default()
    };
    let mut loader = RepositoryLoader::new(vec![directory(1, PackOrigin::World)], options);
    let repository = responder.drive(&mut loader);
    assert_eq!(repository.packs().count(), 9);
    assert!(responder.log.max_reads_out <= 2, "{:?}", responder.log);
    // The big manifest was read with nothing else outstanding.
    assert!(responder.log.max_read_bytes_out <= (big.len() as u64).max(small * 3));
    assert!(responder.log.max_read_bytes_out >= big.len() as u64);
}

#[test]
fn sources_on_one_root_share_its_requests_and_an_unreadable_archive_is_its_report() {
    // Two packs directories and an archive on root 1 (no file at the root itself): one place,
    // asked about one request at a time, every answer accepted.
    let mut responder = two_directories(Order::Seeded(3));
    let repository = load(
        &mut responder,
        vec![
            directory(1, PackOrigin::World),
            directory(1, PackOrigin::World),
            Source::Archive {
                root: RootId(1),
                origin: PackOrigin::World,
            },
        ],
    );
    assert_eq!(names(&repository), ["pack 1", "pack 3"]);
    let unreadable: Vec<_> = repository
        .source_reports()
        .iter()
        .map(|report| report.unreadable)
        .collect();
    assert_eq!(unreadable, [None, None, Some(ErrorKind::NotFound)]);
    assert!(
        repository.invalid().is_empty(),
        "{:?}",
        repository.invalid()
    );
    let lists = responder
        .log
        .requests
        .iter()
        .filter(|request| matches!(request, bedrock_addon::load::Request::ListDir { location, .. } if *location == place(1, "")))
        .count();
    assert_eq!(lists, 1, "one listing serves both directory sources");

    let mut responder = Responder::new(Order::Issue);
    responder.failed(6, "", ErrorKind::PermissionDenied);
    let repository = load(
        &mut responder,
        vec![Source::Archive {
            root: RootId(6),
            origin: PackOrigin::World,
        }],
    );
    assert_eq!(
        repository.source_reports()[0].unreadable,
        Some(ErrorKind::PermissionDenied)
    );
    assert!(repository.invalid().is_empty());
}

#[cfg(feature = "zip")]
#[test]
fn a_pack_folder_deeper_than_four_is_too_deep() {
    use std::io::{Cursor, Write};
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, n) in [("ok/manifest.json", 1), ("a/b/c/d/deep/manifest.json", 2)] {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(manifest(n, "data").as_bytes()).unwrap();
    }
    let bytes = Bytes::from(writer.finish().unwrap().into_inner());
    let mut responder = Responder::new(Order::Issue);
    responder.root_file(4, bytes.clone());
    let repository = load(
        &mut responder,
        vec![Source::Archive {
            root: RootId(4),
            origin: PackOrigin::World,
        }],
    );
    assert_eq!(names(&repository), ["pack 1"]);
    assert!(
        matches!(
            repository.invalid(),
            [pack] if pack.location == place(4, "a/b/c/d/deep")
                && matches!(pack.error, OpenError::Storage(StorageError::LimitExceeded { limit: ZipLimit::Nesting }))
        ),
        "{:?}",
        repository.invalid()
    );
}

/// The description two loads are compared by tells two repositories apart whose packs
/// differ in what a pack of a repository holds: a member of the manifest, a file's path.
#[test]
fn repositories_whose_packs_differ_in_content_are_told_apart() {
    let with_metadata = |license: &str| {
        manifest(1, "data").replacen(
            "\"modules\"",
            &format!("\"metadata\": {{ \"license\": \"{license}\" }},\n  \"modules\""),
            1,
        )
    };
    let load_of = |manifest: String, path: &str, bytes: &[u8]| {
        let mut responder = Responder::new(Order::Issue);
        responder.file(1, "a/manifest.json", manifest).file(
            1,
            &format!("a/{path}"),
            bytes.to_vec(),
        );
        load(&mut responder, vec![directory(1, PackOrigin::World)])
    };
    let reference = load_of(with_metadata("MIT"), "entities/pig.json", b"{}");
    assert_eq!(
        describe(&reference),
        describe(&load_of(with_metadata("MIT"), "entities/pig.json", b"{}"))
    );
    for (what, other) in [
        (
            "a member of the manifest",
            load_of(with_metadata("CC0"), "entities/pig.json", b"{}"),
        ),
        (
            "the path of a file",
            load_of(with_metadata("MIT"), "entities/cow.json", b"{}"),
        ),
    ] {
        assert_eq!(names(&reference), names(&other), "{what}");
        assert_ne!(describe(&reference), describe(&other), "{what}");
    }
    // A byte of a content file is no state of a repository: it lists a pack's files and
    // reads the manifest; the files are read by the stack load, whose own order test compares
    // what comes of them.
    let other = load_of(with_metadata("MIT"), "entities/pig.json", b"[]");
    assert_eq!(describe(&reference), describe(&other));
    let pig = PackPath::from_static("entities/pig.json");
    for repository in [&reference, &other] {
        let pack = repository.packs().next().unwrap();
        assert!(!pack.is_resident(&pig));
        assert!(matches!(pack.read(&pig), Err(StorageError::NotResident)));
    }
}

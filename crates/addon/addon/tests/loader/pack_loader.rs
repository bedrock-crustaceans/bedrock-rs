//! `PackLoader`: the folder and archive request sequences, the
//! findings for failed, missing and lossy answers, panicked jobs, progress, cancellation and
//! determinism over completion orders.

use bedrock_addon_core::diagnostic::codes as core_codes;
use std::io::ErrorKind;

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{
    Input, JobKind, ListDepth, Loader, Output, PackLoader, Place, ReadRange, Request, RootId, Sans,
    Stage,
};
use bedrock_addon::pack::{OpenError, OpenOptions, Pack};
use bedrock_addon::storage::{ContentIdentity, ContentKey, ContentKeys, PackPath};
use bytes::Bytes;

use super::responder::{Order, Responder, manifest, place};

fn folder(order: Order) -> Responder {
    let mut responder = Responder::new(order);
    responder.pack(
        1,
        "",
        &[
            ("manifest.json", manifest(1, "data").as_bytes()),
            ("entities/pig.json", b"{}"),
            ("texts/en_US.lang", b"a=b"),
        ],
    );
    responder
}

fn open_folder(responder: &mut Responder, options: OpenOptions) -> Result<Pack, OpenError> {
    let mut loader = PackLoader::directory(RootId(1), options);
    responder.drive(&mut loader)
}

#[test]
fn a_folder_is_listed_then_its_manifest_read_and_parsed() {
    let mut responder = folder(Order::Issue);
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    assert_eq!(
        responder.log.requests,
        [
            Request::ListDir {
                location: Place::root(RootId(1)),
                depth: ListDepth::Recursive,
            },
            Request::ReadFile {
                location: place(1, "manifest.json"),
                range: ReadRange::Whole,
                len_hint: Some(manifest(1, "data").len() as u64),
            },
        ]
    );
    assert_eq!(responder.log.jobs, [JobKind::Manifest]);
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    assert_eq!(
        paths,
        ["entities/pig.json", "manifest.json", "texts/en_US.lang"]
    );
    // Indexed, not read: the content is the stack loader's.
    assert!(pack.is_resident(&PackPath::from_static("manifest.json")));
    assert!(!pack.is_resident(&PackPath::from_static("entities/pig.json")));
    assert_eq!(*pack.location(), Place::root(RootId(1)));
}

#[cfg(feature = "zip")]
#[test]
fn an_indexed_pack_cannot_be_zipped_until_its_files_are_resident() {
    let pack = open_folder(&mut folder(Order::Issue), OpenOptions::default()).unwrap();
    // A zip of the files needs every one resident (`Residency::All`).
    assert!(matches!(
        pack.to_archive(),
        Err(bedrock_addon::pack::WriteError::Storage(
            bedrock_addon::storage::StorageError::NotResident
        ))
    ));
}

#[test]
fn a_folder_with_contents_json_reads_it_and_indexes_it() {
    let mut responder = folder(Order::Issue);
    responder.file(1, "contents.json", &br#"{"version":1,"content":[]}"#[..]);
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    assert!(matches!(
        responder.log.requests.last(),
        Some(Request::ReadFile { location, .. }) if *location == place(1, "contents.json")
    ));
    assert_eq!(responder.log.jobs, [JobKind::Manifest, JobKind::Index]);
    assert!(responder.log.keys.is_empty());
    assert!(pack.report().diagnostics().is_empty());
}

/// A plain `contents.json` without `version` (every shipped vanilla pack's) or with another one:
/// the game has no message for it (its only `contents.json` texts are "contents.json not found…"
/// and "…no key entitlement…", 1.26.60), so the crate's own finding is
/// an information, at `contents.json` `version`.
#[test]
fn a_contents_json_with_another_version_is_reported() {
    for table in [
        &br#"{"content":[]}"#[..],
        &br#"{"version":2,"content":[]}"#[..],
    ] {
        let mut responder = folder(Order::Issue);
        responder.file(1, "contents.json", table);
        let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
        let found: Vec<_> = pack.report().diagnostics().iter().collect();
        assert!(
            matches!(found[..], [d] if d.is(&core_codes::INVALID_VALUE)),
            "{found:?}"
        );
        assert_eq!(
            found[0].severity,
            Severity::Info,
            "an information, not a warning"
        );
        assert_eq!(
            found[0].location.path.as_ref().map(PackPath::as_str),
            Some("contents.json")
        );
        assert_eq!(found[0].location.json_path.to_string(), "version");
    }
}

#[cfg(feature = "encrypted")]
mod encrypted {
    use super::*;

    const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";
    const IDENTITY: &str = "00000000-0000-4000-8000-00000000c0de";

    fn encrypted(order: Order) -> Responder {
        let mut responder = folder(order);
        let dir = "../../../storage/tests/fixtures/encrypted_synthetic";
        responder
            .file(
                1,
                "contents.json",
                super::super::fixture(&format!("{dir}/contents.json")),
            )
            .file(
                1,
                "blocks/ruby.json",
                super::super::fixture(&format!("{dir}/blocks/ruby.json")),
            );
        responder
    }

    #[test]
    fn an_encrypted_contents_json_asks_for_its_key() {
        let mut responder = encrypted(Order::Issue);
        responder.key(IDENTITY, ContentKey::new(*PACK_KEY));
        let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
        assert_eq!(responder.log.keys, [ContentIdentity::new(IDENTITY)]);
        assert!(!pack.report().has_errors());
        assert!(
            pack.report().diagnostics().is_empty(),
            "{:?}",
            pack.report()
        );
    }

    #[test]
    fn a_key_in_the_options_is_not_asked_for() {
        let mut responder = encrypted(Order::Issue);
        let keys: ContentKeys = [(ContentIdentity::new(IDENTITY), ContentKey::new(*PACK_KEY))]
            .into_iter()
            .collect();
        let pack = open_folder(
            &mut responder,
            OpenOptions {
                keys,
                ..OpenOptions::default()
            },
        )
        .unwrap();
        assert!(responder.log.keys.is_empty());
        assert!(pack.report().diagnostics().is_empty());
    }

    #[test]
    fn no_key_for_an_encrypted_pack_is_unsupported_file_format() {
        let mut responder = encrypted(Order::Issue);
        let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
        assert_eq!(responder.log.keys.len(), 1);
        let codes: Vec<_> = pack
            .report()
            .diagnostics()
            .iter()
            .map(|d| &d.code)
            .collect();
        assert_eq!(
            codes,
            [&DiagnosticCode::from(&core_codes::UNSUPPORTED_FILE_FORMAT)],
            "{codes:?}"
        );
    }
}

#[cfg(feature = "zip")]
#[test]
fn an_archive_is_read_by_its_prefix_then_whole_then_indexed() {
    let zip = Pack::from_files(
        [
            (
                PackPath::from_static("manifest.json"),
                Bytes::from(manifest(2, "resources")),
            ),
            (
                PackPath::from_static("textures/a.png"),
                Bytes::from_static(b"png"),
            ),
        ],
        &OpenOptions::default(),
    )
    .unwrap()
    .to_archive()
    .unwrap();
    let mut responder = Responder::new(Order::Issue);
    responder.file(5, "pack.mcpack", zip.clone());
    let mut loader = PackLoader::file(RootId(5), Some(zip.len() as u64), OpenOptions::default());
    // The single file is the root itself: answer reads of the root with the archive.
    let mut requests = Vec::new();
    let pack = loop {
        let Some(output) = loader.poll() else {
            panic!("stalled");
        };
        let input = match output {
            Output::Done(result) => break result.unwrap(),
            Output::Request(request) => {
                requests.push(request.clone());
                let Request::ReadFile {
                    location, range, ..
                } = request
                else {
                    panic!("a file is never listed");
                };
                let bytes = match range {
                    ReadRange::Prefix(n) => zip.slice(..(n as usize).min(zip.len())),
                    ReadRange::Whole => zip.clone(),
                };
                Input::File { location, bytes }
            }
            Output::Job(job) => Input::JobDone(job.run()),
            Output::NeedKey(_) => panic!("no key"),
        };
        loader.handle(input).unwrap();
    };
    let root = Place::root(RootId(5));
    assert_eq!(
        requests,
        [
            Request::ReadFile {
                location: root.clone(),
                range: ReadRange::Prefix(256),
                len_hint: Some(zip.len() as u64),
            },
            Request::ReadFile {
                location: root,
                range: ReadRange::Whole,
                len_hint: Some(zip.len() as u64),
            },
        ]
    );
    assert!(pack.is_resident(&PackPath::from_static("textures/a.png")));
    let progress = loader.progress();
    assert_eq!(progress.stage, Stage::Finished);
    assert_eq!(progress.packs, 1);
    assert_eq!(progress.requests_issued, 2);
    assert_eq!(progress.requests_answered, 2);
    assert_eq!(progress.jobs_issued, 2);
    assert_eq!(progress.jobs_done, 2);
    assert_eq!(
        progress.bytes_received,
        256.min(zip.len() as u64) + zip.len() as u64
    );
}

#[test]
fn a_missing_or_unreadable_root_is_unreadable() {
    let mut responder = Responder::new(Order::Issue);
    responder.missing(1, "");
    assert!(matches!(
        open_folder(&mut responder, OpenOptions::default()),
        Err(OpenError::Unreadable {
            kind: ErrorKind::NotFound,
            ..
        })
    ));
    let mut responder = folder(Order::Issue);
    responder.failed(1, "", ErrorKind::PermissionDenied);
    assert!(matches!(
        open_folder(&mut responder, OpenOptions::default()),
        Err(OpenError::Unreadable {
            kind: ErrorKind::PermissionDenied,
            ..
        })
    ));
}

#[test]
fn an_unreadable_manifest_is_unreadable() {
    let mut responder = folder(Order::Issue);
    responder.failed(1, "manifest.json", ErrorKind::Interrupted);
    let result = open_folder(&mut responder, OpenOptions::default());
    assert!(
        matches!(&result, Err(OpenError::Unreadable { location, kind: ErrorKind::Interrupted }) if *location == place(1, "manifest.json")),
        "{result:?}"
    );
}

#[test]
fn an_unreadable_contents_json_is_a_read_error_and_the_pack_opens() {
    let mut responder = folder(Order::Issue);
    responder
        .file(1, "contents.json", &b"{}"[..])
        .failed(1, "contents.json", ErrorKind::Other);
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    let codes: Vec<_> = pack
        .report()
        .diagnostics()
        .iter()
        .map(|d| (d.is(&core_codes::READ_ERROR), d.detail.as_deref()))
        .collect();
    assert_eq!(
        codes,
        [(
            true,
            Some(&bedrock_addon::diagnostic::DiagnosticDetail::Io(
                ErrorKind::Other
            ))
        )],
        "{codes:?}"
    );
}

#[test]
fn lossy_and_rejected_names_are_skipped_with_path_rejected() {
    let mut responder = folder(Order::Issue);
    responder
        .file(1, "textures/b?d.png", &b"x"[..])
        .lossy(1, "textures/b?d.png")
        .raw_name(1, "", "../escape.json");
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    let findings: Vec<_> = pack.report().diagnostics().iter().collect();
    assert_eq!(findings.len(), 2, "{findings:?}");
    for finding in findings {
        assert!(finding.is(&core_codes::PATH_REJECTED), "{finding:?}");
        assert_eq!(finding.severity, Severity::Info);
    }
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    assert!(
        !paths
            .iter()
            .any(|p| p.contains("escape") || p.contains("b?d")),
        "{paths:?}"
    );
}

#[test]
fn a_panicked_manifest_job_is_internal() {
    let mut responder = folder(Order::Issue);
    responder.panic_on(JobKind::Manifest);
    assert!(matches!(
        open_folder(&mut responder, OpenOptions::default()),
        Err(OpenError::Internal { .. })
    ));
}

#[cfg(feature = "zip")]
#[test]
fn a_panicked_archive_index_job_is_internal() {
    let zip = Pack::from_files(
        [(
            PackPath::from_static("manifest.json"),
            Bytes::from(manifest(2, "resources")),
        )],
        &OpenOptions::default(),
    )
    .unwrap()
    .to_archive()
    .unwrap();
    let mut responder = Responder::new(Order::Issue);
    responder.root_file(5, zip.clone()).panic_on(JobKind::Index);
    let mut loader = PackLoader::file(RootId(5), Some(zip.len() as u64), OpenOptions::default());
    let result = responder.drive(&mut loader);
    assert!(
        matches!(&result, Err(OpenError::Internal { location }) if *location == Place::root(RootId(5))),
        "{result:?}"
    );
}

#[test]
fn a_panicked_contents_index_job_is_an_internal_error_and_the_pack_opens() {
    let mut responder = folder(Order::Issue);
    responder
        .file(1, "contents.json", &br#"{"version":1}"#[..])
        .panic_on(JobKind::Index);
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    let codes: Vec<_> = pack
        .report()
        .diagnostics()
        .iter()
        .map(|d| &d.code)
        .collect();
    assert_eq!(codes, [&DiagnosticCode::from(&core_codes::INTERNAL_ERROR)]);
    // A plain `contents.json`: the files stay readable.
    assert!(pack.read(&PackPath::from_static("manifest.json")).is_ok());
}

#[test]
fn every_completion_order_opens_the_same_pack() {
    let reference = format!(
        "{:?}",
        open_folder(&mut folder(Order::Issue), OpenOptions::default())
            .map(|p| (p.report().clone(), p.paths().cloned().collect::<Vec<_>>()))
    );
    for order in Order::ALL {
        let mut responder = folder(order);
        responder.file(1, "contents.json", &br#"{"version":2}"#[..]);
        let mut with_contents = folder(Order::Issue);
        with_contents.file(1, "contents.json", &br#"{"version":2}"#[..]);
        let a = open_folder(&mut responder, OpenOptions::default())
            .map(|p| (p.report().clone(), p.paths().cloned().collect::<Vec<_>>()));
        let b = open_folder(&mut with_contents, OpenOptions::default())
            .map(|p| (p.report().clone(), p.paths().cloned().collect::<Vec<_>>()));
        assert_eq!(format!("{a:?}"), format!("{b:?}"), "{order:?}");
        let c = open_folder(&mut folder(order), OpenOptions::default())
            .map(|p| (p.report().clone(), p.paths().cloned().collect::<Vec<_>>()));
        assert_eq!(format!("{c:?}"), reference, "{order:?}");
    }
}

#[test]
fn a_dropped_loader_cancels_its_jobs() {
    let responder = folder(Order::Issue);
    let mut loader = PackLoader::directory(RootId(1), OpenOptions::default());
    let job = loop {
        match loader.poll() {
            Some(Output::Request(request)) => {
                let input = responder.respond(&request);
                loader.handle(input).unwrap();
            }
            Some(Output::Job(job)) => break job,
            other => panic!("{other:?}"),
        }
    };
    drop(loader);
    // The job returns early, without panicking; its result has nobody to go to.
    let done = job.run();
    let _ = done.id();
}

#[cfg(feature = "zip")]
#[test]
fn a_panicked_brarchive_index_job_of_an_archive_pack_names_each_archive() {
    let archive = brarchive::serialize([("pig.json", &b"{}"[..])]).unwrap();
    let zip = Pack::from_files(
        [
            (
                PackPath::from_static("manifest.json"),
                Bytes::from(manifest(2, "resources")),
            ),
            (
                PackPath::from_static("__brarchive/entities.brarchive"),
                Bytes::from(archive),
            ),
        ],
        &OpenOptions::default(),
    )
    .unwrap()
    .to_archive()
    .unwrap();
    let mut responder = Responder::new(Order::Issue);
    // The archive's own `Index` job runs; the brarchive's panics.
    responder
        .root_file(5, zip.clone())
        .panic_after(JobKind::Index, 1);
    let mut loader = PackLoader::file(RootId(5), Some(zip.len() as u64), OpenOptions::default());
    let pack = responder.drive(&mut loader).unwrap();
    let findings: Vec<_> = pack
        .report()
        .diagnostics()
        .iter()
        .map(|d| (&d.code, d.location.path.as_ref().map(PackPath::as_str)))
        .collect();
    assert_eq!(
        findings,
        [(
            &DiagnosticCode::from(&core_codes::INTERNAL_ERROR),
            Some("__brarchive/entities.brarchive")
        )]
    );
}

/// A PNG header of `width` by `height` (signature and `IHDR`; no image data).
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes
}

/// The icon findings of `pack`: code, severity, path.
fn icon_findings(pack: &Pack) -> Vec<(&'static str, Severity, String)> {
    pack.icon_findings()
        .iter()
        .map(|d| {
            (
                d.code.name(),
                d.severity,
                d.location.path.as_ref().unwrap().as_str().to_owned(),
            )
        })
        .collect()
}

/// A pack of the manifest and `icon`, every file resident.
fn pack_with_icon(icon: Option<Vec<u8>>) -> Pack {
    let mut files = vec![(
        PackPath::from_static("manifest.json"),
        manifest(1, "data").into_bytes(),
    )];
    files.extend(icon.map(|bytes| (PackPath::from_static("pack_icon.png"), bytes)));
    Pack::from_files(files, &OpenOptions::default()).unwrap()
}

// families/rp_misc fixtures pi01 to pi04, validation P1 to P3 (ours).
#[test]
fn pi01_to_pi04_the_pack_icon_lints() {
    assert_eq!(icon_findings(&pack_with_icon(Some(png(256, 256)))), []);
    // A square icon of any side passes: the rule is "square", not a power of two.
    assert_eq!(icon_findings(&pack_with_icon(Some(png(100, 100)))), []);
    let at = "pack_icon.png".to_owned();
    assert_eq!(
        icon_findings(&pack_with_icon(None)),
        [("PackIconMissing", Severity::Info, at.clone())]
    );
    let jpeg = b"\xff\xd8\xff\xe0\x00\x10JFIF".to_vec();
    assert_eq!(
        icon_findings(&pack_with_icon(Some(jpeg))),
        [("InvalidValue", Severity::Info, at.clone())]
    );
    let shape = pack_with_icon(Some(png(256, 128)));
    assert_eq!(
        icon_findings(&shape),
        [("PackIconShape", Severity::Info, at)]
    );
    assert_eq!(
        shape
            .icon_findings()
            .iter()
            .next()
            .unwrap()
            .code
            .to_string(),
        "pack_icon.png is not a square image"
    );
    // The signature alone is a PNG to `Pack::icon`, and no image to the lint.
    let truncated = pack_with_icon(Some(b"\x89PNG\r\n\x1a\n".to_vec()));
    assert!(truncated.icon().is_some());
    assert_eq!(icon_findings(&truncated)[0].0, "InvalidValue");
}

/// A folder pack opened by a loader has its files indexed, not read: `Pack::icon` is `None`
/// though the file is there. "Missing" is about the pack's paths, and an icon whose bytes
/// were never read is not judged.
#[test]
fn an_indexed_icon_is_not_missing_and_not_judged() {
    let mut responder = Responder::new(Order::Issue);
    responder.file(1, "manifest.json", manifest(1, "data"));
    responder.file(1, "pack_icon.png", png(256, 128));
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    assert!(pack.icon().is_none());
    assert_eq!(icon_findings(&pack), []);
    let mut responder = Responder::new(Order::Issue);
    responder.file(1, "manifest.json", manifest(1, "data"));
    let pack = open_folder(&mut responder, OpenOptions::default()).unwrap();
    assert_eq!(icon_findings(&pack).len(), 1);
    // A host that has the bytes lints them itself.
    assert_eq!(
        Pack::check_icon(&png(256, 128))
            .iter()
            .next()
            .unwrap()
            .code
            .name(),
        "PackIconShape"
    );
}

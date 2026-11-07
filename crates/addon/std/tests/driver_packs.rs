//! `Driver` with the real loaders: pack folders, archives, repositories and key sidecars on a
//! temporary disk tree and the storage fixtures of `bedrock_addon`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon::diagnostic::codes as core_codes;
use std::fs;
use std::path::Path;

use bedrock_addon::pack::{OpenError, PackOrigin};
use bedrock_addon_std::{DirSource, Driver, LoadError, Options, Strategy, load_dir};

/// A behaviour pack manifest with a fixed identity.
fn manifest(name: &str, uuid: &str) -> String {
    format!(
        r#"{{
    "format_version": 2,
    "header": {{
        "name": "{name}",
        "description": "",
        "uuid": "{uuid}",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 0]
    }},
    "modules": [{{ "type": "data", "uuid": "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40", "version": [1, 0, 0] }}]
}}"#
    )
}

const RUBY: &str = "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21";
const EMERALD: &str = "6f2a1b4d-4b72-4c2f-8f1f-4d2a6c1b7e32";

fn write(dir: &Path, path: &str, bytes: &[u8]) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// `crates/addon/storage/tests/fixtures/<path>`.
#[cfg(any(feature = "zip", feature = "encrypted"))]
fn storage_fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../storage/tests/fixtures")
        .join(path)
}

fn sequential() -> Options {
    Options::default().with_strategy(Strategy::Sequential)
}

#[test]
fn pack_dir_opens_a_pack_folder() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "ruby/manifest.json",
        manifest("ruby", RUBY).as_bytes(),
    );
    let mut driver = Driver::new(sequential()).unwrap();
    let pack = driver.pack_dir(dir.path().join("ruby")).unwrap();
    assert_eq!(pack.id().uuid.to_string(), RUBY);
    assert_eq!(
        driver.roots().host_path(pack.location()),
        Some(dir.path().join("ruby"))
    );
}

#[test]
fn a_folder_without_a_manifest_is_an_open_error_at_its_path() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "empty/readme.txt", b"no pack here");
    let mut driver = Driver::new(sequential()).unwrap();
    match driver.pack_dir(dir.path().join("empty")) {
        Err(LoadError::Open {
            path,
            source: OpenError::NoManifest { .. },
        }) => assert_eq!(path, dir.path().join("empty")),
        other => panic!("expected NoManifest, got {:?}", other.err()),
    }
}

#[test]
fn a_missing_folder_is_an_unreadable_open_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        load_dir(dir.path().join("missing"), &sequential()),
        Err(LoadError::Open {
            source: OpenError::Unreadable { .. },
            ..
        })
    ));
}

#[cfg(feature = "zip")]
#[test]
fn every_zip_fixture_opens_as_an_archive_and_the_strategies_agree() {
    // 15 fixtures are single packs; 8 hold several manifests at their shallowest depth, which
    // is `MultipleManifests`.
    let mut archives: Vec<std::path::PathBuf> = fs::read_dir(storage_fixture("zip_packs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    archives.sort();
    let open_all = |strategy| {
        let mut driver = Driver::new(Options::default().with_strategy(strategy)).unwrap();
        archives
            .iter()
            .map(|archive| match driver.pack_archive(archive) {
                Ok(pack) => Ok(format!("{:?} {:?}", pack.id(), pack.report())),
                Err(LoadError::Open {
                    path,
                    source: source @ OpenError::MultipleManifests { .. },
                }) => {
                    assert_eq!(&path, archive, "the error names the archive");
                    Err(format!("{source:?}"))
                }
                Err(error) => panic!("{}: {error:?}", archive.display()),
            })
            .collect::<Vec<_>>()
    };
    let sequential = open_all(Strategy::Sequential);
    assert_eq!(sequential.len(), 23);
    assert_eq!(
        sequential.iter().filter(|outcome| outcome.is_ok()).count(),
        15
    );
    #[cfg(feature = "rayon")]
    assert_eq!(open_all(Strategy::Rayon { threads: None }), sequential);
    let first_pack = sequential.iter().position(Result::is_ok).unwrap();
    let one_shot =
        bedrock_addon_std::load_archive(&archives[first_pack], &self::sequential()).unwrap();
    assert_eq!(
        Ok(format!("{:?} {:?}", one_shot.id(), one_shot.report())),
        sequential[first_pack]
    );
}

#[test]
fn repository_registers_the_packs_of_every_source_in_order() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "behavior_packs/ruby/manifest.json",
        manifest("ruby", RUBY).as_bytes(),
    );
    write(
        dir.path(),
        "behavior_packs/broken/readme.txt",
        b"no manifest",
    );
    write(
        dir.path(),
        "development_behavior_packs/emerald/manifest.json",
        manifest("emerald", EMERALD).as_bytes(),
    );
    let mut driver = Driver::new(Options::default()).unwrap();
    let repository = driver
        .repository([
            DirSource::packs(dir.path().join("behavior_packs"), PackOrigin::World),
            DirSource::packs(
                dir.path().join("development_behavior_packs"),
                PackOrigin::Dev,
            ),
            DirSource::packs(dir.path().join("missing_packs"), PackOrigin::World),
        ])
        .unwrap();
    let uuids: Vec<String> = repository
        .packs()
        .map(|pack| pack.id().uuid.to_string())
        .collect();
    assert_eq!(uuids, vec![RUBY.to_owned(), EMERALD.to_owned()]);
    assert_eq!(
        repository.invalid().len(),
        1,
        "the folder without a manifest"
    );
    assert_eq!(repository.source_reports().len(), 3);
}

#[cfg(feature = "encrypted")]
#[test]
fn an_encrypted_pack_folder_is_decrypted_with_its_sidecar_key_only_when_opted_in() {
    use bedrock_addon::diagnostic::DiagnosticCode;
    use bedrock_addon::pack::Pack;
    use bedrock_addon::storage::PackPath;

    /// Whether `contents.json` could not be decrypted (a warning on `contents.json`).
    fn contents_unsupported(pack: &Pack) -> bool {
        pack.report().diagnostics().iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::from(&core_codes::UNSUPPORTED_FILE_FORMAT)
                && diagnostic.location.path == Some(PackPath::from_static("contents.json"))
        })
    }

    const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("ruby");
    for file in ["contents.json", "blocks/ruby.json", "texts/ruby.lang"] {
        write(
            &pack,
            file,
            &fs::read(storage_fixture(&format!("encrypted_synthetic/{file}"))).unwrap(),
        );
    }
    write(&pack, "manifest.json", manifest("ruby", RUBY).as_bytes());
    fs::write(dir.path().join("ruby.key"), PACK_KEY).unwrap();

    let without = load_dir(&pack, &sequential()).unwrap();
    assert!(contents_unsupported(&without), "{:?}", without.report());

    let with = load_dir(&pack, &sequential().with_sidecar_keys(true)).unwrap();
    assert!(!contents_unsupported(&with), "{:?}", with.report());
}

/// Options whose capability `boom` has a validator that panics: a manifest asking for it
/// makes its `Manifest` job panic.
fn panicking_capability_options(strategy: Strategy) -> Options {
    use bedrock_addon::manifest::CapabilityRegistry;
    use bedrock_addon::pack::OpenOptions;

    let mut capabilities = CapabilityRegistry::default();
    capabilities.insert("boom", |_| panic!("a host validator panicked"));
    Options::default()
        .with_strategy(strategy)
        .with_open(OpenOptions {
            capabilities,
            ..OpenOptions::default()
        })
}

#[test]
fn a_panicking_job_is_fed_back_as_job_panicked_under_every_strategy() {
    let dir = tempfile::tempdir().unwrap();
    let asks_for_boom = manifest("boom", RUBY).replacen(
        "\"modules\"",
        "\"capabilities\": [\"boom\"],\n    \"modules\"",
        1,
    );
    write(dir.path(), "boom/manifest.json", asks_for_boom.as_bytes());
    write(
        dir.path(),
        "emerald/manifest.json",
        manifest("emerald", EMERALD).as_bytes(),
    );
    #[cfg_attr(not(feature = "rayon"), allow(unused_mut))]
    let mut strategies = vec![Strategy::Sequential];
    #[cfg(feature = "rayon")]
    strategies.push(Strategy::Rayon {
        threads: std::num::NonZeroUsize::new(2),
    });
    for strategy in strategies {
        let mut driver = Driver::new(panicking_capability_options(strategy.clone())).unwrap();
        // A panicked `Manifest` job leaves no pack and reports `OpenError::Internal`.
        match driver.pack_dir(dir.path().join("boom")) {
            Err(LoadError::Open {
                path,
                source: OpenError::Internal { .. },
            }) => assert_eq!(path, dir.path().join("boom"), "{strategy:?}"),
            other => panic!("{strategy:?}: expected Internal, got {:?}", other.err()),
        }
        // The driver and its pool survive the panic.
        let pack = driver.pack_dir(dir.path().join("emerald")).unwrap();
        assert_eq!(pack.id().uuid.to_string(), EMERALD, "{strategy:?}");
    }
}

const SAPPHIRE: &str = "7a3b2c5e-5c83-4d3a-9a2a-5e3b7d2c8f43";

#[test]
fn load_stacks_loads_a_composed_world_and_the_strategies_agree() {
    use bedrock_addon::load::{LoadOptions, StackOptions};
    use bedrock_addon::pack::WorldPacks;

    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "behavior_packs/ruby/manifest.json",
        manifest("ruby", RUBY).as_bytes(),
    );
    write(
        dir.path(),
        "behavior_packs/ruby/blocks/ruby.json",
        br#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:ruby"}, "components": {}}}"#,
    );
    write(
        dir.path(),
        "behavior_packs/emerald/manifest.json",
        manifest("emerald", EMERALD).as_bytes(),
    );
    let resource = manifest("sapphire", SAPPHIRE)
        .replace("\"type\": \"data\"", "\"type\": \"resources\"")
        .replace(
            "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40",
            "0b1e4d2f-4a8c-4d66-9b4a-6c9f7d2e3a51",
        );
    write(
        dir.path(),
        "resource_packs/sapphire/manifest.json",
        resource.as_bytes(),
    );
    write(
        dir.path(),
        "resource_packs/sapphire/texts/en_US.lang",
        b"a=b",
    );
    let world = WorldPacks::parse(
        Some(format!(r#"[{{ "pack_id": "{RUBY}", "version": [1, 0, 0] }}]"#).as_bytes()),
        Some(format!(r#"[{{ "pack_id": "{SAPPHIRE}", "version": [1, 0, 0] }}]"#).as_bytes()),
    )
    .unwrap();

    let load = |strategy| {
        let mut driver = Driver::new(Options::default().with_strategy(strategy)).unwrap();
        let repository = driver
            .repository([
                DirSource::packs(dir.path().join("behavior_packs"), PackOrigin::World),
                DirSource::packs(dir.path().join("resource_packs"), PackOrigin::World),
            ])
            .unwrap();
        let stacks = driver
            .load_stacks(
                repository.stacks(&world, LoadOptions::default()),
                StackOptions::default(),
            )
            .unwrap();
        let behavior: Vec<String> = stacks
            .behavior
            .packs()
            .map(|pack| pack.id().uuid.to_string())
            .collect();
        let resource: Vec<String> = stacks
            .resource
            .packs()
            .map(|pack| pack.id().uuid.to_string())
            .collect();
        assert_eq!(behavior, vec![RUBY.to_owned()], "only the selected pack");
        assert_eq!(resource, vec![SAPPHIRE.to_owned()]);
        let report = stacks.load_all();
        assert!(!report.has_errors(), "{report:?}");
        let report = format!("{report:?}");
        (format!("{stacks:?}"), report)
    };
    #[cfg_attr(not(feature = "rayon"), allow(unused_variables))]
    let sequential = load(Strategy::Sequential);
    #[cfg(feature = "rayon")]
    assert_eq!(load(Strategy::Rayon { threads: None }), sequential);
}

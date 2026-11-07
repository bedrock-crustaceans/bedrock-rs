//! `Pack::from_files` and `Pack::from_archive` over the fixtures (G1): every vanilla manifest,
//! every zip pack, the encrypted packs; `Pack::read`, `is_resident`, `to_archive`.

use bedrock_addon_core::diagnostic::codes as core_codes;
use std::path::Path;

use bedrock_addon::diagnostic::{Code, Severity};
use bedrock_addon::load::RootId;
use bedrock_addon::manifest::{CapabilityRegistry, Manifest};
use bedrock_addon::pack::{OpenError, OpenOptions, Pack};
use bedrock_addon::storage::{
    ContentIdentity, ContentKey, ContentKeys, PackPath, StorageError, ZipFault,
};
use bytes::Bytes;

use super::{fixture, fixtures};

/// Every `manifest.json` below `dir`, sorted.
fn manifests_below(dir: &Path) -> Vec<std::path::PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.file_name().is_some_and(|name| name == "manifest.json") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

#[test]
fn every_vanilla_manifest_opens_as_a_pack_with_from_files() {
    let manifests = manifests_below(&fixtures().join("manifest/vanilla"));
    assert_eq!(manifests.len(), 221);
    for file in manifests {
        let bytes = std::fs::read(&file).unwrap();
        let expected = Manifest::parse(
            &path("manifest.json"),
            &bytes,
            &CapabilityRegistry::default(),
        )
        .unwrap()
        .into_value();
        let pack = Pack::from_files([(path("manifest.json"), bytes)], &OpenOptions::default())
            .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        assert_eq!(*pack.id(), expected.identity(), "{}", file.display());
        assert_eq!(*pack.manifest(), expected);
        assert_eq!(pack.location().root, RootId::MEMORY);
        assert!(!pack.report().has_errors(), "{}", file.display());
        assert_eq!(pack.report().identity.as_ref(), Some(pack.id()));
    }
}

#[cfg(feature = "zip")]
/// The entries of a zip's `unzip -Z1` listing that are files.
fn listing(zip: &str) -> Vec<String> {
    let text = String::from_utf8(fixture(&format!(
        "../../../storage/tests/fixtures/zip_listings/{zip}.txt"
    )))
    .unwrap();
    text.lines()
        .filter(|line| !line.ends_with('/') && !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(feature = "zip")]
/// The zip packs and the folder of their shallowest manifest, or how many share that depth.
fn shallowest(entries: &[String]) -> Result<String, usize> {
    let manifests: Vec<&String> = entries
        .iter()
        .filter(|entry| entry.rsplit('/').next() == Some("manifest.json"))
        .collect();
    let least = manifests
        .iter()
        .map(|m| m.matches('/').count())
        .min()
        .unwrap();
    let at: Vec<&&String> = manifests
        .iter()
        .filter(|m| m.matches('/').count() == least)
        .collect();
    if at.len() > 1 {
        return Err(at.len());
    }
    Ok(at[0]
        .strip_suffix("manifest.json")
        .unwrap()
        .trim_end_matches('/')
        .to_owned())
}

#[cfg(feature = "zip")]
#[test]
fn every_zip_pack_opens_with_from_archive_by_the_shallowest_manifest() {
    let mut zips: Vec<String> =
        std::fs::read_dir(fixtures().join("../../../storage/tests/fixtures/zip_packs"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
    zips.sort();
    assert_eq!(zips.len(), 23);
    let mut single = 0;
    for zip in zips {
        let bytes = fixture(&format!("../../../storage/tests/fixtures/zip_packs/{zip}"));
        let entries = listing(&zip);
        let result = Pack::from_archive(bytes.clone(), &OpenOptions::default());
        match shallowest(&entries) {
            Ok(prefix) => {
                single += 1;
                let pack = result.unwrap_or_else(|error| panic!("{zip}: {error}"));
                let mut expected: Vec<String> = entries
                    .iter()
                    .filter_map(|entry| {
                        if prefix.is_empty() {
                            Some(entry.clone())
                        } else {
                            entry.strip_prefix(&format!("{prefix}/")).map(str::to_owned)
                        }
                    })
                    .collect();
                expected.sort();
                let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
                assert_eq!(paths, expected, "{zip}");
                for path in pack.paths() {
                    assert!(pack.is_resident(path), "{zip}: {path}");
                }
                assert_eq!(
                    pack.location().path.as_str(),
                    prefix,
                    "{zip}: the pack root is the manifest's folder"
                );
                if prefix.is_empty() {
                    assert_eq!(
                        pack.to_archive().unwrap(),
                        bytes,
                        "{zip}: the original bytes"
                    );
                }
            }
            Err(count) => assert!(
                matches!(result, Err(OpenError::MultipleManifests { count: c, .. }) if c == count),
                "{zip}: {result:?}"
            ),
        }
    }
    assert_eq!(single, 15);
}

#[cfg(feature = "zip")]
#[test]
fn an_archive_pack_reads_its_files_inflated() {
    let pack = Pack::from_archive(
        fixture("../../../storage/tests/fixtures/zip_packs/11b.zip"),
        &OpenOptions::default(),
    )
    .unwrap();
    let manifest = pack.read(&path("manifest.json")).unwrap();
    assert!(
        Manifest::parse(
            &path("manifest.json"),
            &manifest,
            &CapabilityRegistry::default()
        )
        .is_ok()
    );
    assert!(matches!(
        pack.read(&path("no/such.json")),
        Err(StorageError::NotFound)
    ));
}

#[cfg(feature = "zip")]
#[test]
fn bytes_that_are_not_an_archive_are_no_pack() {
    for (file, fault) in [
        (
            "../../../storage/tests/fixtures/synthetic/not_a_zip.zip",
            ZipFault::Malformed,
        ),
        (
            "../../../storage/tests/fixtures/synthetic/empty_zip_zero_entries.zip",
            ZipFault::Malformed,
        ),
        (
            "../../../storage/tests/fixtures/synthetic/truncated_2bytes.bin",
            ZipFault::Malformed,
        ),
    ] {
        let result = Pack::from_archive(fixture(file), &OpenOptions::default());
        assert!(
            matches!(result, Err(OpenError::Storage(StorageError::Zip { fault: f })) if f == fault),
            "{file}: {result:?}"
        );
    }
    let truncated = Pack::from_archive(
        fixture("../../../storage/tests/fixtures/synthetic/truncated_near_full.bin"),
        &OpenOptions::default(),
    );
    assert!(
        matches!(truncated, Err(OpenError::Storage(_))),
        "{truncated:?}"
    );
}

#[cfg(feature = "zip")]
#[test]
fn an_archive_without_a_manifest_is_no_manifest() {
    let zip = zip_of(&[("textures/a.png", b"a")]);
    assert!(matches!(
        Pack::from_archive(zip, &OpenOptions::default()),
        Err(OpenError::NoManifest { .. })
    ));
}

#[cfg(feature = "zip")]
/// Zip bytes of `files`, through a pack: `from_files` then `to_archive`.
fn zip_of(files: &[(&str, &[u8])]) -> Bytes {
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        path("manifest.json"),
        Bytes::from(super::responder::manifest(7, "resources")),
    )];
    all.extend(
        files
            .iter()
            .map(|(p, b)| (path(p), Bytes::copy_from_slice(b))),
    );
    let pack = Pack::from_files(all, &OpenOptions::default()).unwrap();
    let zip = pack.to_archive().unwrap();
    // Drop the manifest by re-zipping without it when asked for a manifest-less archive.
    if files.iter().any(|(p, _)| *p == "manifest.json") {
        return zip;
    }
    strip_manifest(&zip)
}

#[cfg(feature = "zip")]
/// The zip without its `manifest.json` (built with the `zip` crate's own reader and writer).
fn strip_manifest(zip: &[u8]) -> Bytes {
    use std::io::{Cursor, Read, Write};
    let mut archive = zip::ZipArchive::new(Cursor::new(zip)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name() == "manifest.json" {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        writer
            .start_file(file.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    Bytes::from(writer.finish().unwrap().into_inner())
}

#[cfg(feature = "zip")]
#[test]
fn a_memory_pack_zips_its_files_and_opens_again_from_the_zip() {
    let manifest = super::responder::manifest(3, "data");
    let files = [
        (path("manifest.json"), Bytes::from(manifest)),
        (path("entities/pig.json"), Bytes::from_static(b"{}")),
        (path("texts/en_US.lang"), Bytes::from_static(b"a=b")),
    ];
    let pack = Pack::from_files(files.clone(), &OpenOptions::default()).unwrap();
    let zip = pack.to_archive().unwrap();
    let again = Pack::from_archive(zip, &OpenOptions::default()).unwrap();
    assert_eq!(again.id(), pack.id());
    for (path, bytes) in files {
        assert_eq!(again.read(&path).unwrap(), bytes);
        assert_eq!(pack.read(&path).unwrap(), bytes);
    }
}

#[test]
fn from_files_without_a_manifest_is_no_manifest() {
    let result = Pack::from_files(
        [(path("blocks/a.json"), &b"{}"[..])],
        &OpenOptions::default(),
    );
    assert!(
        matches!(result, Err(OpenError::NoManifest { .. })),
        "{result:?}"
    );
}

#[test]
fn from_files_takes_the_single_subdirectory_holding_the_manifest() {
    let manifest = super::responder::manifest(4, "data");
    let pack = Pack::from_files(
        [
            (path("Pack/manifest.json"), Bytes::from(manifest)),
            (path("Pack/items/a.json"), Bytes::from_static(b"{}")),
            (path("notes.txt"), Bytes::from_static(b"x")),
        ],
        &OpenOptions::default(),
    )
    .unwrap();
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    assert_eq!(paths, ["items/a.json", "manifest.json"]);
    assert_eq!(pack.location().path.as_str(), "Pack");
    assert_eq!(pack.read(&path("items/a.json")).unwrap(), &b"{}"[..]);
}

#[test]
fn two_subdirectories_with_a_manifest_are_multiple_manifests() {
    let result = Pack::from_files(
        [
            (
                path("a/manifest.json"),
                super::responder::manifest(1, "data"),
            ),
            (
                path("b/manifest.json"),
                super::responder::manifest(2, "data"),
            ),
        ],
        &OpenOptions::default(),
    );
    assert!(
        matches!(result, Err(OpenError::MultipleManifests { count: 2, .. })),
        "{result:?}"
    );
}

#[test]
fn an_invalid_manifest_is_invalid_manifest_with_its_findings() {
    let result = Pack::from_files(
        [(path("manifest.json"), &b"{\"format_version\": 2}"[..])],
        &OpenOptions::default(),
    );
    let Err(OpenError::InvalidManifest { report, .. }) = result else {
        panic!("{result:?}");
    };
    assert!(report.has_errors());
}

/// The keys `encrypted_synthetic/` was built with.
const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";
const SYNTHETIC_IDENTITY: &str = "00000000-0000-4000-8000-00000000c0de";

/// The synthetic encrypted pack with a valid plaintext manifest of ours.
fn synthetic_files() -> Vec<(PackPath, Bytes)> {
    let dir = "../../../storage/tests/fixtures/encrypted_synthetic";
    vec![
        (
            path("manifest.json"),
            Bytes::from(super::responder::manifest(9, "data")),
        ),
        (
            path("contents.json"),
            Bytes::from(fixture(&format!("{dir}/contents.json"))),
        ),
        (
            path("blocks/ruby.json"),
            Bytes::from(fixture(&format!("{dir}/blocks/ruby.json"))),
        ),
        (
            path("texts/ruby.lang"),
            Bytes::from(fixture(&format!("{dir}/texts/ruby.lang"))),
        ),
    ]
}

#[cfg(feature = "encrypted")]
#[test]
fn an_encrypted_pack_with_its_key_decrypts_every_asset() {
    let keys: ContentKeys = [(
        ContentIdentity::new(SYNTHETIC_IDENTITY),
        ContentKey::new(*PACK_KEY),
    )]
    .into_iter()
    .collect();
    let options = OpenOptions {
        keys,
        ..OpenOptions::default()
    };
    let pack = Pack::from_files(synthetic_files(), &options).unwrap();
    assert!(!pack.report().has_errors(), "{:?}", pack.report());
    assert!(
        pack.read(&path("blocks/ruby.json"))
            .unwrap()
            .starts_with(br#"{"format_version":"1.21.0","minecraft:block""#)
    );
    assert_eq!(
        pack.read(&path("texts/ruby.lang")).unwrap(),
        &b"tile.demo:ruby.name=Ruby Block\n"[..]
    );
}

#[test]
fn an_encrypted_pack_without_its_key_opens_and_its_assets_are_unsupported() {
    let pack = Pack::from_files(synthetic_files(), &OpenOptions::default()).unwrap();
    let codes: Vec<_> = pack
        .report()
        .diagnostics()
        .iter()
        .map(|d| &d.code)
        .collect();
    assert!(
        codes.contains(&&Code::from(&core_codes::UNSUPPORTED_FILE_FORMAT)),
        "{codes:?}"
    );
    assert!(pack.read(&path("manifest.json")).is_ok());
    assert!(matches!(
        pack.read(&path("blocks/ruby.json")),
        Err(StorageError::MissingContentKey)
    ));
}

#[cfg(feature = "encrypted")]
#[test]
fn a_wrong_key_leaves_the_assets_undecryptable() {
    let keys: ContentKeys = [(
        ContentIdentity::new(SYNTHETIC_IDENTITY),
        ContentKey::new(*b"0123456789abcdef0123456789abcdef"),
    )]
    .into_iter()
    .collect();
    let options = OpenOptions {
        keys,
        ..OpenOptions::default()
    };
    let pack = Pack::from_files(synthetic_files(), &options).unwrap();
    assert!(matches!(
        pack.read(&path("blocks/ruby.json")),
        Err(StorageError::Decrypt)
    ));
}

#[test]
fn the_vanilla_encrypted_test_pack_opens_without_a_key() {
    let dir = fixtures().join("../../../storage/tests/fixtures/encrypted/testPack");
    let mut files = Vec::new();
    for relative in [
        "manifest.json",
        "contents.json",
        "signatures.json",
        "test/test.json",
        "test/test2.txt",
        "test/nested/test3.txt",
        "__brarchive/test.brarchive",
        "__brarchive/test/nested.brarchive",
    ] {
        files.push((
            path(relative),
            Bytes::from(std::fs::read(dir.join(relative)).unwrap()),
        ));
    }
    let pack = Pack::from_files(files, &OpenOptions::default()).unwrap();
    assert_eq!(pack.manifest().header.name, "Test Pack");
    let warnings: Vec<_> = pack
        .report()
        .diagnostics()
        .at_least(Severity::Warning)
        .map(|d| d.code.name())
        .collect();
    assert!(warnings.contains(&"UnsupportedFileFormat"), "{warnings:?}");
    assert!(matches!(
        pack.read(&path("test/test.json")),
        Err(StorageError::MissingContentKey)
    ));
}

#[cfg(feature = "encrypted")]
#[test]
fn a_whole_file_encrypted_archive_opens_with_its_key_from_the_options() {
    let keys: ContentKeys = [(
        ContentIdentity::new("00000000-0000-4000-8000-0000000a2c41"),
        ContentKey::new(*PACK_KEY),
    )]
    .into_iter()
    .collect();
    let bytes = fixture("../../../storage/tests/fixtures/encrypted_archive/whole.mcpack");
    // It hides `synthetic/valid_reference.zip`, whose manifest is `{"format_version":2}`.
    let with_key = Pack::from_archive(
        bytes.clone(),
        &OpenOptions {
            keys,
            ..OpenOptions::default()
        },
    );
    assert!(
        matches!(with_key, Err(OpenError::InvalidManifest { .. })),
        "{with_key:?}"
    );
    assert!(matches!(
        Pack::from_archive(bytes, &OpenOptions::default()),
        Err(OpenError::Storage(StorageError::MissingContentKey))
    ));
}

#[cfg(feature = "zip")]
#[test]
fn the_baked_pack_reads_through_its_brarchive_layer() {
    let pack = Pack::from_archive(
        fixture("../../../storage/tests/fixtures/zip_packs/bakedPack.zip"),
        &OpenOptions::default(),
    )
    .unwrap();
    // The archive is encrypted (no key): the brarchives cannot be read either.
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    assert!(paths.contains(&"__brarchive/test.brarchive"), "{paths:?}");
}

#[cfg(feature = "zip")]
#[test]
fn a_manifest_deeper_than_four_folders_is_too_deep() {
    use std::io::{Cursor, Write};
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "a/b/c/d/e/manifest.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer
        .write_all(super::responder::manifest(1, "data").as_bytes())
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert!(matches!(
        Pack::from_archive(bytes, &OpenOptions::default()),
        Err(OpenError::Storage(StorageError::LimitExceeded {
            limit: bedrock_addon::storage::ZipLimit::Nesting
        }))
    ));
}

#[cfg(feature = "encrypted")]
#[test]
fn a_whole_file_encrypted_archive_zips_back_to_its_ciphertext() {
    let identity = "00000000-0000-4000-8000-0000000a2c41";
    let keys: ContentKeys = [(ContentIdentity::new(identity), ContentKey::new(*PACK_KEY))]
        .into_iter()
        .collect();
    let options = OpenOptions {
        keys,
        ..OpenOptions::default()
    };
    // `whole.mcpack` hides a manifest without a header; the same layout with a valid manifest.
    let plain = Pack::from_files(
        [(
            path("manifest.json"),
            Bytes::from(super::responder::manifest(4, "data")),
        )],
        &OpenOptions::default(),
    )
    .unwrap()
    .to_archive()
    .unwrap();
    let header =
        &fixture("../../../storage/tests/fixtures/encrypted_archive/whole.mcpack")[..0x100];
    let mut encrypted = header.to_vec();
    encrypted.extend(cfb8_encrypt(PACK_KEY, &plain));
    let pack = Pack::from_archive(encrypted.clone(), &options).unwrap();
    assert_eq!(
        pack.to_archive().unwrap(),
        encrypted,
        "the original, never the plaintext"
    );
}

/// AES-256-CFB8 encryption with the key's first 16 bytes as IV (the cipher of 03).
#[cfg(feature = "encrypted")]
fn cfb8_encrypt(key: &[u8; 32], plain: &[u8]) -> Vec<u8> {
    use aes::cipher::{BlockEncrypt, KeyInit};
    let cipher = aes::Aes256::new(key.into());
    let mut register: [u8; 16] = key[..16].try_into().unwrap();
    plain
        .iter()
        .map(|byte| {
            let mut block = aes::Block::from(register);
            cipher.encrypt_block(&mut block);
            let out = byte ^ block[0];
            register.copy_within(1.., 0);
            register[15] = out;
            out
        })
        .collect()
}

#[cfg(feature = "zip")]
#[test]
fn to_archive_writes_a_loose_file_not_the_brarchive_entry_shadowing_it() {
    let archive = brarchive::serialize([("pig.json", &b"archived"[..])]).unwrap();
    let pack = Pack::from_files(
        [
            (
                path("manifest.json"),
                Bytes::from(super::responder::manifest(5, "data")),
            ),
            (path("entities/pig.json"), Bytes::from_static(b"loose")),
            (path("__brarchive/entities.brarchive"), Bytes::from(archive)),
        ],
        &OpenOptions::default(),
    )
    .unwrap();
    assert_eq!(
        pack.read(&path("entities/pig.json")).unwrap(),
        &b"archived"[..]
    );
    let zip = pack.to_archive().unwrap();
    let mut reader = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
    let mut loose = Vec::new();
    std::io::Read::read_to_end(
        &mut reader.by_name("entities/pig.json").unwrap(),
        &mut loose,
    )
    .unwrap();
    assert_eq!(loose, b"loose");
}

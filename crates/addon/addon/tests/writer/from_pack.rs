//! `PackWriter::from_pack` over the fixture packs: every file copied byte for byte, the
//! manifest by value in its format, and what a copy leaves out.

use bedrock_addon::manifest::{Manifest, ManifestFormat};
use bedrock_addon::pack::{OpenOptions, Pack, PackWriter, WriteError};
use bedrock_addon::storage::{ContentIdentity, ContentKey, ContentKeys};
use bytes::Bytes;

use super::packs::{Case, brarchive_folder, folder_with_subpacks, plain_packs, v3_folder};
use super::{bytes_at, fixture, fixtures, listed, parsed, path, paths, ruby_manifest};

/// The manifest a copy's `manifest.json` must read as: the fixture's, which a format-0 file
/// (written back as format 1, the format vanilla upgrades it to) does not say it was.
pub fn as_rewritten(manifest: &Manifest) -> Manifest {
    let mut manifest = manifest.clone();
    if manifest.original_format == ManifestFormat::V0 {
        manifest.original_format = ManifestFormat::V1;
    }
    manifest
}

/// The writer holds `case`'s manifest by value, in its format, and its files byte for byte.
pub fn assert_copy(case: &Case, writer: &PackWriter) {
    let name = &case.name;
    let mut files = listed(writer);
    let (first, manifest) = files.remove(0);
    assert_eq!(first, "manifest.json", "{name}");
    let manifest = parsed(&manifest);
    assert_eq!(manifest.format, case.manifest.format, "{name}: the format");
    assert_eq!(manifest, as_rewritten(&case.manifest), "{name}: the value");
    let listed: Vec<&String> = files.iter().map(|(at, _)| at).collect();
    let expected: Vec<&String> = case.files.iter().map(|(at, _)| at).collect();
    assert_eq!(listed, expected, "{name}: the file set");
    for ((at, bytes), (_, expected)) in files.iter().zip(&case.files) {
        assert!(bytes == expected, "{name}: the bytes of {at}");
    }
}

#[test]
fn from_pack_copies_every_fixture_pack_byte_for_byte_and_its_manifest_by_value() {
    let cases = plain_packs();
    let mut formats = std::collections::BTreeSet::new();
    for case in &cases {
        let writer = PackWriter::from_pack(&case.pack)
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        assert_copy(case, &writer);
        formats.insert(case.manifest.format);
    }
    // Formats 1, 2 and 3 are all among the fixtures.
    assert_eq!(
        formats.into_iter().collect::<Vec<_>>(),
        [ManifestFormat::V1, ManifestFormat::V2, ManifestFormat::V3]
    );
}

#[test]
fn a_copy_leaves_out_contents_json_and_signatures_json_of_the_pack_root() {
    let case = folder_with_subpacks();
    for listed in ["contents.json", "signatures.json"] {
        assert!(case.pack.paths().any(|at| at.as_str() == listed));
    }
    let writer = PackWriter::from_pack(&case.pack).unwrap();
    // They list and sign bytes the writer is there to change; a file of the same name in a
    // subpack folder is a file like any other.
    assert_eq!(
        paths(&writer),
        [
            "manifest.json",
            "subpacks/low/contents.json",
            "subpacks/low/textures/a.png",
            "textures/a.png",
        ]
    );
    assert_copy(&case, &writer);
}

#[test]
fn a_brarchive_layer_is_copied_as_loose_files_and_its_archive_is_left_out() {
    let case = brarchive_folder();
    let writer = PackWriter::from_pack(&case.pack).unwrap();
    assert_eq!(
        paths(&writer),
        [
            "manifest.json",
            "blocks/main.js",
            "blocks/ruby.json",
            "blocks/sapphire.json",
            "textures/a.png",
        ]
    );
    // What the pack reads at each path, the layer before the loose file, an empty entry
    // falling through.
    for (at, _) in listed(&writer).iter().skip(1) {
        assert_eq!(
            bytes_at(&writer, at).unwrap(),
            case.pack.read(&path(at)).unwrap(),
            "{at}"
        );
    }
    assert_eq!(
        bytes_at(&writer, "blocks/main.js").unwrap(),
        b"loose script"
    );
    assert_ne!(bytes_at(&writer, "blocks/ruby.json").unwrap(), b"shadowed");
    assert_copy(&case, &writer);
}

/// An archive of the layer without an entry serves no file, so the pack reads nothing through
/// it: a copy leaves it out as it leaves out every archive of the layer, and holds every file
/// the pack reads. Nothing is lost but an empty container.
#[test]
fn an_archive_of_the_layer_without_an_entry_is_left_out_and_nothing_is_lost() {
    let empty = brarchive::serialize(std::iter::empty::<(&str, &[u8])>()).unwrap();
    let full = brarchive::serialize([("ruby.json", &b"{}"[..])]).unwrap();
    let pack = Pack::from_files(
        [
            (
                path("manifest.json"),
                Bytes::from(ruby_manifest().to_json().unwrap()),
            ),
            (path("__brarchive/empty.brarchive"), Bytes::from(empty)),
            (path("__brarchive/blocks.brarchive"), Bytes::from(full)),
            (path("blocks/loose.json"), Bytes::from_static(b"[]")),
        ],
        &OpenOptions::default(),
    )
    .unwrap();
    let writer = PackWriter::from_pack(&pack).unwrap();
    assert_eq!(
        paths(&writer),
        ["manifest.json", "blocks/loose.json", "blocks/ruby.json"]
    );
    // Every file the pack reads, but the two containers, is in the copy with its bytes.
    let read: Vec<String> = pack
        .paths()
        .map(ToString::to_string)
        .filter(|at| !at.ends_with(".brarchive") && at != "manifest.json")
        .collect();
    assert_eq!(read, ["blocks/loose.json", "blocks/ruby.json"]);
    for at in &read {
        assert_eq!(
            bytes_at(&writer, at).unwrap(),
            pack.read(&path(at)).unwrap(),
            "{at}"
        );
    }
    // The pack the copy opens as reads the same files.
    let copy = writer
        .into_pack(bedrock_addon::pack::PackOrigin::World)
        .unwrap();
    let copied: Vec<String> = copy
        .paths()
        .map(ToString::to_string)
        .filter(|at| at != "manifest.json")
        .collect();
    assert_eq!(copied, read);
}

/// An MCB document: the header for `key` at `version`, then `body`.
#[cfg(feature = "voxel_shape")]
fn mcb(key: &str, version: (u16, u16, u32), body: &[u8]) -> Vec<u8> {
    let mut bytes = b"\x7FMCB".to_vec();
    bytes.extend(version.0.to_le_bytes());
    bytes.extend(version.1.to_le_bytes());
    bytes.extend(version.2.to_le_bytes());
    bytes.push(u8::try_from(key.len()).unwrap());
    bytes.extend(key.as_bytes());
    bytes.extend(body);
    bytes
}

/// The body of a baked voxel shape `demo:step` with one box.
#[cfg(feature = "voxel_shape")]
fn step_shape() -> Vec<u8> {
    let mut body = vec![9];
    body.extend(b"demo:step");
    body.push(1);
    for value in [0.0_f32, 0.0, 0.0, 16.0, 8.0, 16.0] {
        body.extend(value.to_le_bytes());
    }
    mcb("minecraft:voxel_shape", (1, 21, 110), &body)
}

/// MCB documents are written back as JSON text: a document whose body the
/// decoder reads is written as that text at its path (loose, or out of a brarchive layer),
/// and the family reads it. A document it does not read has no text: its bytes are copied.
#[cfg(feature = "voxel_shape")]
#[test]
fn an_mcb_document_the_decoder_reads_is_written_as_json_text_and_any_other_as_its_bytes() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::voxel_shape::VoxelShape;
    use bedrock_addon::load::LoadContext;

    let archive = fixture("storage/brarchive/mcb.brarchive");
    let baked = brarchive::serialize([("step.json", step_shape())]).unwrap();
    let pack = Pack::from_files(
        [
            (
                path("manifest.json"),
                Bytes::from(ruby_manifest().to_json().unwrap()),
            ),
            (path("__brarchive/items.brarchive"), Bytes::from(archive)),
            (path("__brarchive/shapes.brarchive"), Bytes::from(baked)),
            (
                path("cameras/loose.json"),
                Bytes::from_static(b"\x7FMCB\x01\x00\x1a\x00\x1e\x00\x00\x00\x00"),
            ),
            (path("shapes/loose.json"), Bytes::from(step_shape())),
        ],
        &OpenOptions::default(),
    )
    .unwrap();
    let writer = PackWriter::from_pack(&pack).unwrap();
    assert_eq!(
        paths(&writer),
        [
            "manifest.json",
            "cameras/loose.json",
            "items/ruby.json",
            "items/sapphire.json",
            "shapes/loose.json",
            "shapes/step.json",
        ]
    );
    // The two shapes are of a layout the decoder reads: JSON text, floats as floats.
    let text = concat!(
        r#"{"format_version":"1.21.110","minecraft:voxel_shape":{"description":"#,
        r#"{"identifier":"demo:step"},"shape":{"boxes":["#,
        r#"{"min":[0.0,0.0,0.0],"max":[16.0,8.0,16.0]}]}}}"#
    );
    for at in ["shapes/loose.json", "shapes/step.json"] {
        let bytes = bytes_at(&writer, at).unwrap();
        assert_eq!(String::from_utf8_lossy(&bytes), text, "{at}");
        let read = VoxelShape::parse(&path(at), &bytes, &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{at}: {rejected:?}"));
        assert!(read.diagnostics().is_empty(), "{:?}", read.diagnostics());
        assert_eq!(read.value().description.identifier.as_str(), "demo:step");
    }
    // `mcb.brarchive` holds one binary document of no known layout and one JSON file beside
    // it; the loose camera is a header alone. Neither binary has a text: copied as it is.
    for (at, binary) in [
        ("cameras/loose.json", true),
        ("items/ruby.json", true),
        ("items/sapphire.json", false),
    ] {
        let bytes = bytes_at(&writer, at).unwrap();
        assert_eq!(bytes.starts_with(b"\x7FMCB"), binary, "{at}");
        assert_eq!(bytes, pack.read(&path(at)).unwrap(), "{at}");
    }
}

/// The keys `encrypted_synthetic/` and `encrypted_archive/whole.mcpack` were built with.
const PACK_KEY: &[u8; 32] = b"pack-key-for-the-synthetic-pack!";

fn keys(identity: &str) -> OpenOptions {
    OpenOptions {
        keys: [(ContentIdentity::new(identity), ContentKey::new(*PACK_KEY))]
            .into_iter()
            .collect::<ContentKeys>(),
        ..OpenOptions::default()
    }
}

/// The synthetic encrypted pack with a valid plaintext manifest.
fn synthetic_files() -> Vec<(bedrock_addon::storage::PackPath, Bytes)> {
    let dir = "storage/encrypted_synthetic";
    let mut files = vec![(
        path("manifest.json"),
        Bytes::from(ruby_manifest().to_json().unwrap()),
    )];
    for file in ["contents.json", "blocks/ruby.json", "texts/ruby.lang"] {
        files.push((path(file), Bytes::from(fixture(&format!("{dir}/{file}")))));
    }
    files
}

fn assert_encrypted(name: &str, pack: &Pack) {
    let refused = PackWriter::from_pack(pack);
    assert!(
        matches!(refused, Err(WriteError::Encrypted)),
        "{name}: {:?}",
        refused.map(|writer| paths(&writer))
    );
}

#[test]
fn an_encrypted_pack_without_its_key_is_not_copied() {
    let synthetic = Pack::from_files(synthetic_files(), &OpenOptions::default()).unwrap();
    assert_encrypted("encrypted_synthetic, no key", &synthetic);
    // The vanilla test pack: an encrypted `contents.json`, two brarchives.
    let dir = fixtures().join("storage/encrypted/testPack");
    let files: Vec<_> = [
        "manifest.json",
        "contents.json",
        "signatures.json",
        "test/test.json",
        "test/test2.txt",
        "test/nested/test3.txt",
        "__brarchive/test.brarchive",
        "__brarchive/test/nested.brarchive",
    ]
    .into_iter()
    .map(|file| {
        (
            path(file),
            Bytes::from(std::fs::read(dir.join(file)).unwrap()),
        )
    })
    .collect();
    let vanilla = Pack::from_files(files, &OpenOptions::default()).unwrap();
    assert_encrypted("encrypted/testPack", &vanilla);
}

/// With its key the pack reads in plain text, and is still not copied: a copy would be the
/// pack without its encryption, and content keys are never written.
#[cfg(feature = "encrypted")]
#[test]
fn an_encrypted_pack_with_its_key_is_not_copied_either() {
    let synthetic = Pack::from_files(
        synthetic_files(),
        &keys("00000000-0000-4000-8000-00000000c0de"),
    )
    .unwrap();
    assert!(synthetic.read(&path("texts/ruby.lang")).is_ok());
    assert_encrypted("encrypted_synthetic, with its key", &synthetic);
}

#[cfg(feature = "zip")]
#[test]
fn an_encrypted_archive_is_not_copied() {
    let baked = Pack::from_archive(
        fixture("storage/zip_packs/bakedPack.zip"),
        &OpenOptions::default(),
    )
    .unwrap();
    assert_encrypted("bakedPack.zip", &baked);
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

/// A stored zip of `members` (name, bytes).
#[cfg(feature = "encrypted")]
fn zip_of(members: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in members {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// `zip` behind the encryption header of `storage/encrypted_archive/whole.mcpack` (which
/// itself hides a manifest without a header, so no pack), and the options holding its key.
#[cfg(feature = "encrypted")]
fn whole_file_encrypted(zip: &[u8]) -> (Vec<u8>, OpenOptions) {
    let header = &fixture("storage/encrypted_archive/whole.mcpack")[..0x100];
    let mut bytes = header.to_vec();
    bytes.extend(cfb8_encrypt(PACK_KEY, zip));
    (bytes, keys("00000000-0000-4000-8000-0000000a2c41"))
}

/// A whole-file encrypted archive opened with its key holds plain files; a copy would write
/// them out.
#[cfg(feature = "encrypted")]
#[test]
fn a_whole_file_encrypted_archive_is_not_copied() {
    let manifest = ruby_manifest().to_json().unwrap();
    let zip = zip_of(&[
        ("manifest.json", manifest.as_bytes()),
        ("textures/a.png", b"\x89PNG"),
    ]);
    let (bytes, options) = whole_file_encrypted(&zip);
    let pack = Pack::from_archive(bytes, &options).unwrap();
    assert_eq!(pack.read(&path("textures/a.png")).unwrap(), &b"\x89PNG"[..]);
    assert_encrypted("a whole-file encrypted archive", &pack);
    // The same zip without the encryption is copied.
    let plain = Pack::from_archive(zip, &OpenOptions::default()).unwrap();
    assert_eq!(
        paths(&PackWriter::from_pack(&plain).unwrap()),
        ["manifest.json", "textures/a.png"]
    );
}

/// The packs of a whole-file encrypted composite have no ciphertext of their own, and their
/// files are plain: not copied either.
#[cfg(feature = "encrypted")]
#[test]
fn the_packs_of_a_whole_file_encrypted_composite_are_not_copied() {
    use bedrock_addon::load::{RepositoryLoader, RootId, Source};
    use bedrock_addon::pack::PackOrigin;

    use super::responder::{Order, Responder, manifest};

    let nested = zip_of(&[("manifest.json", manifest(8, "resources").as_bytes())]);
    let outer = zip_of(&[
        ("bp/manifest.json", manifest(7, "data").as_bytes()),
        ("rp.mcpack", &nested),
    ]);
    let (bytes, options) = whole_file_encrypted(&outer);
    let mut responder = Responder::new(Order::Issue);
    responder.root_file(4, bytes);
    let source = Source::Archive {
        root: RootId(4),
        origin: PackOrigin::World,
    };
    let mut loader = RepositoryLoader::new(vec![source], options);
    let repository = responder.drive(&mut loader);
    assert_eq!(repository.packs().count(), 2, "{:?}", repository.invalid());
    for pack in repository.packs() {
        assert_encrypted(&pack.location().to_string(), pack);
    }
}

#[test]
fn a_copy_is_changed_like_any_writer() {
    let case = v3_folder();
    let mut writer = PackWriter::from_pack(&case.pack).unwrap();
    let (first, _) = &case.files[0];
    assert!(writer.remove(&path(first)));
    writer
        .put_bytes(path("textures/new.png"), b"new".to_vec())
        .unwrap();
    writer.manifest_mut().header.name = "renamed".to_owned();
    let manifest = parsed(&bytes_at(&writer, "manifest.json").unwrap());
    assert_eq!(manifest.header.name, "renamed");
    assert_eq!(manifest.format, ManifestFormat::V3);
    assert!(bytes_at(&writer, first).is_none());
    assert_eq!(bytes_at(&writer, "textures/new.png").unwrap(), b"new");
    // The pack it was copied from is as it was.
    assert_eq!(case.pack.manifest().header.name, case.manifest.header.name);
    assert!(case.pack.read(&path(first)).is_ok());
}

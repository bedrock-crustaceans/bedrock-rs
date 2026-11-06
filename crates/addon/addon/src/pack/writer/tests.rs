//! The sides of `PackWriter` that need a pack no public constructor builds (a file indexed and
//! not loaded, an archive outside the layer), and its cost.

use bedrock_addon_core::pack_identity::{PackOrigin, PackPath};

use super::{PackWriter, WriteError};
use crate::pack::tests::pack_from;
use crate::storage::StorageError;

/// A format-2 resource pack manifest.
const MANIFEST: &str = r#"{
  "format_version": 2,
  "header": {
    "name": "ruby",
    "uuid": "00000000-0000-4000-8000-000000000001",
    "version": [1, 0, 0],
    "min_engine_version": [1, 21, 0]
  },
  "modules": [
    { "type": "resources", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] }
  ]
}"#;

fn paths(writer: &PackWriter) -> Vec<&str> {
    writer.files().map(|(path, _)| path.as_str()).collect()
}

#[test]
fn a_file_that_is_indexed_and_not_resident_stops_the_copy() {
    let resident = pack_from(
        MANIFEST,
        &[
            ("textures/a.png", Some(b"a")),
            ("textures/b.png", Some(b"b")),
        ],
        PackOrigin::World,
    );
    let writer = PackWriter::from_pack(&resident).unwrap();
    assert_eq!(
        paths(&writer),
        ["manifest.json", "textures/a.png", "textures/b.png"]
    );
    // One file of the two only indexed: no copy, rather than a copy without it.
    let indexed = pack_from(
        MANIFEST,
        &[("textures/a.png", Some(b"a")), ("textures/b.png", None)],
        PackOrigin::World,
    );
    let refused = PackWriter::from_pack(&indexed);
    assert!(
        matches!(refused, Err(WriteError::Storage(StorageError::NotResident))),
        "{:?}",
        refused.map(|writer| paths(&writer).len())
    );
}

#[test]
fn the_files_a_copy_leaves_out_need_not_be_resident() {
    let pack = pack_from(
        MANIFEST,
        &[
            ("contents.json", None),
            ("signatures.json", None),
            ("textures/a.png", Some(b"a")),
        ],
        PackOrigin::World,
    );
    let writer = PackWriter::from_pack(&pack).unwrap();
    assert_eq!(paths(&writer), ["manifest.json", "textures/a.png"]);
}

/// `ruby.json` (88 bytes), `main.js` (empty), `sapphire.json` (39 bytes).
const SMALL: &[u8] = include_bytes!("../../../../storage/tests/fixtures/brarchive/small.brarchive");

#[test]
fn an_archive_the_pack_does_not_read_through_is_copied_as_a_file() {
    // Resident, and never read into a layer: the pack serves no `blocks/ruby.json`, so the
    // archive is all there is of those files and a copy keeps it, byte for byte.
    let unread = pack_from(
        MANIFEST,
        &[("__brarchive/blocks.brarchive", Some(SMALL))],
        PackOrigin::World,
    );
    assert!(
        unread
            .read(&PackPath::from_static("blocks/ruby.json"))
            .is_err()
    );
    let writer = PackWriter::from_pack(&unread).unwrap();
    let files: Vec<(&str, &[u8])> = writer
        .files()
        .skip(1)
        .map(|(path, bytes)| (path.as_str(), bytes))
        .collect();
    assert_eq!(files, [("__brarchive/blocks.brarchive", SMALL)]);
    // A file in the archive folder that is no archive, and an archive at a path that stands
    // for no folder: files like any other.
    let odd = pack_from(
        MANIFEST,
        &[
            ("__brarchive/blocks.brarchive", Some(b"not an archive")),
            ("textures/__brarchive.brarchive", Some(SMALL)),
        ],
        PackOrigin::World,
    );
    let writer = PackWriter::from_pack(&odd).unwrap();
    assert_eq!(
        paths(&writer),
        [
            "manifest.json",
            "__brarchive/blocks.brarchive",
            "textures/__brarchive.brarchive"
        ]
    );
}

#[test]
fn a_zip_error_names_what_the_archive_writer_refused() {
    use std::error::Error;

    use crate::storage::ZipFault;

    let error = WriteError::Zip {
        source: StorageError::Zip {
            fault: ZipFault::Unsupported,
        },
    };
    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<StorageError>());
    assert!(
        matches!(
            source,
            Some(StorageError::Zip {
                fault: ZipFault::Unsupported
            })
        ),
        "{source:?}"
    );
}

/// A writer with `n` small files in a few hundred folders.
fn writer_of(n: usize) -> PackWriter {
    let manifest = crate::manifest::Manifest::with_ids(
        "cost",
        crate::pack::PackType::Resource,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2),
    );
    let mut writer = PackWriter::new(manifest);
    for index in 0..n {
        let path = PackPath::new(&format!("textures/{}/{index}.json", index % 251)).unwrap();
        writer
            .put_bytes(path, format!("{{\"index\":{index}}}").into_bytes())
            .unwrap();
    }
    writer
}

#[test]
fn listing_the_files_costs_time_linear_in_their_number() {
    crate::test_cost::assert_linear_at("PackWriter::files", 100_000, writer_of, |writer| {
        let (count, bytes) = writer.files().fold((0, 0), |(count, bytes), (_, file)| {
            (count + 1, bytes + file.len())
        });
        assert!(bytes > count);
        count
    });
}

#[test]
fn copying_filling_and_opening_cost_time_linear_in_the_files() {
    crate::test_cost::assert_linear_at(
        "PackWriter::from_pack",
        20_000,
        |n| writer_of(n).into_pack(PackOrigin::World).unwrap(),
        |pack| PackWriter::from_pack(pack).unwrap(),
    );
    crate::test_cost::assert_linear_at("PackWriter::put_bytes", 20_000, |n| n, |n| writer_of(*n));
    crate::test_cost::assert_linear_consuming_at(
        "PackWriter::into_pack",
        10_000,
        writer_of,
        |writer| writer.into_pack(PackOrigin::World).unwrap(),
    );
}

#[cfg(feature = "zip")]
#[test]
fn the_archive_costs_time_linear_in_the_files() {
    crate::test_cost::assert_linear_at("PackWriter::to_archive", 2_000, writer_of, |writer| {
        writer.to_archive().unwrap()
    });
}

#[test]
fn a_cloned_writer_is_independent_and_writes_the_same_files() {
    let source = pack_from(
        MANIFEST,
        &[("textures/a.png", Some(b"a"))],
        PackOrigin::World,
    );
    let mut original = PackWriter::from_pack(&source).unwrap();
    let before: Vec<_> = original
        .files()
        .map(|(path, bytes)| (path.clone(), bytes))
        .collect();
    let mut copy = original.clone();
    let copied: Vec<_> = copy
        .files()
        .map(|(path, bytes)| (path.clone(), bytes))
        .collect();
    assert_eq!(copied, before);
    copy.manifest_mut().header.name = "changed".into();
    copy.put_bytes(PackPath::from_static("textures/b.png"), b"b".to_vec())
        .unwrap();
    assert_eq!(paths(&original), ["manifest.json", "textures/a.png"]);
    assert_eq!(paths(&copy).len(), 3);
    original.manifest_mut();
    assert_eq!(paths(&original).len(), 2);
}

#[test]
fn the_manifest_is_readable_without_a_mutable_borrow() {
    let source = pack_from(MANIFEST, &[], PackOrigin::World);
    let mut writer = PackWriter::from_pack(&source).unwrap();
    assert_eq!(writer.manifest().header.name, "ruby");
    writer.manifest_mut().header.name = "onyx".into();
    assert_eq!(writer.manifest().header.name, "onyx");
}

#[test]
fn put_bytes_takes_shared_bytes_without_a_copy() {
    let source = pack_from(MANIFEST, &[], PackOrigin::World);
    let mut writer = PackWriter::from_pack(&source).unwrap();
    let bytes = bytes::Bytes::from_static(b"png");
    writer
        .put_bytes(PackPath::from_static("textures/a.png"), bytes.clone())
        .unwrap();
    writer
        .put_bytes(PackPath::from_static("textures/b.png"), vec![1_u8])
        .unwrap();
    let kept: Vec<(&str, &[u8])> = writer
        .files()
        .map(|(path, bytes)| (path.as_str(), bytes))
        .filter(|(path, _)| path.starts_with("textures"))
        .collect();
    assert_eq!(
        kept,
        [
            ("textures/a.png", &b"png"[..]),
            ("textures/b.png", &[1_u8][..])
        ]
    );
}

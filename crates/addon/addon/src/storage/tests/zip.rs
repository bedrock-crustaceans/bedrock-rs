//! Zip decoding and encoding over the copied packs, the synthetic archives and archives built
//! here.

use std::io::{Cursor, Read, Write};

use ::zip::write::SimpleFileOptions;
use ::zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};
use bedrock_addon_core::load::Limits;
use bytes::Bytes;

use super::{fixture, fixture_names};
use crate::storage::zip::{ZipIndex, encode};
use crate::storage::{PackPath, PathRejected, StorageError, ZipFault, ZipLimit};

/// A zip of `files`, stored or deflated, written with the `zip` crate directly (not the crate's own encoder).
fn zip_of(files: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .last_modified_time(DateTime::DEFAULT);
    for (name, bytes) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn decode(bytes: impl Into<Bytes>) -> Result<ZipIndex, StorageError> {
    ZipIndex::decode(bytes.into(), &Limits::default())
}

fn paths(index: &ZipIndex) -> Vec<String> {
    index
        .files()
        .iter()
        .map(|entry| entry.path.to_string())
        .collect()
}

/// Every file of the index with its bytes, in index order.
fn contents(index: &ZipIndex) -> Vec<(PackPath, Bytes)> {
    index
        .files()
        .iter()
        .map(|entry| (entry.path.clone(), index.read(entry).unwrap()))
        .collect()
}

/// Replaces every occurrence of `from` by `to` (same length) in `bytes`.
fn patch(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    assert_eq!(from.len(), to.len());
    let mut found = 0;
    let mut i = 0;
    while i + from.len() <= bytes.len() {
        if &bytes[i..i + from.len()] == from {
            bytes[i..i + from.len()].copy_from_slice(to);
            found += 1;
            i += from.len();
        } else {
            i += 1;
        }
    }
    assert!(found > 0, "pattern not found");
}

mod packs {
    use super::*;

    #[test]
    fn every_zip_pack_decodes_to_the_unzip_listing() {
        let names = fixture_names("zip_packs");
        assert_eq!(names.len(), 23);
        for name in names {
            let index = decode(fixture(&format!("zip_packs/{name}"))).unwrap();
            // `unzip -Z1` lists directories with a trailing `/`; the index holds files only
            let listing = String::from_utf8(fixture(&format!("zip_listings/{name}.txt"))).unwrap();
            let mut expected: Vec<String> = listing
                .lines()
                .filter(|line| !line.ends_with('/'))
                .map(str::to_owned)
                .collect();
            expected.sort();
            assert_eq!(paths(&index), expected, "{name}");
            assert!(index.duplicates().is_empty(), "{name}");
        }
    }

    #[test]
    fn every_entry_reads_its_declared_size_and_matches_the_zip_crate() {
        for name in fixture_names("zip_packs") {
            let bytes = fixture(&format!("zip_packs/{name}"));
            let index = decode(bytes.clone()).unwrap();
            let mut oracle = ZipArchive::new(Cursor::new(bytes)).unwrap();
            for entry in index.files() {
                let read = index.read(entry).unwrap();
                assert_eq!(read.len() as u64, entry.size, "{name}: {}", entry.path);
                let mut expected = Vec::new();
                oracle
                    .by_name(entry.path.as_str())
                    .unwrap()
                    .read_to_end(&mut expected)
                    .unwrap();
                assert_eq!(read, expected, "{name}: {}", entry.path);
            }
        }
    }

    #[test]
    fn every_zip_pack_reencodes_to_a_zip_that_decodes_identically() {
        for name in fixture_names("zip_packs") {
            let index = decode(fixture(&format!("zip_packs/{name}"))).unwrap();
            let files = contents(&index);
            let encoded = encode(files.iter().map(|(path, bytes)| (path, bytes.as_ref()))).unwrap();
            let again = decode(encoded).unwrap();
            assert_eq!(contents(&again), files, "{name}");
        }
    }

    #[test]
    fn get_and_read_path_find_a_file() {
        let index = decode(fixture("zip_packs/bakedPack.zip")).unwrap();
        let manifest = PackPath::from_static("manifest.json");
        assert!(index.get(&manifest).is_some());
        assert!(index.read_path(&manifest).unwrap().starts_with(b"{"));
        assert!(matches!(
            index.read_path(&PackPath::from_static("missing.json")),
            Err(StorageError::NotFound)
        ));
    }
}

mod synthetic {
    use super::*;

    #[test]
    fn valid_reference_holds_one_manifest() {
        let index = decode(fixture("synthetic/valid_reference.zip")).unwrap();
        assert_eq!(paths(&index), ["manifest.json"]);
        assert_eq!(index.files()[0].size, 248);
    }

    #[test]
    fn empty_archive_has_no_files() {
        let index = decode(fixture("synthetic/empty_zip_zero_entries.zip")).unwrap();
        assert!(index.files().is_empty());
    }

    #[test]
    fn truncated_and_foreign_bytes_are_malformed() {
        for name in [
            "synthetic/truncated_2bytes.bin",
            "synthetic/truncated_4bytes.bin",
            "synthetic/truncated_near_full.bin",
            "synthetic/not_a_zip.zip",
        ] {
            assert!(
                matches!(
                    decode(fixture(name)),
                    Err(StorageError::Zip {
                        fault: ZipFault::Malformed
                    })
                ),
                "{name}"
            );
        }
        assert!(matches!(
            decode(Vec::new()),
            Err(StorageError::Zip {
                fault: ZipFault::Malformed
            })
        ));
    }
}

mod limits {
    use super::*;

    fn decode_with(bytes: Vec<u8>, limits: Limits) -> Result<ZipIndex, StorageError> {
        ZipIndex::decode(Bytes::from(bytes), &limits)
    }

    #[test]
    fn too_many_entries() {
        let bytes = zip_of(
            &[("a", b"1"), ("b", b"2"), ("c", b"3")],
            CompressionMethod::Stored,
        );
        let limits = Limits {
            max_entries: 2,
            ..Limits::default()
        };
        assert!(matches!(
            decode_with(bytes.clone(), limits),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::Entries
            })
        ));
        let limits = Limits {
            max_entries: 3,
            ..Limits::default()
        };
        assert!(decode_with(bytes, limits).is_ok());
    }

    #[test]
    fn entry_too_large_by_its_declared_size() {
        let bytes = zip_of(&[("a", &[0; 11])], CompressionMethod::Deflated);
        let limits = Limits {
            max_entry_bytes: 10,
            ..Limits::default()
        };
        assert!(matches!(
            decode_with(bytes, limits),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::EntryBytes
            })
        ));
    }

    #[test]
    fn pack_too_large_decompressed() {
        let bytes = zip_of(
            &[("a", &[0; 10]), ("b", &[0; 10])],
            CompressionMethod::Deflated,
        );
        let limits = Limits {
            max_pack_bytes: 19,
            ..Limits::default()
        };
        assert!(matches!(
            decode_with(bytes.clone(), limits),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::PackBytes
            })
        ));
        let limits = Limits {
            max_pack_bytes: 20,
            ..Limits::default()
        };
        assert!(decode_with(bytes, limits).is_ok());
    }

    #[test]
    fn archive_file_too_large() {
        let bytes = zip_of(&[("a", b"1")], CompressionMethod::Stored);
        let limits = Limits {
            max_archive_bytes: bytes.len() as u64 - 1,
            ..Limits::default()
        };
        assert!(matches!(
            decode_with(bytes, limits),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::ArchiveBytes
            })
        ));
    }

    #[test]
    fn entry_name_longer_than_255_bytes() {
        let long = "a".repeat(256);
        let bytes = zip_of(&[(long.as_str(), b"1")], CompressionMethod::Stored);
        assert!(matches!(
            decode(bytes),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::EntryName
            })
        ));
        let fits = "a".repeat(255);
        assert!(decode(zip_of(&[(fits.as_str(), b"1")], CompressionMethod::Stored)).is_ok());
    }

    #[test]
    fn a_declared_zip_bomb_is_refused_before_inflating() {
        // 300 MiB declared for a 16-byte entry: the central directory alone trips the default
        // 256 MiB per-entry limit.
        let mut bytes = zip_of(&[("bomb.json", &[b'x'; 16])], CompressionMethod::Stored);
        let declared = (300u32 << 20).to_le_bytes();
        patch(
            &mut bytes,
            &[16, 0, 0, 0, 16, 0, 0, 0],
            &[
                16,
                0,
                0,
                0,
                declared[0],
                declared[1],
                declared[2],
                declared[3],
            ],
        );
        assert!(matches!(
            decode(bytes),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::EntryBytes
            })
        ));
    }

    #[test]
    fn an_entry_inflating_past_its_declared_size_is_malformed() {
        // deflated 4 KiB of zeros, declared as 8 bytes
        let mut bytes = zip_of(&[("a.json", &[0; 4096])], CompressionMethod::Deflated);
        patch(&mut bytes, &4096u32.to_le_bytes(), &8u32.to_le_bytes());
        let index = decode(bytes).unwrap();
        let entry = &index.files()[0];
        assert_eq!(entry.size, 8);
        assert!(matches!(
            index.read(entry),
            Err(StorageError::Zip {
                fault: ZipFault::Malformed
            })
        ));
    }

    /// A central directory of `count` fixed-size records: a header signature, then zeros (the
    /// count checks never read them).
    fn directory_of(count: u64) -> Vec<u8> {
        let mut bytes = vec![0; usize::try_from(count).unwrap() * 46];
        bytes[..4].copy_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        bytes
    }

    /// An end-of-central-directory record declaring `count` entries (`0xFFFF` defers to the
    /// zip64 record) in a central directory of `size` bytes at offset 0.
    fn end_record(count: u16, size: u32) -> Vec<u8> {
        let mut bytes = vec![0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0];
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&[0; 6]);
        bytes
    }

    #[test]
    fn the_declared_entry_count_is_checked_before_the_directory_is_read() {
        // three entries declared over a directory of three empty records: the count decides
        let limits = Limits {
            max_entries: 2,
            ..Limits::default()
        };
        let mut bytes = directory_of(3);
        bytes.extend(end_record(3, 3 * 46));
        assert!(matches!(
            decode_with(bytes, limits),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::Entries
            })
        ));
    }

    #[test]
    fn a_zip64_end_record_declaring_65_537_entries_is_refused() {
        let mut bytes = directory_of(65_537);
        let size = bytes.len() as u64;
        // zip64 end-of-central-directory record right after the directory
        bytes.extend_from_slice(&[0x50, 0x4B, 0x06, 0x06]);
        bytes.extend_from_slice(&44u64.to_le_bytes());
        bytes.extend_from_slice(&[45, 0, 45, 0]);
        bytes.extend_from_slice(&[0; 8]);
        bytes.extend_from_slice(&65_537u64.to_le_bytes());
        bytes.extend_from_slice(&65_537u64.to_le_bytes());
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        // its locator, pointing at it
        bytes.extend_from_slice(&[0x50, 0x4B, 0x06, 0x07, 0, 0, 0, 0]);
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend(end_record(0xFFFF, u32::MAX));
        assert!(matches!(
            decode(bytes),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::Entries
            })
        ));
    }

    /// `bytes` (a zip with no archive comment) with `comment` set as its archive comment.
    fn with_comment(mut bytes: Vec<u8>, comment: &[u8]) -> Vec<u8> {
        let length = bytes.len() - 2;
        bytes[length..].copy_from_slice(&u16::try_from(comment.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(comment);
        bytes
    }

    #[test]
    fn an_end_record_signature_in_the_archive_comment_is_not_the_end_record() {
        let limits = Limits {
            max_entries: 2,
            ..Limits::default()
        };
        // prose naming the signature: its "count" is 8 293 and its comment overruns the file
        let prose = b"PK\x05\x06 is the end-record signature".to_vec();
        // a record declaring three entries in a central directory past the end of the file
        let mut record = vec![0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0, 3, 0, 3, 0, 0, 0, 0, 0];
        record.extend_from_slice(&0x7FFF_FFFFu32.to_le_bytes());
        record.extend_from_slice(&[0, 0]);
        for comment in [prose, record] {
            let bytes = with_comment(
                zip_of(&[("manifest.json", b"{}")], CompressionMethod::Stored),
                &comment,
            );
            let index = decode_with(bytes, limits).unwrap_or_else(|error| {
                panic!(
                    "`{}` read as the end record: {error:?}",
                    comment.escape_ascii()
                )
            });
            assert_eq!(paths(&index), ["manifest.json"]);
        }
    }

    #[test]
    fn a_long_name_past_the_records_the_zip_crate_reads_is_refused() {
        // the end record is patched to declare one entry; the raw directory still holds two
        let long = "y".repeat(256);
        let mut bytes = zip_of(
            &[("a.json", b"1"), (long.as_str(), b"2")],
            CompressionMethod::Stored,
        );
        let end = bytes.len() - 22;
        bytes[end + 8..end + 12].copy_from_slice(&[1, 0, 1, 0]);
        assert!(matches!(
            decode(bytes),
            Err(StorageError::LimitExceeded {
                limit: ZipLimit::EntryName
            })
        ));
    }
}

mod paths {
    use super::*;

    #[test]
    fn dot_dot_entry_is_rejected() {
        let bytes = zip_of(&[("../evil.json", b"{}")], CompressionMethod::Stored);
        assert!(matches!(
            decode(bytes),
            Err(StorageError::PathRejected {
                reason: PathRejected::DotSegment { .. },
                ..
            })
        ));
    }

    #[test]
    fn rejected_path_is_reported_as_written() {
        let bytes = zip_of(&[("a/./b.json", b"{}")], CompressionMethod::Stored);
        let Err(StorageError::PathRejected { path, .. }) = decode(bytes) else {
            panic!("not rejected");
        };
        assert_eq!(&*path, "a/./b.json");
    }

    #[test]
    fn colon_is_rejected() {
        let bytes = zip_of(&[("C:evil.json", b"{}")], CompressionMethod::Stored);
        assert!(matches!(
            decode(bytes),
            Err(StorageError::PathRejected {
                reason: PathRejected::InvalidChar { ch: ':', .. },
                ..
            })
        ));
    }

    #[test]
    fn backslashes_and_repeated_separators_normalise() {
        let bytes = zip_of(
            &[("textures\\blocks\\a.png", b"png"), ("b//c.json", b"{}")],
            CompressionMethod::Stored,
        );
        let index = decode(bytes).unwrap();
        assert_eq!(paths(&index), ["b/c.json", "textures/blocks/a.png"]);
        assert_eq!(
            index
                .read_path(&PackPath::from_static("textures/blocks/a.png"))
                .unwrap(),
            &b"png"[..]
        );
    }

    #[test]
    fn directory_entries_are_not_files() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .add_directory("textures/", SimpleFileOptions::default())
            .unwrap();
        writer
            .start_file("textures/a.png", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"png").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert_eq!(paths(&decode(bytes).unwrap()), ["textures/a.png"]);
    }
}

mod duplicates {
    use super::*;

    #[test]
    fn duplicate_entry_names_last_wins() {
        // the zip crate refuses to write a duplicate name, so write two names and rename one
        let mut bytes = zip_of(
            &[
                ("a.json", b"first"),
                ("b.json", b"other"),
                ("x.json", b"last!"),
            ],
            CompressionMethod::Stored,
        );
        patch(&mut bytes, b"x.json", b"a.json");
        let index = decode(bytes).unwrap();
        assert_eq!(paths(&index), ["a.json", "b.json"]);
        assert_eq!(
            index.read_path(&PackPath::from_static("a.json")).unwrap(),
            &b"last!"[..]
        );
        assert_eq!(index.duplicates(), [PackPath::from_static("a.json")]);
    }

    #[test]
    fn names_equal_after_normalising_are_duplicates_last_wins() {
        let bytes = zip_of(
            &[("a/b.json", b"first"), ("a\\b.json", b"last!")],
            CompressionMethod::Stored,
        );
        let index = decode(bytes).unwrap();
        assert_eq!(paths(&index), ["a/b.json"]);
        assert_eq!(
            index.read_path(&PackPath::from_static("a/b.json")).unwrap(),
            &b"last!"[..]
        );
        assert_eq!(index.duplicates(), [PackPath::from_static("a/b.json")]);
    }
}

mod writer {
    use super::*;

    fn sample() -> Vec<(PackPath, &'static [u8])> {
        vec![
            (
                PackPath::from_static("blocks/ruby.json"),
                br#"{"format_version":"1.21.0"}"#,
            ),
            (
                PackPath::from_static("manifest.json"),
                br#"{"format_version":2}"#,
            ),
            (PackPath::from_static("textures/ruby.png"), b"\x89PNG"),
        ]
    }

    fn encode_sample() -> Bytes {
        encode(sample().iter().map(|(path, bytes)| (path, *bytes))).unwrap()
    }

    #[test]
    fn output_is_byte_stable() {
        assert_eq!(encode_sample(), encode_sample());
    }

    #[test]
    fn manifest_comes_first_and_the_rest_keep_their_order() {
        let archive = ZipArchive::new(Cursor::new(encode_sample())).unwrap();
        let names: Vec<&str> = archive.file_names().collect();
        assert_eq!(
            names,
            ["manifest.json", "blocks/ruby.json", "textures/ruby.png"]
        );
    }

    #[test]
    fn every_entry_is_deflated_at_the_fixed_time() {
        let mut archive = ZipArchive::new(Cursor::new(encode_sample())).unwrap();
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            assert_eq!(
                file.last_modified(),
                Some(DateTime::DEFAULT),
                "{}",
                file.name()
            );
            assert_eq!(
                file.compression(),
                CompressionMethod::Deflated,
                "{}",
                file.name()
            );
        }
    }

    #[test]
    fn decodes_to_what_was_written() {
        let index = decode(encode_sample()).unwrap();
        let mut expected = sample();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        let read: Vec<(PackPath, Bytes)> = contents(&index);
        assert_eq!(
            read,
            expected
                .into_iter()
                .map(|(path, bytes)| (path, Bytes::from_static(bytes)))
                .collect::<Vec<_>>()
        );
    }
}

mod nested {
    use super::*;

    fn pack(name: &str) -> Vec<u8> {
        let manifest = format!(r#"{{"format_version":2,"header":{{"name":"{name}"}}}}"#);
        zip_of(
            &[("manifest.json", manifest.as_bytes())],
            CompressionMethod::Deflated,
        )
    }

    #[test]
    fn an_mcaddon_lists_its_nested_packs_by_magic() {
        let behavior = pack("bp");
        let resource = pack("rp");
        let addon = zip_of(
            &[
                ("bp.mcpack", &behavior),
                ("rp.zip.bin", &resource),
                ("fake.mcpack", b"not a zip at all"),
                ("folder_pack/manifest.json", br#"{"format_version":2}"#),
                ("folder_pack/inner.mcpack", &behavior),
            ],
            CompressionMethod::Deflated,
        );
        let index = decode(addon).unwrap();
        // top level only, detected by magic bytes, not by extension
        let nested = index.nested_archives();
        assert_eq!(
            nested.archives,
            [
                PackPath::from_static("bp.mcpack"),
                PackPath::from_static("rp.zip.bin")
            ]
        );
        assert!(nested.unreadable.is_empty());
        let inner = index
            .read_path(&PackPath::from_static("bp.mcpack"))
            .unwrap();
        let inner = ZipIndex::decode(inner, &Limits::default()).unwrap();
        assert!(
            inner
                .read_path(&PackPath::from_static("manifest.json"))
                .unwrap()
                .ends_with(br#""bp"}}"#)
        );
    }

    #[test]
    fn a_nested_composite_opens_from_the_outer_bytes() {
        let inner_addon = zip_of(
            &[("deep.mcpack", &pack("deep"))],
            CompressionMethod::Deflated,
        );
        let outer = zip_of(
            &[("inner.mcaddon", &inner_addon)],
            CompressionMethod::Stored,
        );
        let outer = decode(outer).unwrap();
        let nested = outer.nested_archives().archives;
        assert_eq!(nested, [PackPath::from_static("inner.mcaddon")]);
        let inner = decode(outer.read_path(&nested[0]).unwrap()).unwrap();
        assert_eq!(
            inner.nested_archives().archives,
            [PackPath::from_static("deep.mcpack")]
        );
    }

    #[test]
    fn an_unreadable_member_is_reported_and_the_rest_still_listed() {
        let behavior = pack("bp");
        let mut addon = zip_of(
            &[("bp.mcpack", &behavior), ("locked.mcpack", &behavior)],
            CompressionMethod::Stored,
        );
        set_encrypted_flag(&mut addon, b"locked.mcpack");
        let index = decode(addon).unwrap();
        let nested = index.nested_archives();
        assert_eq!(nested.archives, [PackPath::from_static("bp.mcpack")]);
        assert_eq!(nested.unreadable.len(), 1);
        assert_eq!(
            nested.unreadable[0].0,
            PackPath::from_static("locked.mcpack")
        );
        assert!(matches!(
            nested.unreadable[0].1,
            StorageError::Zip {
                fault: ZipFault::Unsupported
            }
        ));
    }

    /// Sets the zip-encryption bit of the entry named `name` in its local and central headers.
    fn set_encrypted_flag(bytes: &mut [u8], name: &[u8]) {
        let mut patched = 0;
        for at in 0..bytes.len().saturating_sub(46) {
            let (flags, name_len, name_at) = match &bytes[at..at + 4] {
                [0x50, 0x4B, 0x03, 0x04] => (at + 6, at + 26, at + 30),
                [0x50, 0x4B, 0x01, 0x02] => (at + 8, at + 28, at + 46),
                _ => continue,
            };
            let len = usize::from(u16::from_le_bytes([bytes[name_len], bytes[name_len + 1]]));
            if &bytes[name_at..name_at + len] == name {
                bytes[flags] |= 1;
                patched += 1;
            }
        }
        assert_eq!(patched, 2, "local and central header");
    }
}

/// `ZipArchive::extract` (and its unwrapped-root variant) writes to the host file system; the
/// storage code never calls it. The clippy `disallowed-methods` list enforces the same.
#[test]
fn extract_is_never_called() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/storage");
    let needle = concat!(".", "extract", "(");
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(!text.contains(needle), "{}", path.display());
                assert!(
                    !text.contains(concat!("extract", "_unwrapped_root_dir")),
                    "{}",
                    path.display()
                );
            }
        }
    }
}

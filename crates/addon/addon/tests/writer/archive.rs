//! `PackWriter::to_archive`: deflate, `manifest.json` first, the same bytes for the same
//! content, and a pack that opens from them as the pack that was written.

use std::io::{Cursor, Read};

use bedrock_addon::json::{EncodeError, Unrepresentable};
use bedrock_addon::manifest::{ManifestFormat, Setting};
use bedrock_addon::pack::{OpenOptions, Pack, PackWriter, WriteError};

use super::into_pack::assert_same_pack;
use super::packs::plain_packs;
use super::{listed, path, paths, ruby_manifest};

/// The files of the examples below, in an order that is not the order of `files()`.
const FILES: [(&str, &[u8]); 5] = [
    ("textures/ruby.png", b"\x89PNG ruby"),
    ("Blocks/upper.json", b"{}"),
    ("blocks/demo/ruby.json", b"{\"format_version\":\"1.21.0\"}"),
    ("a.txt", b""),
    ("texts/en_US.lang", b"tile.demo:ruby.name=Ruby\n"),
];

fn writer_of(files: impl IntoIterator<Item = (&'static str, &'static [u8])>) -> PackWriter {
    let mut writer = PackWriter::new(ruby_manifest());
    for (at, bytes) in files {
        writer.put_bytes(path(at), bytes).unwrap();
    }
    writer
}

/// The entries of `zip`, in archive order: name, bytes, method, time and mode.
fn entries(zip: &[u8]) -> Vec<(String, Vec<u8>, zip::CompressionMethod)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip)).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).unwrap();
            // Nothing of the host or of the clock: 1980-01-01 and plain file permissions.
            assert_eq!(
                entry.last_modified(),
                Some(zip::DateTime::default()),
                "{}",
                entry.name()
            );
            assert_eq!(entry.unix_mode(), Some(0o100_644), "{}", entry.name());
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (entry.name().to_owned(), bytes, entry.compression())
        })
        .collect()
}

#[test]
fn the_archive_holds_the_files_deflated_in_the_order_of_files() {
    let writer = writer_of(FILES);
    let zip = writer.to_archive().unwrap();
    let entries = entries(&zip);
    let names: Vec<&str> = entries.iter().map(|(name, ..)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "manifest.json",
            "Blocks/upper.json",
            "a.txt",
            "blocks/demo/ruby.json",
            "texts/en_US.lang",
            "textures/ruby.png",
        ]
    );
    assert_eq!(names, paths(&writer));
    for ((name, bytes, method), (_, expected)) in entries.iter().zip(listed(&writer)) {
        assert_eq!(*bytes, expected, "{name}");
        assert_eq!(*method, zip::CompressionMethod::Deflated, "{name}");
    }
}

#[test]
fn the_same_content_gives_the_same_archive_bytes() {
    let writer = writer_of(FILES);
    let first = writer.to_archive().unwrap();
    assert!(!first.is_empty());
    // The same writer twice.
    assert_eq!(writer.to_archive().unwrap(), first);
    // Another writer, filled in another order, with a file written twice and one removed.
    let mut other = writer_of(FILES.into_iter().rev());
    other.put_bytes(path("a.txt"), b"first".to_vec()).unwrap();
    other.put_bytes(path("a.txt"), Vec::new()).unwrap();
    other.put_bytes(path("gone.txt"), b"gone".to_vec()).unwrap();
    assert!(other.remove(&path("gone.txt")));
    assert_eq!(other.to_archive().unwrap(), first);
    // Other content, other bytes: a file's bytes, a file's name, the manifest.
    let mut changed = writer_of(FILES);
    changed.put_bytes(path("a.txt"), b"a".to_vec()).unwrap();
    assert_ne!(changed.to_archive().unwrap(), first);
    let mut renamed = writer_of(FILES);
    renamed.remove(&path("a.txt"));
    renamed.put_bytes(path("b.txt"), Vec::new()).unwrap();
    assert_ne!(renamed.to_archive().unwrap(), first);
    let mut retitled = writer_of(FILES);
    retitled.manifest_mut().header.name = "onyx".to_owned();
    assert_ne!(retitled.to_archive().unwrap(), first);
}

#[test]
fn the_archive_of_every_copied_fixture_pack_opens_as_the_same_pack() {
    for case in plain_packs() {
        let writer = PackWriter::from_pack(&case.pack).unwrap();
        let zip = writer
            .to_archive()
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        let pack = Pack::from_archive(zip, &OpenOptions::default())
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        assert_same_pack(&case, &pack);
    }
}

/// "A v1/v2 manifest carrying data only v3 can hold (settings beyond labels) is
/// `NotRepresentable`". `files()` cannot fail, so it lists no `manifest.json` for such a
/// writer; the archive is where the error is returned.
#[test]
fn a_format_2_manifest_given_a_format_3_setting_is_not_representable() {
    let mut writer = writer_of(FILES);
    writer.manifest_mut().settings.push(Setting::Label {
        text: "A label, which format 2 holds".to_owned(),
    });
    assert!(writer.to_archive().is_ok());
    assert_eq!(paths(&writer)[0], "manifest.json");
    writer.manifest_mut().settings.push(Setting::Toggle {
        name: "demo:lit".to_owned(),
        text: "Lit".to_owned(),
        default: true,
    });
    let refused = writer.to_archive();
    let Err(WriteError::Encode(EncodeError::NotRepresentable { at, what })) = refused else {
        panic!("{:?}", refused.map(|zip| zip.len()));
    };
    assert_eq!(at.to_string(), "settings/1");
    assert_eq!(what, Unrepresentable::Unsupported);
    assert_eq!(
        paths(&writer),
        [
            "Blocks/upper.json",
            "a.txt",
            "blocks/demo/ruby.json",
            "texts/en_US.lang",
            "textures/ruby.png",
        ],
        "every file but the manifest that cannot be written"
    );
    // In format 3 the same manifest is written, and the archive opens with the setting.
    writer.manifest_mut().format = ManifestFormat::V3;
    let zip = writer.to_archive().unwrap();
    assert_eq!(paths(&writer)[0], "manifest.json");
    let pack = Pack::from_archive(zip, &OpenOptions::default()).unwrap();
    assert_eq!(pack.manifest().settings.len(), 2);
    assert_eq!(pack.manifest().format, ManifestFormat::V3);
}

/// What `Pack::from_archive` refuses under the default limits, `to_archive` does not write:
/// a file whose path is longer than 255 bytes, and more entries than an archive may hold.
/// The error is the reader's own. The bounds themselves are written and read.
#[test]
fn to_archive_refuses_what_the_readers_default_limits_refuse() {
    use bedrock_addon::load::Limits;
    use bedrock_addon::storage::{StorageError, ZipLimit};
    let limit_of = |writer: &PackWriter| match writer.to_archive() {
        Err(WriteError::Zip {
            source: StorageError::LimitExceeded { limit },
        }) => Some(limit),
        Err(other) => panic!("{other:?}"),
        Ok(bytes) => {
            // What is written, the reader reads.
            let pack = Pack::from_archive(bytes, &OpenOptions::default()).unwrap();
            assert_eq!(pack.paths().count(), writer.files().count());
            None
        }
    };
    // A path of 255 bytes is an entry name the reader takes; one more byte is not.
    let name = |bytes: usize| format!("{}.json", "a/".repeat(20) + &"b".repeat(bytes - 45));
    assert_eq!(name(255).len(), 255);
    let mut longest = PackWriter::new(ruby_manifest());
    longest.put_bytes(path(&name(255)), &b"{}"[..]).unwrap();
    assert_eq!(limit_of(&longest), None);
    let mut too_long = PackWriter::new(ruby_manifest());
    too_long.put_bytes(path(&name(256)), &b"{}"[..]).unwrap();
    assert_eq!(limit_of(&too_long), Some(ZipLimit::EntryName));
    // The writer itself holds the file, and the pack it opens as has it: the limit is the
    // archive's, not the pack's.
    let opened = too_long
        .into_pack(bedrock_addon::pack::PackOrigin::World)
        .unwrap();
    assert!(opened.read(&path(&name(256))).is_ok());
    // As many entries as the reader takes, the manifest among them; one more is refused.
    let most = usize::try_from(Limits::default().max_entries).unwrap();
    let mut full = PackWriter::new(ruby_manifest());
    for n in 1..most {
        full.put_bytes(path(&format!("f/{n}")), Vec::new()).unwrap();
    }
    assert_eq!(full.files().count(), most);
    assert_eq!(limit_of(&full), None);
    full.put_bytes(path("f/one_more"), Vec::new()).unwrap();
    assert_eq!(limit_of(&full), Some(ZipLimit::Entries));
}

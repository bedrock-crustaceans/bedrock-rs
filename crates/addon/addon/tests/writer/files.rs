//! The writer's own file set: the manifest it writes and the order of `files()`.

use bedrock_addon::json::WriteStyle;
use bedrock_addon::json::{EncodeError, Unrepresentable};
use bedrock_addon::manifest::{Manifest, ManifestFormat, Setting};
use bedrock_addon::pack::{PackWriter, WriteError};
use bedrock_addon::storage::PackPath;

use super::{bytes_at, listed, parsed, path, paths, ruby_manifest};

/// `manifest` as `Manifest::write_json` writes it.
fn written(manifest: &bedrock_addon::manifest::Manifest) -> Vec<u8> {
    let mut out = Vec::new();
    manifest
        .write_json(&mut out, WriteStyle::default())
        .unwrap();
    out
}

#[test]
fn a_new_writer_holds_its_manifest_and_no_other_file() {
    let manifest = ruby_manifest();
    let writer = PackWriter::new(manifest.clone());
    assert_eq!(
        listed(&writer),
        [("manifest.json".to_owned(), written(&manifest))]
    );
}

#[test]
fn a_new_manifest_is_written_in_format_2() {
    let writer = PackWriter::new(ruby_manifest());
    let bytes = bytes_at(&writer, "manifest.json").expect("the manifest is a file");
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(text.contains("\"format_version\": 2"), "{text}");
    let read = parsed(&bytes);
    assert_eq!(read.format, ManifestFormat::V2);
    assert_eq!(read.original_format, ManifestFormat::V2);
    assert_eq!(read, ruby_manifest());
}

#[test]
fn the_manifest_file_follows_every_change_made_through_manifest_mut() {
    let mut writer = PackWriter::new(ruby_manifest());
    let before = bytes_at(&writer, "manifest.json").unwrap();
    writer.manifest_mut().header.name = "sapphire".to_owned();
    let after = bytes_at(&writer, "manifest.json").unwrap();
    assert_ne!(before, after);
    assert_eq!(parsed(&after).header.name, "sapphire");
    // And again: the bytes are those of the manifest as it is now, not of the first reading.
    writer.manifest_mut().header.name = "ruby".to_owned();
    assert_eq!(bytes_at(&writer, "manifest.json").unwrap(), before);
    assert_eq!(paths(&writer), ["manifest.json"]);
}

#[test]
fn put_bytes_adds_a_file_and_replaces_the_one_at_its_path() {
    let mut writer = PackWriter::new(ruby_manifest());
    writer
        .put_bytes(path("textures/ruby.png"), b"first".to_vec())
        .unwrap();
    assert_eq!(paths(&writer), ["manifest.json", "textures/ruby.png"]);
    assert_eq!(bytes_at(&writer, "textures/ruby.png").unwrap(), b"first");
    // A second file leaves the first alone.
    writer
        .put_bytes(path("textures/onyx.png"), &b"other"[..])
        .unwrap();
    assert_eq!(bytes_at(&writer, "textures/ruby.png").unwrap(), b"first");
    // The same path again: the new bytes, and still one file there.
    writer
        .put_bytes(path("textures/ruby.png"), b"second".to_vec())
        .unwrap();
    assert_eq!(bytes_at(&writer, "textures/ruby.png").unwrap(), b"second");
    assert_eq!(
        paths(&writer),
        ["manifest.json", "textures/onyx.png", "textures/ruby.png"]
    );
}

#[test]
fn files_follow_the_manifest_in_bytewise_path_order() {
    let mut writer = PackWriter::new(ruby_manifest());
    // Given in an order that is neither: bytewise, an upper-case letter sorts before every
    // lower-case one and `.` before `/`; without regard to case `a.json` and `a/z.json` lead
    // and `B.json` follows `b.json`.
    for name in [
        "b.json",
        "a/z.json",
        "Z.txt",
        "a.json",
        "B.json",
        "blocks/a.json",
    ] {
        writer.put_bytes(path(name), name.as_bytes()).unwrap();
    }
    assert_eq!(
        paths(&writer),
        [
            // First although `B.json`, `Z.txt`, `a…` and `b…` all sort before it.
            "manifest.json",
            "B.json",
            "Z.txt",
            "a.json",
            "a/z.json",
            "b.json",
            "blocks/a.json",
        ]
    );
    for (path, bytes) in listed(&writer).into_iter().skip(1) {
        assert_eq!(bytes, path.as_bytes());
    }
}

#[test]
fn remove_takes_the_file_away_and_says_whether_there_was_one() {
    let mut writer = PackWriter::new(ruby_manifest());
    writer.put_bytes(path("a.txt"), b"a".to_vec()).unwrap();
    writer.put_bytes(path("b.txt"), b"b".to_vec()).unwrap();
    assert!(writer.remove(&path("a.txt")));
    assert_eq!(paths(&writer), ["manifest.json", "b.txt"]);
    // Gone: a second removal finds nothing, and neither does a path never written.
    assert!(!writer.remove(&path("a.txt")));
    assert!(!writer.remove(&path("c.txt")));
    assert_eq!(paths(&writer), ["manifest.json", "b.txt"]);
}

/// A format-2 manifest holding a setting only format 3 can: `Manifest::write_json` refuses it.
fn unwritable_manifest() -> Manifest {
    let mut manifest = ruby_manifest();
    manifest.settings.push(Setting::Toggle {
        name: "demo:lit".to_owned(),
        text: "Lit".to_owned(),
        default: true,
    });
    manifest
}

#[test]
fn put_writes_a_document_as_its_json_and_replaces_the_one_at_its_path() {
    let mut writer = PackWriter::new(ruby_manifest());
    let mut document = ruby_manifest();
    let at = path("copies/of_the_manifest.json");
    writer.put(at.clone(), &document).unwrap();
    assert_eq!(
        bytes_at(&writer, at.as_str()).unwrap(),
        written(&document),
        "the bytes of `Content::write_json` in the default style"
    );
    document.header.name = "onyx".to_owned();
    writer.put(at.clone(), &document).unwrap();
    assert_eq!(bytes_at(&writer, at.as_str()).unwrap(), written(&document));
    assert_eq!(paths(&writer), ["manifest.json", at.as_str()]);
}

#[test]
fn a_document_that_cannot_be_written_is_an_encode_error_and_changes_nothing() {
    let mut writer = PackWriter::new(ruby_manifest());
    let at = path("copies/of_the_manifest.json");
    writer.put(at.clone(), &ruby_manifest()).unwrap();
    let before = listed(&writer);
    // Replacing a file and writing a new one: both refused, both without a trace.
    for target in [at, path("copies/another.json")] {
        let refused = writer.put(target, &unwritable_manifest());
        assert!(
            matches!(
                refused,
                Err(WriteError::Encode(EncodeError::NotRepresentable {
                    what: Unrepresentable::Unsupported,
                    ..
                }))
            ),
            "{refused:?}"
        );
        assert_eq!(listed(&writer), before);
    }
}

#[test]
fn manifest_json_and_the_pack_root_are_not_paths_of_a_file() {
    let mut writer = PackWriter::new(ruby_manifest());
    let before = listed(&writer);
    for reserved in [path("manifest.json"), PackPath::ROOT] {
        let refused = writer.put_bytes(reserved.clone(), b"{}".to_vec());
        assert!(
            matches!(&refused, Err(WriteError::ReservedPath { path }) if *path == reserved),
            "put_bytes {reserved:?}: {refused:?}"
        );
        let refused = writer.put(reserved.clone(), &ruby_manifest());
        assert!(
            matches!(&refused, Err(WriteError::ReservedPath { path }) if *path == reserved),
            "put {reserved:?}: {refused:?}"
        );
        // Nothing to take away either: the manifest stays, written from the manifest.
        assert!(!writer.remove(&reserved));
        assert_eq!(listed(&writer), before);
    }
    // Only the manifest at the root is reserved: the same name in a folder, and a name that
    // differs by case, are files like any other (vanilla reads paths case-sensitively).
    for free in ["subpacks/a/manifest.json", "Manifest.json"] {
        writer.put_bytes(path(free), b"{}".to_vec()).unwrap();
    }
    assert_eq!(
        paths(&writer),
        ["manifest.json", "Manifest.json", "subpacks/a/manifest.json"]
    );
}

#[test]
fn a_writer_is_send_and_sync_and_prints_its_identity_without_its_bytes() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<PackWriter>();
    let mut writer = PackWriter::new(ruby_manifest());
    writer
        .put_bytes(path("secret.txt"), b"not for a log".to_vec())
        .unwrap();
    let printed = format!("{writer:?}");
    assert!(
        printed.contains("00000000-0000-0000-0000-000000000001"),
        "{printed}"
    );
    assert!(printed.contains("files: 1"), "{printed}");
    assert!(
        !printed.contains("not for a log") && !printed.contains("110, 111"),
        "{printed}"
    );
}

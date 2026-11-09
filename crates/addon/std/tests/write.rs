//! `write_dir` and `write_archive` through the public API. The disk side is tested in
//! `src/write.rs` without a `PackWriter`; the round trips here go through one.

use bedrock_addon::pack::WriteError;
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::SaveError;

#[test]
fn a_writer_error_converts_to_a_pack_error() {
    let duplicate = || WriteError::DuplicatePath {
        path: PackPath::from_static("blocks/a_b.json"),
    };
    assert!(matches!(
        SaveError::from(duplicate()),
        SaveError::Pack(WriteError::DuplicatePath { .. })
    ));
}

#[test]
fn write_dir_writes_the_writers_files_byte_for_byte() {
    use bedrock_addon::manifest::Manifest;
    use bedrock_addon::pack::{PackType, PackWriter};

    let writer = PackWriter::new(Manifest::with_ids(
        "ruby",
        PackType::Behavior,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2),
    ));
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ruby");
    bedrock_addon_std::write_dir(&writer, &out).unwrap();
    for (path, bytes) in writer.files() {
        let host = path
            .segments()
            .fold(out.clone(), |host, segment| host.join(segment));
        assert_eq!(std::fs::read(host).unwrap(), bytes, "{}", path.as_str());
    }
    assert!(matches!(
        bedrock_addon_std::write_dir(&writer, &out),
        Err(SaveError::NotEmpty { .. })
    ));
}

#[cfg(feature = "zip")]
#[test]
fn write_archive_is_byte_stable_across_runs() {
    use bedrock_addon::manifest::Manifest;
    use bedrock_addon::pack::{PackType, PackWriter};

    let writer = PackWriter::new(Manifest::with_ids(
        "ruby",
        PackType::Behavior,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2),
    ));
    let dir = tempfile::tempdir().unwrap();
    bedrock_addon_std::write_archive(&writer, dir.path().join("a.mcpack")).unwrap();
    bedrock_addon_std::write_archive(&writer, dir.path().join("b.mcpack")).unwrap();
    let first = std::fs::read(dir.path().join("a.mcpack")).unwrap();
    assert_eq!(first, std::fs::read(dir.path().join("b.mcpack")).unwrap());
    assert_eq!(first, writer.to_archive().unwrap());
}

/// A writer whose format-2 manifest holds a setting only format 3 can: `PackWriter::files`
/// lists no `manifest.json` for it.
// `files()` lists every file but `manifest.json`, and `to_archive` and `into_pack` return the
// error, so no stale or empty manifest bytes are ever written.
fn writer_with_an_unwritable_manifest() -> bedrock_addon::pack::PackWriter {
    use bedrock_addon::manifest::{Manifest, Setting};
    use bedrock_addon::pack::{PackType, PackWriter};
    use bedrock_addon::storage::PackPath;

    let mut manifest = Manifest::with_ids(
        "ruby",
        PackType::Behavior,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2),
    );
    manifest.settings.push(Setting::Toggle {
        name: "demo:lit".to_owned(),
        text: "Lit".to_owned(),
        default: true,
    });
    let mut writer = PackWriter::new(manifest);
    writer
        .put_bytes(PackPath::from_static("textures/ruby.png"), b"ruby".to_vec())
        .unwrap();
    writer
}

/// Where an `Encode` error of the writer points, or the whole error when it is another.
fn encode_error_at(error: &SaveError) -> Result<String, String> {
    use bedrock_addon::json::EncodeError;
    match error {
        SaveError::Pack(WriteError::Encode(EncodeError::NotRepresentable { at, .. })) => {
            Ok(at.to_string())
        }
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn write_dir_of_a_manifest_that_cannot_be_written_is_the_writers_error_and_writes_nothing() {
    let writer = writer_with_an_unwritable_manifest();
    let dir = tempfile::tempdir().unwrap();
    // Absent before, and an empty folder that is already there: neither gets a file.
    let absent = dir.path().join("absent");
    let empty = dir.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    for out in [&absent, &empty] {
        let refused = bedrock_addon_std::write_dir(&writer, out)
            .expect_err("a pack folder without a manifest was written");
        let at = encode_error_at(&refused).unwrap();
        // With `zip` the error is the one `to_archive` gives, with the member's path.
        #[cfg(feature = "zip")]
        {
            let archive = bedrock_addon_std::write_archive(&writer, dir.path().join("a.mcpack"))
                .expect_err("an archive without a manifest was written");
            assert_eq!(at, "settings/0");
            assert_eq!(Ok(at), encode_error_at(&archive));
        }
        #[cfg(not(feature = "zip"))]
        assert_eq!(at, "");
    }
    assert!(!absent.exists(), "not even the folder");
    assert_eq!(std::fs::read_dir(&empty).unwrap().count(), 0);
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        1,
        "no archive and no temporary file either"
    );
}

//! Writing a `PackWriter`'s files to a directory or its archive to a file.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use bedrock_addon::pack::PackWriter;
use bedrock_addon::storage::PackPath;

use crate::io::host_path;
use crate::{HostIoError, HostOp, SaveError};

/// Writes `writer.files()` under `dir`, creating it; `SaveError::NotEmpty` if it exists and
/// is not empty (the writer never merges into an existing pack folder).
///
/// Each file is written with exactly the bytes `writer.files()` gives, at its pack path below
/// `dir` (parent directories created as needed). A failure part-way leaves the files written
/// so far; the directory was empty or absent before, so removing it undoes the attempt.
///
/// A writer whose manifest its format cannot hold lists no `manifest.json`: that is
/// `SaveError::Pack(WriteError::Encode(..))`, and nothing is written or created.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bedrock_addon::manifest::Manifest;
/// use bedrock_addon::pack::{PackType, PackWriter};
/// use bedrock_addon_std::write_dir;
///
/// let writer = PackWriter::new(Manifest::new("Ruby lamps", PackType::Behavior));
/// write_dir(&writer, "behavior_packs/ruby_lamps")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// `SaveError::NotEmpty` when `dir` is not empty, `SaveError::Io` when a directory or file
/// cannot be written, and `SaveError::Pack` when the writer's manifest cannot be encoded.
pub fn write_dir(writer: &PackWriter, dir: impl AsRef<Path>) -> Result<(), SaveError> {
    let dir = dir.as_ref();
    require_manifest(writer)?;
    ensure_empty_dir(dir)?;
    write_files(writer.files(), dir)
}

/// Refuses a writer whose files do not start with `manifest.json`: its manifest cannot be
/// written in its format, and a folder of the other files would be no pack.
fn require_manifest(writer: &PackWriter) -> Result<(), SaveError> {
    use bedrock_addon::diagnostic::JsonPath;
    use bedrock_addon::json::{EncodeError, Unrepresentable};
    use bedrock_addon::pack::WriteError;

    if writer
        .files()
        .next()
        .is_some_and(|(path, _)| path.as_str() == "manifest.json")
    {
        return Ok(());
    }
    // The writer's own error, with the member's path: `to_archive` returns it before it zips
    // anything.
    #[cfg(feature = "zip")]
    writer.to_archive()?;
    // Without `zip` the writer has no call that returns it: the same variant, at the root.
    Err(SaveError::Pack(WriteError::Encode(
        EncodeError::NotRepresentable {
            at: JsonPath::new(),
            what: Unrepresentable::Unsupported,
        },
    )))
}

/// Writes `writer.to_archive()` to a uniquely named temporary file in `path`'s directory,
/// `sync_all`s it, and renames it over `path` (`std::fs::rename` replaces an existing file on
/// every platform); on any error the temporary file is removed. A crash leaves either the old
/// archive or the new one, never a torn file.
///
/// # Errors
///
/// `SaveError::Pack` when the archive cannot be built, `SaveError::NoFileName` when `path` ends
/// in no file name, and `SaveError::Io` when the temporary file cannot be created, written or
/// renamed.
#[cfg(feature = "zip")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
pub fn write_archive(writer: &PackWriter, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let bytes = writer.to_archive()?;
    replace_file(path.as_ref(), &bytes)
}

/// `dir` exists and is empty afterwards: created (with its parents) when absent, refused with
/// `NotEmpty` when it holds anything.
fn ensure_empty_dir(dir: &Path) -> Result<(), SaveError> {
    match fs::read_dir(dir) {
        Ok(mut entries) => match entries.next() {
            None => Ok(()),
            Some(Ok(_)) => Err(SaveError::NotEmpty {
                path: dir.to_path_buf(),
            }),
            Some(Err(source)) => Err(HostIoError::new(HostOp::List, dir, source).into()),
        },
        Err(source) if source.kind() == ErrorKind::NotFound => fs::create_dir_all(dir)
            .map_err(|source| HostIoError::new(HostOp::CreateDir, dir, source).into()),
        Err(source) => Err(HostIoError::new(HostOp::List, dir, source).into()),
    }
}

/// Writes each file at its pack path below `dir`, which exists.
fn write_files<'a>(
    files: impl IntoIterator<Item = (&'a PackPath, &'a [u8])>,
    dir: &Path,
) -> Result<(), SaveError> {
    for (path, bytes) in files {
        let host = host_path(dir, path);
        if let Some(parent) = host.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| HostIoError::new(HostOp::CreateDir, parent, source))?;
        }
        fs::write(&host, bytes).map_err(|source| HostIoError::new(HostOp::Write, &host, source))?;
    }
    Ok(())
}

/// Replaces the file at `path` with `bytes` through a synced temporary file in its directory;
/// the temporary file is removed on any error.
#[cfg(any(feature = "zip", test))]
fn replace_file(path: &Path, bytes: &[u8]) -> Result<(), SaveError> {
    use std::io::Write;

    let Some(name) = path.file_name() else {
        return Err(SaveError::NoFileName { path: path.into() });
    };
    let (temp, mut file) = create_temp_file(temp_dir_of(path), name)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let failed = match written {
        Err(source) => Some(HostIoError::new(HostOp::Write, &temp, source)),
        Ok(()) => fs::rename(&temp, path)
            .err()
            .map(|source| HostIoError::new(HostOp::Rename, path, source)),
    };
    if let Some(error) = failed {
        // Best effort: the error to report is the write's, not the cleanup's.
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }
    Ok(())
}

/// The directory the temporary file of `path` goes to: its parent, or `.` for a bare name.
#[cfg(any(feature = "zip", test))]
fn temp_dir_of(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Creates `.<name>.<process id>.<counter>.tmp` in `dir`, never opening an existing file.
#[cfg(any(feature = "zip", test))]
fn create_temp_file(
    dir: &Path,
    name: &std::ffi::OsStr,
) -> Result<(std::path::PathBuf, fs::File), SaveError> {
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Names already taken (by another process with a recycled id) are skipped this often.
    const ATTEMPTS: u32 = 16;
    static NEXT: AtomicU64 = AtomicU64::new(0);

    for _ in 0..ATTEMPTS {
        let mut temp_name = std::ffi::OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(
            ".{}.{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let temp = dir.join(temp_name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => return Ok((temp, file)),
            Err(source) if source.kind() == ErrorKind::AlreadyExists => {}
            Err(source) => return Err(HostIoError::new(HostOp::CreateTemp, &temp, source).into()),
        }
    }
    Err(HostIoError::new(HostOp::CreateTemp, dir, ErrorKind::AlreadyExists.into()).into())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods, clippy::disallowed_types)]

    use std::fs;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    use super::*;

    fn files() -> Vec<(PackPath, Vec<u8>)> {
        vec![
            (
                PackPath::from_static("manifest.json"),
                b"{\"format_version\":2}".to_vec(),
            ),
            (
                PackPath::from_static("blocks/ores/ruby.json"),
                b"{\"format_version\":\"1.26.20\"}".to_vec(),
            ),
            (
                PackPath::from_static("pack_icon.png"),
                vec![0x89, b'P', b'N', b'G', 0, 255],
            ),
            (PackPath::from_static("texts/en_US.lang"), Vec::new()),
        ]
    }

    /// `write_dir` without a `PackWriter`: the disk side alone, over files given by hand.
    fn write_dir_of(files: &[(PackPath, Vec<u8>)], dir: &Path) -> Result<(), SaveError> {
        ensure_empty_dir(dir)?;
        write_files(
            files.iter().map(|(path, bytes)| (path, bytes.as_slice())),
            dir,
        )
    }

    /// Every file below `dir`, relative path and bytes, sorted.
    fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut found = Vec::new();
        let mut pending = vec![dir.to_path_buf()];
        while let Some(current) = pending.pop() {
            for entry in fs::read_dir(&current).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    let relative = path.strip_prefix(dir).unwrap();
                    let name = relative
                        .components()
                        .map(|c| c.as_os_str().to_str().unwrap())
                        .collect::<Vec<_>>()
                        .join("/");
                    found.push((name, fs::read(&path).unwrap()));
                }
            }
        }
        found.sort();
        found
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    mod dir {
        use super::*;

        #[test]
        fn creates_the_directory_and_writes_every_file_byte_for_byte() {
            let temp = tempfile::tempdir().unwrap();
            let out = temp.path().join("packs/ruby");
            write_dir_of(&files(), &out).unwrap();
            let mut expected: Vec<(String, Vec<u8>)> = files()
                .into_iter()
                .map(|(path, bytes)| (path.as_str().to_owned(), bytes))
                .collect();
            expected.sort();
            assert_eq!(tree(&out), expected);
        }

        #[test]
        fn two_runs_write_identical_trees() {
            let temp = tempfile::tempdir().unwrap();
            write_dir_of(&files(), &temp.path().join("a")).unwrap();
            write_dir_of(&files(), &temp.path().join("b")).unwrap();
            assert_eq!(tree(&temp.path().join("a")), tree(&temp.path().join("b")));
        }

        #[test]
        fn an_existing_empty_directory_is_used() {
            let temp = tempfile::tempdir().unwrap();
            write_dir_of(&files(), temp.path()).unwrap();
            assert_eq!(tree(temp.path()).len(), files().len());
        }

        #[test]
        fn a_non_empty_directory_is_not_empty_and_untouched() {
            let temp = tempfile::tempdir().unwrap();
            fs::write(temp.path().join("keep.txt"), b"mine").unwrap();
            match write_dir_of(&files(), temp.path()) {
                Err(SaveError::NotEmpty { path }) => assert_eq!(path, temp.path()),
                other => panic!("expected NotEmpty, got {other:?}"),
            }
            assert_eq!(
                tree(temp.path()),
                vec![("keep.txt".to_owned(), b"mine".to_vec())]
            );
        }

        #[test]
        fn a_directory_holding_only_a_subdirectory_is_not_empty() {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("blocks")).unwrap();
            assert!(matches!(
                write_dir_of(&files(), temp.path()),
                Err(SaveError::NotEmpty { .. })
            ));
        }

        #[test]
        fn a_target_that_is_a_file_is_an_io_error() {
            let temp = tempfile::tempdir().unwrap();
            let file = temp.path().join("ruby.mcpack");
            fs::write(&file, b"PK").unwrap();
            match write_dir_of(&files(), &file) {
                Err(SaveError::Io(error)) => {
                    assert_eq!(error.op, HostOp::List);
                    assert_eq!(error.path, file);
                    assert_eq!(error.source.kind(), ErrorKind::NotADirectory);
                }
                other => panic!("expected an I/O error, got {other:?}"),
            }
        }

        #[test]
        fn a_file_that_cannot_be_written_is_an_io_error_naming_it() {
            let temp = tempfile::tempdir().unwrap();
            // `blocks` is a file, so `blocks/ores/ruby.json` cannot be created below it.
            let clash = vec![
                (PackPath::from_static("blocks"), b"file".to_vec()),
                (
                    PackPath::from_static("blocks/ores/ruby.json"),
                    b"{}".to_vec(),
                ),
            ];
            match write_dir_of(&clash, &temp.path().join("out")) {
                Err(SaveError::Io(error)) => {
                    let path = &error.path;
                    assert!(path.starts_with(temp.path().join("out/blocks")), "{path:?}");
                }
                other => panic!("expected an I/O error, got {other:?}"),
            }
        }
    }

    mod archive {
        use super::*;

        const ZIP: &[u8] = b"PK\x03\x04 pretend archive bytes";

        fn archive_dir() -> (tempfile::TempDir, PathBuf) {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("ruby.mcpack");
            (temp, path)
        }

        #[test]
        fn writes_the_bytes_and_leaves_no_temporary_file() {
            let (temp, path) = archive_dir();
            replace_file(&path, ZIP).unwrap();
            assert_eq!(fs::read(&path).unwrap(), ZIP);
            assert_eq!(names_in(temp.path()), vec!["ruby.mcpack".to_owned()]);
        }

        #[test]
        fn replaces_an_existing_archive() {
            let (temp, path) = archive_dir();
            fs::write(
                &path,
                b"old archive, longer than the new one by a good margin",
            )
            .unwrap();
            replace_file(&path, ZIP).unwrap();
            assert_eq!(fs::read(&path).unwrap(), ZIP);
            assert_eq!(names_in(temp.path()), vec!["ruby.mcpack".to_owned()]);
        }

        #[test]
        fn two_runs_write_identical_bytes() {
            let (temp, path) = archive_dir();
            let other = temp.path().join("copy.mcpack");
            replace_file(&path, ZIP).unwrap();
            replace_file(&other, ZIP).unwrap();
            assert_eq!(fs::read(&path).unwrap(), fs::read(&other).unwrap());
        }

        #[test]
        fn the_temporary_file_is_removed_when_the_rename_fails() {
            let (temp, path) = archive_dir();
            // A non-empty directory where the archive goes: the rename over it fails.
            fs::create_dir(&path).unwrap();
            fs::write(path.join("inside"), b"x").unwrap();
            match replace_file(&path, ZIP) {
                Err(SaveError::Io(error)) => assert_eq!(error.path, path),
                other => panic!("expected an I/O error, got {other:?}"),
            }
            assert_eq!(names_in(temp.path()), vec!["ruby.mcpack".to_owned()]);
            assert_eq!(names_in(&path), vec!["inside".to_owned()]);
        }

        #[test]
        fn a_failed_rename_names_the_operation() {
            let (_temp, path) = archive_dir();
            // A non-empty directory where the archive goes: only the rename over it fails.
            fs::create_dir(&path).unwrap();
            fs::write(path.join("inside"), b"x").unwrap();
            assert!(
                matches!(
                    replace_file(&path, ZIP),
                    Err(SaveError::Io(HostIoError {
                        op: HostOp::Rename,
                        ..
                    }))
                ),
                "the failed call was the rename"
            );
        }

        #[test]
        fn a_missing_directory_is_an_io_error_and_leaves_nothing() {
            let (temp, _) = archive_dir();
            let path = temp.path().join("missing/ruby.mcpack");
            assert!(matches!(
                replace_file(&path, ZIP),
                Err(SaveError::Io(HostIoError { op: HostOp::CreateTemp, source, .. }))
                    if source.kind() == ErrorKind::NotFound
            ));
            assert!(names_in(temp.path()).is_empty());
        }

        #[test]
        fn a_path_without_a_file_name_is_no_io_error() {
            assert!(matches!(
                replace_file(Path::new("/"), ZIP),
                Err(SaveError::NoFileName { .. })
            ));
        }

        #[test]
        fn a_relative_path_writes_next_to_the_working_directory_name() {
            // `Path::parent` of a bare file name is `""`; the temporary file goes to `.`.
            assert_eq!(
                super::super::temp_dir_of(Path::new("ruby.mcpack")),
                Path::new(".")
            );
            assert_eq!(
                super::super::temp_dir_of(Path::new("out/ruby.mcpack")),
                Path::new("out")
            );
        }
    }
}

//! The driver's errors: no pack (`LoadError`), an unreadable server-side file (`ReadError`), a
//! pack that could not be written (`SaveError`).

use std::fmt;
use std::path::{Path, PathBuf};

use bedrock_addon::document::Rejected;
use bedrock_addon::load::{DriveError, LoaderError};
use bedrock_addon::pack::OpenError;
use bedrock_addon::storage::PathRejected;

/// Why the driver produced no pack, repository or stacks.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    /// The pack itself could not be produced; `path` is the host path of `source`'s location.
    /// The message does not repeat `source`, which error reporters print from the chain.
    #[error("no pack could be opened at {path}")]
    Open {
        /// The host path.
        path: PathBuf,
        /// Why.
        #[source]
        source: OpenError,
    },
    /// A loader reported a protocol violation: a bug in this driver.
    #[error(transparent)]
    Loader(#[from] LoaderError),
    /// The loader waits, but nothing is outstanding: a bug in the loader.
    #[error("the loader stalled with no request or job outstanding")]
    Stalled,
    /// The rayon pool could not be built.
    #[cfg(feature = "rayon")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    #[error(transparent)]
    Pool(#[from] rayon::ThreadPoolBuildError),
}

/// `Stalled` and `Loader` keep their meaning; a later `DriveError` is a stall too.
impl From<DriveError> for LoadError {
    fn from(error: DriveError) -> Self {
        match error {
            DriveError::Loader(error) => LoadError::Loader(error),
            _ => LoadError::Stalled,
        }
    }
}

/// What the host was asked to do when an I/O call failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HostOp {
    /// Read a file's bytes.
    Read,
    /// List a directory.
    List,
    /// Look a path up (type, size).
    Metadata,
    /// Create a directory and its parents.
    CreateDir,
    /// Write a file's bytes.
    Write,
    /// Rename a file over its target.
    Rename,
    /// Create the temporary file of an atomic write.
    CreateTemp,
}

/// The verb, in lowercase, as the message of [`HostIoError`] uses it.
impl fmt::Display for HostOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HostOp::Read => "read",
            HostOp::List => "list",
            HostOp::Metadata => "inspect",
            HostOp::CreateDir => "create directory",
            HostOp::Write => "write",
            HostOp::Rename => "rename",
            HostOp::CreateTemp => "create temporary file",
        })
    }
}

/// A host I/O call that failed: what was attempted, on which path, and the error the host gave.
/// The message does not repeat `source`, which error reporters print from the chain.
#[derive(Debug, thiserror::Error)]
#[error("cannot {op} {path}")]
#[non_exhaustive]
pub struct HostIoError {
    /// What was attempted.
    pub op: HostOp,
    /// The host path it was attempted on.
    pub path: PathBuf,
    /// Why it failed.
    #[source]
    pub source: std::io::Error,
}

impl HostIoError {
    /// `op` on `path` failed with `source`.
    pub(crate) fn new(op: HostOp, path: &Path, source: std::io::Error) -> HostIoError {
        HostIoError {
            op,
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Why a server-side file could not be read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// A host call failed: the file or directory could not be looked up, listed or read.
    #[error(transparent)]
    Io(#[from] HostIoError),
    /// The app-data root exists and is not a directory.
    #[error("{path} is not a directory")]
    NotADirectory {
        /// The host path.
        path: PathBuf,
    },
    /// A file's name is not valid UTF-8, so it has no pack path.
    #[error("{path} has a name that is not valid UTF-8")]
    NotUtf8 {
        /// The host path.
        path: PathBuf,
    },
    /// A file's name, read as a pack path, breaks a path rule (a `:` or a control character).
    #[error("{path} is not a valid pack path")]
    NotAPackPath {
        /// The host path.
        path: PathBuf,
        /// The rule it breaks.
        #[source]
        source: PathRejected,
    },
    /// The file was read but rejected (`world_*_packs.json` with an error).
    #[error(transparent)]
    Rejected(#[from] Rejected),
}

/// Why a pack could not be written to disk.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SaveError {
    /// A host call failed: a directory could not be listed or created, or a file written.
    #[error(transparent)]
    Io(#[from] HostIoError),
    /// The target directory exists and is not empty.
    #[error("{path} exists and is not empty")]
    NotEmpty {
        /// The host path.
        path: PathBuf,
    },
    /// The archive path ends in no file name, so no temporary file can sit next to it.
    #[error("{path} has no file name")]
    NoFileName {
        /// The host path.
        path: PathBuf,
    },
    /// The pack could not be produced.
    #[error(transparent)]
    Pack(#[from] bedrock_addon::pack::WriteError),
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn host_io_errors_keep_the_operation_the_path_and_the_source() {
        let path = Path::new("packs/ruby");
        let error = HostIoError::new(HostOp::List, path, ErrorKind::NotFound.into());
        assert_eq!(error.to_string(), "cannot list packs/ruby");
        assert!(matches!(
            ReadError::from(error),
            ReadError::Io(HostIoError { op: HostOp::List, path: found, source })
                if found == path && source.kind() == ErrorKind::NotFound
        ));
        assert!(matches!(
            SaveError::from(HostIoError::new(
                HostOp::Write,
                path,
                ErrorKind::PermissionDenied.into()
            )),
            SaveError::Io(HostIoError { op: HostOp::Write, source, .. })
                if source.kind() == ErrorKind::PermissionDenied
        ));
    }
}

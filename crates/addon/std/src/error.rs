//! The driver's errors: no pack (`LoadError`), an unreadable server-side file (`ReadError`), a
//! pack that could not be written (`SaveError`).

use std::path::{Path, PathBuf};

use bedrock_addon::document::Rejected;
use bedrock_addon::load::{DriveError, LoaderError};
use bedrock_addon::pack::OpenError;

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

/// Why a server-side file could not be read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// The file could not be read.
    #[error("cannot read {path}")]
    Io {
        /// The host path.
        path: PathBuf,
        /// Why.
        #[source]
        source: std::io::Error,
    },
    /// The file was read but rejected (`world_*_packs.json` with an error).
    #[error(transparent)]
    Rejected(#[from] Rejected),
}

impl ReadError {
    /// `Io` for `source` at `path`.
    pub(crate) fn io(path: &Path, source: std::io::Error) -> ReadError {
        ReadError::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Why a pack could not be written to disk.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SaveError {
    /// A file or directory could not be written.
    #[error("cannot write {path}")]
    Io {
        /// The host path.
        path: PathBuf,
        /// Why.
        #[source]
        source: std::io::Error,
    },
    /// The target directory exists and is not empty.
    #[error("{path} exists and is not empty")]
    NotEmpty {
        /// The host path.
        path: PathBuf,
    },
    /// The pack could not be produced.
    #[error(transparent)]
    Pack(#[from] bedrock_addon::pack::WriteError),
}

impl SaveError {
    /// `Io` for `source` at `path`.
    pub(crate) fn io(path: &Path, source: std::io::Error) -> SaveError {
        SaveError::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn io_constructors_keep_the_path_and_the_source() {
        let path = Path::new("packs/ruby");
        assert!(matches!(
            ReadError::io(path, ErrorKind::NotFound.into()),
            ReadError::Io { path: found, source }
                if found == path && source.kind() == ErrorKind::NotFound
        ));
        assert!(matches!(
            SaveError::io(path, ErrorKind::PermissionDenied.into()),
            SaveError::Io { path: found, source }
                if found == path && source.kind() == ErrorKind::PermissionDenied
        ));
    }
}

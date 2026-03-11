use std::{ffi::NulError, str::Utf8Error, sync::PoisonError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to acquire database lock")]
    DatabaseLockError,
    #[error("failed to convert Rust string to C string: {0}")]
    NulError(#[from] NulError),
    #[error("leveldb error contains invalid UTF-8 bytes: {0}")]
    InvalidUtf8(#[from] Utf8Error),
    #[error("leveldb error: {0}")]
    LevelDbError(String),
    #[error("{0}")]
    NbtError(#[from] nbtx::Error),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("invalid packed array index size: {0}")]
    InvalidBitSize(u8),
    #[error("invalid {0}")]
    Invalid(&'static str),
    #[error("an exception occurred within LevelDB")]
    Exception,
    #[error("unknown error")]
    Unknown,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Error {
        Error::DatabaseLockError
    }
}

// #[cfg(feature = "rusty-leveldb")]
// impl From<rusty_leveldb::Status> for Error {
//     fn from(err: rusty_leveldb::Status) -> Error {
//         Error::LevelDbError(err.err)
//     }
// }

pub type Result<T> = std::result::Result<T, Error>;

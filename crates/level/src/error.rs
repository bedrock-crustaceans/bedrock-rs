use std::sync::PoisonError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("requested {requested} bits for array, but requires at least {required}")]
    TooFewBits { requested: u32, required: u32 },
    #[error("failed to acquire database lock")]
    DatabaseLockError,
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
    #[error("unknown error")]
    Unknown,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Error {
        Error::DatabaseLockError
    }
}

impl From<rusty_leveldb::Status> for Error {
    fn from(err: rusty_leveldb::Status) -> Error {
        Error::LevelDbError(err.err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

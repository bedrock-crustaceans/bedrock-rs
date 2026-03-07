use std::{ffi::NulError, str::Utf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
    InvalidIndexSize(u8),
    #[error("invalid {0}")]
    Invalid(&'static str),
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

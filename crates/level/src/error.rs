use thiserror::Error;

#[derive(Debug, Error)]
pub enum LevelError {
    #[error("invalid value")]
    Invalid,
    #[error("unknown error")]
    Unknown
}

pub type Result<T> = std::result::Result<T, LevelError>;
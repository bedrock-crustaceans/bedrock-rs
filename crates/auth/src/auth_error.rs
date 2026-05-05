use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("jwt error: {0}")]
    JWTError(#[from] jsonwebtoken::errors::Error),
    #[error("missing field: {0}")]
    Missing(&'static str),
    #[error("unsupported")]
    Unsupported,
}

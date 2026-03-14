use bedrockrs_network::error::ConnectionError;
use thiserror::Error;

pub enum StartError {}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Connection Error: {0}")]
    ConnectionError(#[from] ConnectionError),
    #[error("Login aborted, reason: {reason}")]
    Abort { reason: String },
    #[error("Wrong protocol version (client: {client}, server: {server:?})")]
    WrongProtocolVersion { client: i32, server: Vec<i32> },
    #[error("Format Error: {0}")]
    FormatError(String),
}

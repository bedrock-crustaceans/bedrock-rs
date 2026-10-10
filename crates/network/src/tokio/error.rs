use crate::error::{LoginError, NetworkCodecError};
use crate::raknet::RakNetGamePacket;
use raknet_tokio::prelude::{RakClientError, RakServerError, RakSessionError};
use std::io::Error as IOError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("Address bind error")]
    AddrBindError,
    #[error("No transport configured")]
    NoTransport,
    #[error("Already Online")]
    AlreadyOnline,
    #[error("Not Listening")]
    NotListening,
    #[error("Transport Error: {0}")]
    TransportListenerError(#[from] TransportLayerError),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("NetworkCodec Error: {0}")]
    NetworkCodecError(#[from] NetworkCodecError),
    #[error("Connection Closed")]
    ConnectionClosed,
    #[error("Transport Error: {0}")]
    TransportError(#[from] TransportLayerError),
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
}

#[derive(Error, Debug)]
pub enum TransportLayerError {
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
    #[error("RakNet Error: {0}")]
    RakNetError(#[from] RakNetError),
    #[cfg(feature = "nethernet-tokio")]
    #[error("NetherNet Error: {0}")]
    NetherNet(#[from] nethernet_tokio::NetherError),
    // #[error("Quic Error: {0}")]
    // QuicError(#[from] QuicError),
}

#[derive(Error, Debug)]
pub enum RakNetError {
    #[error("Session Error: {0}")]
    SessionError(#[from] RakSessionError),
    #[error("Server Error: {0}")]
    ServerError(#[from] RakServerError),
    #[error("Client Error: {0}")]
    ClientError(#[from] RakClientError),
    #[error("Invalid RakNet Header (expected: {expected}, got: {0})", expected = RakNetGamePacket::ID)]
    InvalidRakNetHeader(u8),
    #[error("Format Error: {0}")]
    FormatError(&'static str),
}

// #[derive(Error, Debug, Clone)]
// pub enum QuicError {
//     // #[error("Stream Error: {0}")]
//     // StreamError(s2n_quic::stream::Error),
// }

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Login Error: {0}")]
    Login(#[from] LoginError),
    #[error("Connection Error: {0}")]
    Connection(#[from] ConnectionError),
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("RakNet Error: {0}")]
    RakNet(#[from] RakNetError),
    #[error("Session Error: {0}")]
    Session(#[from] SessionError),
    #[error("Auth Error: {0}")]
    Auth(#[from] bedrock_auth::AuthError),
    #[error("Timed out connecting")]
    Timeout,
}

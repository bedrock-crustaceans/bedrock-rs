use io::Error as IOError;
use std::io;

use bedrockrs_proto_core::error::ProtoCodecError;
use thiserror::Error;

use crate::info::RAKNET_GAMEPACKET_ID;

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("Address bind error")]
    AddrBindError,
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
    #[error("Quic Error: {0}")]
    QuicError(#[from] QuicError),
}

#[derive(Error, Debug, Clone)]
pub enum RakNetError {
    #[error("Receive Error: {0}")]
    RecvError(#[from] rak_rs::connection::RecvError),
    #[error("Send Error: {0}")]
    SendError(#[from] rak_rs::connection::queue::SendQueueError),
    #[error("Server Error: {0}")]
    ServerError(#[from] rak_rs::error::server::ServerError),
    #[error("Invalid RakNet Header (expected: {RAKNET_GAMEPACKET_ID}, got: {0})")]
    InvalidRakNetHeader(u8),
    #[error("Format Error: {0}")]
    FormatError(&'static str),
}

#[derive(Error, Debug, Clone)]
pub enum QuicError {
    // #[error("Stream Error: {0}")]
    // StreamError(s2n_quic::stream::Error),
}

#[derive(Error, Debug)]
pub enum NetworkCodecError {
    #[error("ProtoCodec Error: {0}")]
    ProtoCodecError(#[from] ProtoCodecError),
    #[error("Compression Error: {0}")]
    CompressionError(#[from] CompressionError),
    #[error("Encryption Error: {0}")]
    EncryptionError(#[from] EncryptionError),
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Zlib Error: {0}")]
    ZlibError(IOError),
    #[error("Snappy Error: {0}")]
    SnappyError(IOError),
    #[error("Unknown Compression Method: {0}")]
    UnknownCompressionMethod(u8),
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encrypted data length invalid (len={0}, expected > 8 bytes)")]
    InvalidLength(usize),
    #[error("Encrypted data trailer invalid")]
    InvalidTrailer,
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
}

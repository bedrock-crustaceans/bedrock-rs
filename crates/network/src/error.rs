use bedrock_protocol_core::error::{PacketCodecError, ProtoCodecError};
use io::Error as IOError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkCodecError {
    #[error("PacketCodec Error: {0}")]
    PacketCodecError(#[from] PacketCodecError),
    #[error("ProtoCodec Error: {0}")]
    ProtoCodecError(#[from] ProtoCodecError),
    #[error("Compression Error: {0}")]
    CompressionError(#[from] CompressionError),
    #[error("Encryption Error: {0}")]
    EncryptionError(#[from] EncryptionError),
    #[error("IO Error: {0}")]
    IOError(#[from] IOError),
    #[error("Batch holds a zero-length packet")]
    EmptyPacket,
    #[error("Packet length {declared} exceeds the {remaining} bytes left in the batch")]
    PacketLengthOutOfBounds { declared: u32, remaining: usize },
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Zlib Error: {0}")]
    ZlibError(IOError),
    #[error("Snappy Error: {0}")]
    SnappyError(IOError),
    #[error("Unknown Compression Method: {0}")]
    UnknownCompressionMethod(u8),
    #[error("Unknown Network Compression Algorithm: {0}")]
    UnknownNetworkAlgorithm(u16),
    #[error("Batch method {found} does not match negotiated method {negotiated}")]
    UnexpectedMethod { negotiated: u8, found: u8 },
    #[error("Decompressed batch exceeds {0} bytes")]
    TooLarge(usize),
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

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Auth Error: {0}")]
    Auth(#[from] bedrock_auth::AuthError),
    #[error("Unexpected packet while {state}")]
    UnexpectedPacket { state: &'static str },
    #[error("Client protocol {client} does not match server protocol {server}")]
    ProtocolMismatch { client: i32, server: i32 },
    #[error("Login is not authenticated")]
    NotAuthenticated,
    #[error("Login protocol {login} differs from the requested protocol {requested}")]
    ProtocolChanged { requested: i32, login: i32 },
    #[error("Server rejected the login: {0}")]
    LoginRejected(crate::login::LoginStatus),
    #[error("Server disconnected during login: {0}")]
    Disconnected(crate::login::DisconnectInfo),
}

use bedrock_network_core::error::{PacketCodecError, ProtoCodecError};
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
    #[error("Datagram starts with {0:#04x} instead of the game packet header")]
    InvalidGamePacketHeader(u8),
    #[error("Packet length {declared} exceeds the {remaining} bytes left in the batch")]
    PacketLengthOutOfBounds { declared: u32, remaining: usize },
    #[error("Batch packet count exceeds the {0} packet limit")]
    PacketCountExceeded(usize),
}

#[derive(Error, Debug)]
pub enum MotdError {
    #[error("Pong is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("Pong has no {0} field")]
    MissingField(&'static str),
    #[error("Pong field {0} is not a number")]
    InvalidNumber(&'static str),
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
    #[error("Decompressed batch size exceeds the {0} byte limit")]
    DecompressedSizeExceeded(usize),
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
    #[error("Compression Error: {0}")]
    Compression(#[from] CompressionError),
    #[error("Client protocol {client} does not match server protocol {server}")]
    ProtocolMismatch { client: i32, server: i32 },
    #[error("Login is not authenticated")]
    NotAuthenticated,
    #[error("Login identity key differs from the one the transport authenticated")]
    TransportIdentityMismatch,
}

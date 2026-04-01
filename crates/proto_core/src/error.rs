use std::convert::Infallible;
use std::io::Error as IOError;
use std::num::{ParseIntError, TryFromIntError};
use std::string::FromUtf8Error;

use base64::DecodeError as Base64DecodeError;
use jsonwebtoken::errors::Error as JwtError;
use serde_json::error::Error as JsonError;
use strum::ParseError;
use thiserror::Error;
use uuid::Error as UuidError;

#[derive(Error, Debug)]
pub enum ProtoCodecError {
    #[error("IOError occurred: {0}")]
    IOError(#[from] IOError),
    #[error("Unread bytes remaining: {0} bytes left")]
    LeftOvers(usize),
    #[error("NBT Error: {0}")]
    NbtError(#[from] nbtx::Error),
    #[error("Error while reading UTF8 encoded String: {0}")]
    UTF8Error(#[from] FromUtf8Error),
    #[error("Error while converting integers: {0}")]
    FromIntError(#[from] TryFromIntError),
    #[error("Json Error: {0}")]
    JsonError(#[from] JsonError),
    #[error("Jwt Error: {0}")]
    JwtError(#[from] JwtError),
    #[error("Uuid Error: {0}")]
    UuidError(#[from] UuidError),
    #[error("Base64 decoding Error: {0}")]
    Base64DecodeError(#[from] Base64DecodeError),
    #[error("XUID could not be parsed : {0}")]
    XuidParseError(#[from] ParseIntError),
    /// TODO: This likely hurts performance, but it is *kinda* good for debugging
    #[error("parse value `{0}` to enum variant for {1} enum")]
    InvalidEnumID(String, &'static str),
    #[error("Got an unknown/invalid packet id: {0}")]
    InvalidPacketID(u16),
    #[error("Expected format got mismatched: {0}")]
    FormatMismatch(&'static str),
    #[error("Strum Parse Error: {0}")]
    StrumParseError(#[from] ParseError),
    #[error("Expected Some in: {0}")]
    ExpectedSome(&'static str),
}

impl From<Infallible> for ProtoCodecError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

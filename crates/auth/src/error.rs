use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("http error: {0}")]
    Http(#[from] crate::http::HttpError),
    #[error("jwt error: {0}")]
    Jwt(#[from] crate::jwt::JwtError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("client data is not utf-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("missing field: {0}")]
    Missing(&'static str),
    #[error("connection request ended before a length prefix")]
    Truncated,
    #[error("length prefix {declared} exceeds the {remaining} bytes remaining")]
    LengthOutOfBounds { declared: u32, remaining: usize },
    #[error("{0} unread bytes after the connection request")]
    TrailingBytes(usize),
    #[error("field of {0} bytes does not fit a u32 length prefix")]
    FieldTooLong(usize),
    #[error("no JWKS key with kid {0}")]
    UnknownKeyId(String),
    #[error("certificate chain has {0} tokens, expected 1 or 3")]
    ChainLength(usize),
    #[error("public key is not a base64 P-384 SubjectPublicKeyInfo")]
    InvalidPublicKey,
    #[error("chain claims an XUID but is not signed by the chain root")]
    XuidWithoutXboxLive,
    #[error("chain is signed by the chain root but carries no XUID")]
    XboxLiveWithoutXuid,
    #[error("guest logins are not supported")]
    GuestLogin,
    #[error("invalid client data: {0}")]
    ClientData(#[from] ClientDataError),
    #[error("unsupported")]
    Unsupported,
}

#[derive(Error, Debug)]
pub enum ClientDataError {
    #[error("DeviceOS must be between 1 and 15, got {0}")]
    DeviceOs(i32),
    #[error("GameVersion must only contain digits and dots, got {0:?}")]
    GameVersion(String),
    #[error("LanguageCode must be a BCP-47 language tag, got {0:?}")]
    LanguageCode(String),
    #[error("PlatformOfflineId must be a UUID or empty, got {0:?}")]
    PlatformOfflineId(String),
    #[error("PlatformOnlineId must be an unsigned integer or empty, got {0:?}")]
    PlatformOnlineId(String),
    #[error("SelfSignedId must be a UUID or empty, got {0:?}")]
    SelfSignedId(String),
    #[error("ServerAddress must be host:port or an http(s) NetherNet address, got {0:?}")]
    ServerAddress(String),
    #[error("{field} is not standard base64: {source}")]
    Base64 {
        field: &'static str,
        source: base64::DecodeError,
    },
    #[error("{field} decodes to {actual} bytes, expected {expected}")]
    ImageSize {
        field: &'static str,
        expected: i64,
        actual: usize,
    },
    #[error("PlayFabId must be lowercase hex, got {0:?}")]
    PlayFabId(String),
    #[error("skin animation type must be between 0 and 3, got {0}")]
    AnimationType(i32),
    #[error("{field} does not decode to a JSON object")]
    NotJsonObject { field: &'static str },
    #[error("SkinId must not be empty")]
    EmptySkinId,
    #[error("UIProfile must be between 0 and 2, got {0}")]
    UiProfile(i32),
}

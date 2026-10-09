pub mod auth_data;
pub mod authentication;
pub mod chain;
pub mod client_data;
pub mod connection_request;
pub mod error;
pub mod http;
mod jwt;
pub mod oidc;

#[cfg(test)]
mod test_support;

pub use auth_data::{AuthData, AuthDataClaims, AuthPayload, AuthType, identity_from_xuid};
pub use authentication::{Authentication, Identity};
pub use chain::{ChainRoot, MOJANG_PUBLIC_KEY};
pub use client_data::ClientData;
pub use connection_request::{ConnectionRequest, Login};
pub use error::{AuthError, ClientDataError};
pub use http::{AsyncHttpClient, HttpClient, HttpError};
pub use jwt::{Jwk, JwkSet, JwtError};
pub use oidc::AuthOIDC;

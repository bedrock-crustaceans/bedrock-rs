use crate::authentication::{Authentication, Identity};
use crate::chain::{ChainRoot, unverified_identity, verify_chain};
use crate::error::AuthError;
use crate::http::{AsyncHttpClient, HttpClient};
use crate::jwt::{Algorithm, Expiry, Rules, Token};
use crate::oidc::AuthOIDC;
use md5::{Digest, Md5};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::time::Instant;
use uuid::{Builder, Uuid};

#[derive(Deserialize_repr, Serialize_repr, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthType {
    Online = 0,
    Guest = 1,
    Offline = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthPayload {
    Chain(Vec<String>),
    Token(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct RawAuthData {
    #[serde(rename = "AuthenticationType", default = "legacy_auth_type")]
    auth_type: AuthType,
    #[serde(rename = "Certificate", default)]
    certificate: String,
    #[serde(rename = "Token", default)]
    token: String,
    #[serde(default, skip_serializing)]
    chain: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
struct Certificate {
    chain: Vec<String>,
}

fn legacy_auth_type() -> AuthType {
    AuthType::Online
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthData {
    pub auth_type: AuthType,
    pub auth_payload: AuthPayload,
}

impl AuthData {
    pub fn unverified_identity(&self) -> Result<Identity, AuthError> {
        match &self.auth_payload {
            AuthPayload::Chain(chain) => unverified_identity(chain),
            AuthPayload::Token(token) => Ok(Token::parse(token)?
                .unverified_claims::<AuthDataClaims>()?
                .into()),
        }
    }
}

impl<'de> Deserialize<'de> for AuthData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAuthData::deserialize(deserializer)?;
        let auth_payload = if !raw.token.is_empty() {
            AuthPayload::Token(raw.token)
        } else if !raw.certificate.is_empty() {
            let certificate: Certificate =
                serde_json::from_str(&raw.certificate).map_err(serde::de::Error::custom)?;
            AuthPayload::Chain(certificate.chain)
        } else {
            AuthPayload::Chain(raw.chain)
        };

        Ok(Self {
            auth_payload,
            auth_type: raw.auth_type,
        })
    }
}

impl Serialize for AuthData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (certificate, token) = match &self.auth_payload {
            AuthPayload::Chain(chain) => (
                Certificate {
                    chain: chain.clone(),
                },
                String::new(),
            ),
            AuthPayload::Token(token) => (Certificate::default(), token.clone()),
        };
        RawAuthData {
            auth_type: self.auth_type,
            certificate: serde_json::to_string(&certificate).map_err(serde::ser::Error::custom)?,
            token,
            chain: Vec::new(),
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AuthDataClaims {
    pub mid: String,
    pub xid: String,
    pub xname: String,
    pub cpk: String,
    #[serde(default)]
    pub tid: String,
    #[serde(default)]
    pub leguuid: Option<String>,
}

impl From<AuthDataClaims> for Identity {
    fn from(claims: AuthDataClaims) -> Self {
        let identity = claims.leguuid.or_else(|| {
            (!claims.xid.is_empty()).then(|| identity_from_xuid(&claims.xid).to_string())
        });
        Self {
            identity,
            title_id: (!claims.tid.is_empty()).then_some(claims.tid),
            xuid: claims.xid,
            display_name: claims.xname,
            playfab_id: Some(claims.mid),
            public_key: claims.cpk,
        }
    }
}

pub fn identity_from_xuid(xuid: &str) -> Uuid {
    let digest = Md5::new()
        .chain_update(b"pocket-auth-1-xuid:")
        .chain_update(xuid.as_bytes())
        .finalize();
    Builder::from_md5_bytes(digest.into()).into_uuid()
}

impl AuthData {
    pub fn validate_refreshing(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
        client: &impl HttpClient,
    ) -> Result<Authentication, AuthError> {
        let result = self.validate(oidc, chain_root);
        match refresh_target(&result, oidc) {
            Some(oidc) => {
                oidc.refresh(client)?;
                self.validate(Some(oidc), chain_root)
            }
            None => result,
        }
    }

    pub async fn validate_refreshing_async(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
        client: &impl AsyncHttpClient,
    ) -> Result<Authentication, AuthError> {
        let result = self.validate(oidc, chain_root);
        match refresh_target(&result, oidc) {
            Some(oidc) => {
                oidc.refresh_async(client).await?;
                self.validate(Some(oidc), chain_root)
            }
            None => result,
        }
    }

    pub fn validate(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
    ) -> Result<Authentication, AuthError> {
        if self.auth_type == AuthType::Guest {
            return Err(AuthError::GuestLogin);
        }
        match &self.auth_payload {
            AuthPayload::Chain(chain) => verify_chain(chain, chain_root),
            AuthPayload::Token(token) => Self::validate_token(token, &self.auth_type, oidc),
        }
    }

    fn validate_token(
        token: &str,
        auth_type: &AuthType,
        oidc: Option<&AuthOIDC>,
    ) -> Result<Authentication, AuthError> {
        match (oidc, auth_type) {
            (Some(oidc), AuthType::Online) => {
                let identity: Identity = Self::validate_online_token(token, oidc)?.into();
                if identity.xuid.is_empty() {
                    Ok(Authentication::Unauthenticated(identity))
                } else {
                    Ok(Authentication::Authenticated(identity))
                }
            }
            _ => {
                let claims = Self::decode_unverified_token(token)?;
                Ok(Authentication::Unauthenticated(claims.into()))
            }
        }
    }

    fn validate_online_token(token: &str, oidc: &AuthOIDC) -> Result<AuthDataClaims, AuthError> {
        let token = Token::parse(token)?;
        token.require_algorithm(Algorithm::RS256)?;
        let kid = token.header.kid.clone().ok_or(AuthError::Missing("kid"))?;
        let key = oidc
            .jwk(&kid)
            .ok_or(AuthError::UnknownKeyId(kid))?
            .rs256_key()?;
        let rules = Rules {
            expiry: Expiry::Required,
            check_not_before: true,
            issuer: Some(&oidc.issuer),
            audience: Some(&oidc.audience),
        };
        Ok(token.verify(&key, &rules)?)
    }

    fn decode_unverified_token(token: &str) -> Result<AuthDataClaims, AuthError> {
        Ok(Token::parse(token)?.unverified_claims()?)
    }
}

fn refresh_target<'a>(
    result: &Result<Authentication, AuthError>,
    oidc: Option<&'a AuthOIDC>,
) -> Option<&'a AuthOIDC> {
    match (result, oidc) {
        (Err(AuthError::UnknownKeyId(kid)), Some(oidc))
            if oidc.claim_refresh(kid, Instant::now()) =>
        {
            Some(oidc)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::{JwkSet, JwtError};
    use crate::test_support::{FakeHttp, TestKey, TestRsaKey, block_on, valid_window, with_window};
    use serde_json::json;
    use std::time::Duration;

    fn offline_token_data(key: &TestKey) -> AuthData {
        let claims = json!({ "mid": "", "xid": "", "xname": "Steve", "cpk": key.public_base64() });
        AuthData {
            auth_type: AuthType::Offline,
            auth_payload: AuthPayload::Token(key.sign(&with_window(claims, valid_window()))),
        }
    }

    #[test]
    fn unverified_token_reports_unauthenticated() {
        let key = TestKey::from_seed(1);

        let authentication = offline_token_data(&key)
            .validate(None, &ChainRoot::default())
            .unwrap();

        let Authentication::Unauthenticated(identity) = authentication else {
            panic!("an unverified token must not report authenticated: {authentication:?}");
        };
        assert_eq!(identity.display_name, "Steve");
        assert_eq!(identity.public_key, key.public_base64());
    }

    #[test]
    fn chain_auth_data_is_verified() {
        let key = TestKey::from_seed(1);
        let claims = json!({
            "identityPublicKey": key.public_base64(),
            "extraData": { "XUID": "", "displayName": "Steve" },
        });
        let data = AuthData {
            auth_type: AuthType::Offline,
            auth_payload: AuthPayload::Chain(vec![key.sign(&with_window(claims, valid_window()))]),
        };

        let authentication = data.validate(None, &ChainRoot::default()).unwrap();

        assert!(!authentication.is_authenticated());
        assert_eq!(authentication.identity().public_key, key.public_base64());
    }

    fn online_claims() -> serde_json::Value {
        let claims = json!({
            "mid": "playfab", "xid": "2535400000000001", "xname": "Steve", "cpk": "cpk",
            "iss": "https://authorization.franchise.minecraft-services.net",
            "aud": AuthOIDC::AUDIENCE,
        });
        with_window(claims, valid_window())
    }

    fn online_token_data(key: &TestRsaKey) -> AuthData {
        AuthData {
            auth_type: AuthType::Online,
            auth_payload: AuthPayload::Token(key.sign(&online_claims())),
        }
    }

    #[test]
    fn online_token_signed_with_another_algorithm_is_rejected() {
        let key = TestKey::from_seed(1);
        let oidc = oidc_trusting(key.jwks("current"));
        let data = AuthData {
            auth_type: AuthType::Online,
            auth_payload: AuthPayload::Token(key.sign_with_kid(&online_claims(), "current")),
        };

        let result = data.validate(Some(&oidc), &ChainRoot::default());

        assert!(
            matches!(
                result,
                Err(AuthError::Jwt(JwtError::InvalidAlgorithm {
                    expected: Algorithm::RS256,
                    ..
                }))
            ),
            "{result:?}"
        );
    }

    fn oidc_trusting(jwks: JwkSet) -> AuthOIDC {
        AuthOIDC::new(
            "https://authorization.franchise.minecraft-services.net",
            AuthOIDC::AUDIENCE,
            "https://jwks.invalid",
            jwks,
        )
    }

    #[test]
    fn online_token_signed_by_jwks_key_is_authenticated() {
        let key = TestRsaKey::shared();
        let oidc = oidc_trusting(key.jwks("current"));

        let authentication = online_token_data(key)
            .validate(Some(&oidc), &ChainRoot::default())
            .unwrap();

        assert!(authentication.is_authenticated());
        assert_eq!(authentication.identity().xuid, "2535400000000001");
    }

    fn identity_of_online_token(claims: serde_json::Value) -> Identity {
        let key = TestRsaKey::shared();
        let oidc = oidc_trusting(key.jwks("current"));
        let data = AuthData {
            auth_type: AuthType::Online,
            auth_payload: AuthPayload::Token(key.sign(&claims)),
        };
        data.validate(Some(&oidc), &ChainRoot::default())
            .unwrap()
            .into_identity()
    }

    #[test]
    fn online_token_identity_is_derived_from_the_xuid() {
        let identity = identity_of_online_token(online_claims());

        assert_eq!(
            identity.identity.as_deref(),
            Some("058728db-7a88-3a63-af82-7174a02a2e35")
        );
    }

    #[test]
    fn online_token_legacy_uuid_wins_over_the_xuid() {
        let mut claims = online_claims();
        claims["leguuid"] = json!("6a3f0c4e-1b2d-4e5f-8a9b-0c1d2e3f4a5b");

        let identity = identity_of_online_token(claims);

        assert_eq!(
            identity.identity.as_deref(),
            Some("6a3f0c4e-1b2d-4e5f-8a9b-0c1d2e3f4a5b")
        );
    }

    #[test]
    fn online_token_without_xuid_or_legacy_uuid_has_no_identity() {
        let mut claims = online_claims();
        claims["xid"] = json!("");

        assert_eq!(identity_of_online_token(claims).identity, None);
    }

    #[test]
    fn online_token_title_id_is_kept() {
        let mut claims = online_claims();
        claims["tid"] = json!("20CA2");

        assert_eq!(
            identity_of_online_token(claims).title_id.as_deref(),
            Some("20CA2")
        );
    }

    #[test]
    fn online_token_without_xuid_is_unauthenticated() {
        let key = TestRsaKey::shared();
        let oidc = oidc_trusting(key.jwks("current"));
        let mut claims = online_claims();
        claims["xid"] = json!("");
        let data = AuthData {
            auth_type: AuthType::Online,
            auth_payload: AuthPayload::Token(key.sign(&claims)),
        };

        let authentication = data.validate(Some(&oidc), &ChainRoot::default()).unwrap();

        assert!(
            !authentication.is_authenticated(),
            "a token without an XUID must not report authenticated: {authentication:?}"
        );
    }

    #[test]
    fn guest_token_is_rejected_even_when_signed_by_jwks_key() {
        let key = TestRsaKey::shared();
        let oidc = oidc_trusting(key.jwks("current"));
        let data = AuthData {
            auth_type: AuthType::Guest,
            ..online_token_data(key)
        };

        let result = data.validate(Some(&oidc), &ChainRoot::default());

        assert!(matches!(result, Err(AuthError::GuestLogin)), "{result:?}");
    }

    #[test]
    fn guest_chain_is_rejected() {
        let key = TestKey::from_seed(1);
        let claims = json!({
            "identityPublicKey": key.public_base64(),
            "extraData": { "XUID": "", "displayName": "Steve" },
        });
        let data = AuthData {
            auth_type: AuthType::Guest,
            auth_payload: AuthPayload::Chain(vec![key.sign(&with_window(claims, valid_window()))]),
        };

        let result = data.validate(None, &ChainRoot::default());

        assert!(matches!(result, Err(AuthError::GuestLogin)), "{result:?}");
    }

    #[test]
    fn online_token_with_rotated_kid_refetches_jwks_once() {
        let key = TestRsaKey::shared();
        let mut oidc = oidc_trusting(key.jwks("previous"));
        oidc.refresh_interval = Duration::ZERO;
        let http = FakeHttp::serving(&[(
            "https://jwks.invalid",
            serde_json::to_value(key.jwks("current")).unwrap(),
        )]);

        let authentication = online_token_data(key)
            .validate_refreshing(Some(&oidc), &ChainRoot::default(), &http)
            .unwrap();

        assert!(authentication.is_authenticated());
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[test]
    fn online_token_with_rotated_kid_refetches_jwks_once_async() {
        let key = TestRsaKey::shared();
        let mut oidc = oidc_trusting(key.jwks("previous"));
        oidc.refresh_interval = Duration::ZERO;
        let http = FakeHttp::serving(&[(
            "https://jwks.invalid",
            serde_json::to_value(key.jwks("current")).unwrap(),
        )]);

        let authentication = block_on(online_token_data(key).validate_refreshing_async(
            Some(&oidc),
            &ChainRoot::default(),
            &http,
        ))
        .unwrap();

        assert!(authentication.is_authenticated());
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[test]
    fn online_token_with_unknown_kid_inside_interval_is_not_refetched() {
        let key = TestRsaKey::shared();
        let oidc = oidc_trusting(key.jwks("previous"));
        let http = FakeHttp::serving(&[]);

        let result =
            online_token_data(key).validate_refreshing(Some(&oidc), &ChainRoot::default(), &http);

        assert!(
            matches!(&result, Err(AuthError::UnknownKeyId(kid)) if kid == "current"),
            "{result:?}"
        );
        assert!(http.requests().is_empty());
    }
}

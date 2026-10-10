use crate::authentication::{Authentication, Identity};
use crate::error::AuthError;
use crate::jwt::{Algorithm, Expiry, Header, Rules, Token, VerifyingKey, encode_es384};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use p384::PublicKey;
use p384::SecretKey;
use p384::pkcs8::{DecodePublicKey, EncodePublicKey};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub const MOJANG_PUBLIC_KEY: &str = "MHYwEAYHKoZIzj0CAQYFK4EEACIDYgAECRXueJeTDqNRRgJi/vlRufByu/2G0i2Ebt6YMar5QX/R0DIIyrJMcUpruK4QveTfJSTp3Shlq4Gk34cD/4GUWwkv0DVuzeuB+tXija7HBxii03NHDbPAD0AKnLr2wdAp";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainRoot(Cow<'static, str>);

impl ChainRoot {
    pub const MOJANG: Self = Self(Cow::Borrowed(MOJANG_PUBLIC_KEY));

    pub fn new(public_key_base64: impl Into<String>) -> Self {
        Self(Cow::Owned(public_key_base64.into()))
    }

    pub fn public_key_base64(&self) -> &str {
        &self.0
    }
}

impl Default for ChainRoot {
    fn default() -> Self {
        Self::MOJANG
    }
}

const CHAIN_ISSUER: &str = "Mojang";

#[derive(Deserialize)]
struct LinkClaims {
    #[serde(rename = "identityPublicKey")]
    identity_public_key: Option<String>,
    #[serde(rename = "extraData")]
    extra_data: Option<ExtraData>,
}

#[derive(Deserialize)]
struct ExtraData {
    #[serde(rename = "XUID", default)]
    xuid: String,
    #[serde(rename = "displayName", default)]
    display_name: String,
    identity: Option<String>,
    #[serde(rename = "titleId")]
    title_id: Option<String>,
}

pub(crate) fn verify_chain(
    chain: &[String],
    root: &ChainRoot,
) -> Result<Authentication, AuthError> {
    let (first, rest) = match chain {
        [first, rest @ ..] if matches!(rest.len(), 0 | 2) => (first, rest),
        _ => return Err(AuthError::ChainLength(chain.len())),
    };

    let x5u = Token::parse(first)?
        .header
        .x5u
        .ok_or(AuthError::Missing("x5u"))?;
    let mut claims = verify_link(first, &x5u, false)?;
    let signed_by_root = match (&claims.identity_public_key, rest.is_empty()) {
        (Some(next), false) => {
            parse_public_key(next)? == parse_public_key(root.public_key_base64())?
        }
        _ => false,
    };
    for token in rest {
        let key = claims
            .identity_public_key
            .ok_or(AuthError::Missing("identityPublicKey"))?;
        claims = verify_link(token, &key, true)?;
    }

    let extra = claims.extra_data.ok_or(AuthError::Missing("extraData"))?;
    let identity = Identity {
        xuid: extra.xuid,
        display_name: extra.display_name,
        identity: extra.identity,
        title_id: extra.title_id,
        playfab_id: None,
        public_key: claims
            .identity_public_key
            .ok_or(AuthError::Missing("identityPublicKey"))?,
    };

    match (chain.len(), signed_by_root, identity.xuid.is_empty()) {
        (1, ..) | (_, false, true) => Ok(Authentication::Unauthenticated(identity)),
        (_, true, false) => Ok(Authentication::Authenticated(identity)),
        (_, false, false) => Err(AuthError::XuidWithoutXboxLive),
        (_, true, true) => Err(AuthError::XboxLiveWithoutXuid),
    }
}

fn verify_link(token: &str, key: &str, check_issuer: bool) -> Result<LinkClaims, AuthError> {
    let rules = Rules {
        expiry: Expiry::Required,
        check_not_before: true,
        issuer: check_issuer.then_some(CHAIN_ISSUER),
        audience: None,
    };
    Ok(Token::parse(token)?.verify(&verifying_key(key)?, &rules)?)
}

pub(crate) fn verifying_key(public_key_base64: &str) -> Result<VerifyingKey, AuthError> {
    Ok(VerifyingKey::Es384(
        (&parse_public_key(public_key_base64)?).into(),
    ))
}

pub(crate) fn encode_es384_with_x5u(
    key: &SecretKey,
    claims: &impl Serialize,
) -> Result<String, AuthError> {
    let header = Header {
        alg: Algorithm::ES384.name().into(),
        kid: None,
        x5u: Some(encode_public_key(&key.public_key())?),
    };
    Ok(encode_es384(&header, claims, key)?)
}

pub fn encode_public_key(key: &PublicKey) -> Result<String, AuthError> {
    let der = key
        .to_public_key_der()
        .map_err(|_| AuthError::InvalidPublicKey)?;
    Ok(STANDARD.encode(der.as_bytes()))
}

pub fn parse_public_key(public_key_base64: &str) -> Result<PublicKey, AuthError> {
    let der = STANDARD
        .decode(public_key_base64)
        .map_err(|_| AuthError::InvalidPublicKey)?;
    PublicKey::from_public_key_der(&der).map_err(|_| AuthError::InvalidPublicKey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::JwtError;
    use crate::test_support::{TestKey, now, valid_window, with_window};
    use serde_json::{Value, json};

    struct ChainKeys {
        root: TestKey,
        intermediate: TestKey,
        client: TestKey,
    }

    fn chain_keys() -> ChainKeys {
        ChainKeys {
            root: TestKey::from_seed(10),
            intermediate: TestKey::from_seed(11),
            client: TestKey::from_seed(12),
        }
    }

    fn extra_data(xuid: &str) -> Value {
        json!({
            "XUID": xuid,
            "identity": "2535f2a0-0000-3000-8000-000000000001",
            "displayName": "Steve",
            "titleId": "896928775",
        })
    }

    fn xbox_chain(keys: &ChainKeys, signer: &TestKey, window: Value) -> Vec<String> {
        vec![
            keys.client.sign(&with_window(
                json!({ "identityPublicKey": signer.public_base64(), "certificateAuthority": true }),
                valid_window(),
            )),
            signer.sign(&with_window(
                json!({ "identityPublicKey": keys.intermediate.public_base64(), "iss": "Mojang" }),
                valid_window(),
            )),
            keys.intermediate.sign(&with_window(
                json!({
                    "identityPublicKey": keys.client.public_base64(),
                    "extraData": extra_data("2535400000000001"),
                    "iss": "Mojang",
                }),
                window,
            )),
        ]
    }

    fn root_of(keys: &ChainKeys) -> ChainRoot {
        ChainRoot::new(keys.root.public_base64())
    }

    fn assert_jwt_error(result: Result<Authentication, AuthError>, expected: JwtError) {
        match result {
            Err(AuthError::Jwt(error)) => assert_eq!(
                std::mem::discriminant(&error),
                std::mem::discriminant(&expected),
                "expected {expected:?}, got {error:?}"
            ),
            other => panic!("expected {expected:?}, got {other:?}"),
        }
    }

    #[test]
    fn mojang_public_key_round_trips() {
        let key = parse_public_key(MOJANG_PUBLIC_KEY).unwrap();

        assert_eq!(encode_public_key(&key).unwrap(), MOJANG_PUBLIC_KEY);
    }

    #[test]
    fn xbox_chain_to_root_is_authenticated() {
        let keys = chain_keys();
        let chain = xbox_chain(&keys, &keys.root, valid_window());

        let authentication = verify_chain(&chain, &root_of(&keys)).unwrap();

        assert_eq!(
            authentication,
            Authentication::Authenticated(Identity {
                xuid: "2535400000000001".into(),
                display_name: "Steve".into(),
                identity: Some("2535f2a0-0000-3000-8000-000000000001".into()),
                title_id: Some("896928775".into()),
                playfab_id: None,
                public_key: keys.client.public_base64(),
            })
        );
    }

    #[test]
    fn xbox_chain_with_tampered_signature_is_rejected() {
        let keys = chain_keys();
        let mut chain = xbox_chain(&keys, &keys.root, valid_window());
        let forged = keys.client.sign(&with_window(
            json!({
                "identityPublicKey": keys.client.public_base64(),
                "extraData": extra_data("2535400000000002"),
                "iss": "Mojang",
            }),
            valid_window(),
        ));
        let signature = forged.rsplit('.').next().unwrap_or_default();
        let (signed, _) = chain[2].rsplit_once('.').unwrap();
        chain[2] = format!("{signed}.{signature}");

        let result = verify_chain(&chain, &root_of(&keys));

        assert_jwt_error(result, JwtError::InvalidSignature);
    }

    #[test]
    fn xbox_chain_with_expired_token_is_rejected() {
        let keys = chain_keys();
        let expired = json!({ "nbf": now() - 7200, "exp": now() - 3600 });
        let chain = xbox_chain(&keys, &keys.root, expired);

        let result = verify_chain(&chain, &root_of(&keys));

        assert_jwt_error(result, JwtError::Expired);
    }

    #[test]
    fn xbox_chain_not_yet_valid_is_rejected() {
        let keys = chain_keys();
        let future = json!({ "nbf": now() + 3600, "exp": now() + 7200 });
        let chain = xbox_chain(&keys, &keys.root, future);

        let result = verify_chain(&chain, &root_of(&keys));

        assert_jwt_error(result, JwtError::NotYetValid);
    }

    #[test]
    fn xbox_chain_with_xuid_from_untrusted_root_is_rejected() {
        let keys = chain_keys();
        let impostor = TestKey::from_seed(13);
        let chain = xbox_chain(&keys, &impostor, valid_window());

        let result = verify_chain(&chain, &root_of(&keys));

        assert!(
            matches!(result, Err(AuthError::XuidWithoutXboxLive)),
            "{result:?}"
        );
    }

    #[test]
    fn real_mojang_root_rejects_test_root() {
        let keys = chain_keys();
        let chain = xbox_chain(&keys, &keys.root, valid_window());

        let result = verify_chain(&chain, &ChainRoot::default());

        assert!(
            matches!(result, Err(AuthError::XuidWithoutXboxLive)),
            "{result:?}"
        );
    }

    #[test]
    fn self_signed_chain_is_unauthenticated() {
        let keys = chain_keys();
        let chain = vec![keys.client.sign(&with_window(
            json!({
                "identityPublicKey": keys.client.public_base64(),
                "extraData": extra_data(""),
            }),
            valid_window(),
        ))];

        let authentication = verify_chain(&chain, &root_of(&keys)).unwrap();

        let Authentication::Unauthenticated(identity) = authentication else {
            panic!("a self-signed chain must not report authenticated: {authentication:?}");
        };
        assert_eq!(identity.display_name, "Steve");
        assert_eq!(identity.public_key, keys.client.public_base64());
    }

    #[test]
    fn self_signed_chain_with_wrong_x5u_is_rejected() {
        let keys = chain_keys();
        let token = keys.client.sign(&with_window(
            json!({
                "identityPublicKey": keys.client.public_base64(),
                "extraData": extra_data(""),
            }),
            valid_window(),
        ));
        let other = keys.intermediate.sign(&json!({}));
        let (other_header, _) = other.split_once('.').unwrap();
        let (_, rest) = token.split_once('.').unwrap();
        let chain = vec![format!("{other_header}.{rest}")];

        let result = verify_chain(&chain, &root_of(&keys));

        assert_jwt_error(result, JwtError::InvalidSignature);
    }

    #[test]
    fn chain_of_two_is_rejected() {
        let keys = chain_keys();
        let mut chain = xbox_chain(&keys, &keys.root, valid_window());
        chain.pop();

        let result = verify_chain(&chain, &root_of(&keys));

        assert!(
            matches!(result, Err(AuthError::ChainLength(2))),
            "{result:?}"
        );
    }
}

use base64::Engine;
use base64::alphabet::URL_SAFE;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use p384::ecdsa::signature::Verifier;
use rsa::{BigUint, RsaPublicKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const BASE64URL: GeneralPurpose = GeneralPurpose::new(
    &URL_SAFE,
    GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub const LEEWAY_SECS: f64 = 60.0;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("token is not three dot-separated segments")]
    Malformed,
    #[error("token segment is not base64url: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("token segment is not the expected JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("token is signed with {found}, expected {expected}")]
    InvalidAlgorithm { expected: Algorithm, found: String },
    #[error("token signature does not verify")]
    InvalidSignature,
    #[error("token has no exp claim")]
    MissingExpiry,
    #[error("token has expired")]
    Expired,
    #[error("token is not valid yet")]
    NotYetValid,
    #[error("token issuer is not the expected one")]
    InvalidIssuer,
    #[error("token audience is not the expected one")]
    InvalidAudience,
    #[error("JWK is not a usable RSA key")]
    InvalidJwk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    ES384,
    RS256,
}

impl Algorithm {
    pub fn name(self) -> &'static str {
        match self {
            Self::ES384 => "ES384",
            Self::RS256 => "RS256",
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub alg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x5u: Option<String>,
}

pub enum VerifyingKey {
    Es384(p384::ecdsa::VerifyingKey),
    Rs256(rsa::pkcs1v15::VerifyingKey<Sha256>),
}

impl VerifyingKey {
    fn algorithm(&self) -> Algorithm {
        match self {
            Self::Es384(_) => Algorithm::ES384,
            Self::Rs256(_) => Algorithm::RS256,
        }
    }

    fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), JwtError> {
        let verified = match self {
            Self::Es384(key) => p384::ecdsa::Signature::from_slice(signature)
                .and_then(|signature| key.verify(message, &signature)),
            Self::Rs256(key) => rsa::pkcs1v15::Signature::try_from(signature)
                .and_then(|signature| key.verify(message, &signature)),
        };
        verified.map_err(|_| JwtError::InvalidSignature)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expiry {
    Required,
    Ignored,
}

pub struct Rules<'a> {
    pub expiry: Expiry,
    pub check_not_before: bool,
    pub issuer: Option<&'a str>,
    pub audience: Option<&'a str>,
}

#[derive(Deserialize)]
struct RegisteredClaims {
    exp: Option<f64>,
    nbf: Option<f64>,
    iss: Option<String>,
    #[serde(default)]
    aud: Audience,
}

#[derive(Deserialize, Default)]
#[serde(untagged)]
enum Audience {
    #[default]
    Absent,
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn contains(&self, audience: &str) -> bool {
        match self {
            Self::Absent => false,
            Self::One(one) => one == audience,
            Self::Many(many) => many.iter().any(|one| one == audience),
        }
    }
}

impl Rules<'_> {
    fn check(&self, claims: &RegisteredClaims, now: f64) -> Result<(), JwtError> {
        if self.expiry == Expiry::Required {
            let exp = claims.exp.ok_or(JwtError::MissingExpiry)?;
            if exp < now - LEEWAY_SECS {
                return Err(JwtError::Expired);
            }
        }
        if self.check_not_before && claims.nbf.is_some_and(|nbf| nbf > now + LEEWAY_SECS) {
            return Err(JwtError::NotYetValid);
        }
        if let Some(issuer) = self.issuer
            && claims.iss.as_deref() != Some(issuer)
        {
            return Err(JwtError::InvalidIssuer);
        }
        if let Some(audience) = self.audience
            && !claims.aud.contains(audience)
        {
            return Err(JwtError::InvalidAudience);
        }
        Ok(())
    }
}

pub struct Token<'a> {
    pub header: Header,
    signed: &'a str,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl<'a> Token<'a> {
    pub fn parse(token: &'a str) -> Result<Self, JwtError> {
        let (signed, signature) = token.rsplit_once('.').ok_or(JwtError::Malformed)?;
        let (header, payload) = signed.split_once('.').ok_or(JwtError::Malformed)?;
        if payload.contains('.') {
            return Err(JwtError::Malformed);
        }
        Ok(Self {
            header: serde_json::from_slice(&BASE64URL.decode(header)?)?,
            signed,
            payload: BASE64URL.decode(payload)?,
            signature: BASE64URL.decode(signature)?,
        })
    }

    pub fn require_algorithm(&self, expected: Algorithm) -> Result<(), JwtError> {
        if self.header.alg == expected.name() {
            Ok(())
        } else {
            Err(JwtError::InvalidAlgorithm {
                expected,
                found: self.header.alg.clone(),
            })
        }
    }

    pub fn unverified_claims<T: DeserializeOwned>(&self) -> Result<T, JwtError> {
        Ok(serde_json::from_slice(&self.payload)?)
    }

    pub fn verify<T: DeserializeOwned>(
        &self,
        key: &VerifyingKey,
        rules: &Rules,
    ) -> Result<T, JwtError> {
        self.require_algorithm(key.algorithm())?;
        key.verify(self.signed.as_bytes(), &self.signature)?;
        rules.check(&serde_json::from_slice(&self.payload)?, now())?;
        self.unverified_claims()
    }
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64())
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

impl JwkSet {
    pub fn find(&self, kid: &str) -> Option<&Jwk> {
        self.keys.iter().find(|key| key.kid.as_deref() == Some(kid))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

impl Jwk {
    pub fn rs256_key(&self) -> Result<VerifyingKey, JwtError> {
        let (Some(n), Some(e)) = (&self.n, &self.e) else {
            return Err(JwtError::InvalidJwk);
        };
        if self.kty != "RSA" {
            return Err(JwtError::InvalidJwk);
        }
        let key = RsaPublicKey::new(
            BigUint::from_bytes_be(&BASE64URL.decode(n)?),
            BigUint::from_bytes_be(&BASE64URL.decode(e)?),
        )
        .map_err(|_| JwtError::InvalidJwk)?;
        Ok(VerifyingKey::Rs256(rsa::pkcs1v15::VerifyingKey::new(key)))
    }
}

#[cfg(test)]
pub(crate) fn encode(
    header: &Header,
    claims: &impl Serialize,
    sign: impl Fn(&[u8]) -> Vec<u8>,
) -> String {
    let signed = format!(
        "{}.{}",
        BASE64URL.encode(serde_json::to_vec(header).expect("header serializes")),
        BASE64URL.encode(serde_json::to_vec(claims).expect("claims serialize")),
    );
    let signature = BASE64URL.encode(sign(signed.as_bytes()));
    format!("{signed}.{signature}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TestKey, TestRsaKey, now as now_secs};
    use serde_json::{Value, json};

    const RULES: Rules = Rules {
        expiry: Expiry::Required,
        check_not_before: true,
        issuer: Some("Mojang"),
        audience: Some("minecraft"),
    };

    fn claims_valid_until(exp: i64) -> Value {
        json!({ "name": "Steve", "exp": exp, "iss": "Mojang", "aud": "minecraft" })
    }

    fn valid_claims() -> Value {
        claims_valid_until(now_secs() + 600)
    }

    fn es384_key(key: &TestKey) -> VerifyingKey {
        crate::chain::verifying_key(&key.public_base64()).unwrap()
    }

    fn rs256_key() -> VerifyingKey {
        TestRsaKey::shared().jwks("current").keys[0]
            .rs256_key()
            .unwrap()
    }

    fn tampered(token: &str) -> String {
        let (signed, signature) = token.rsplit_once('.').unwrap();
        let mut signature = BASE64URL.decode(signature).unwrap();
        signature[0] ^= 1;
        format!("{signed}.{}", BASE64URL.encode(signature))
    }

    fn verify(token: &str, key: &VerifyingKey) -> Result<Value, JwtError> {
        Token::parse(token)?.verify(key, &RULES)
    }

    #[test]
    fn es384_token_verifies_with_its_key() {
        let key = TestKey::from_seed(1);

        let claims = verify(&key.sign(&valid_claims()), &es384_key(&key)).unwrap();

        assert_eq!(claims["name"], "Steve");
    }

    #[test]
    fn rs256_token_verifies_with_its_key() {
        let claims = verify(&TestRsaKey::shared().sign(&valid_claims()), &rs256_key()).unwrap();

        assert_eq!(claims["name"], "Steve");
    }

    #[test]
    fn es384_token_with_tampered_signature_is_rejected() {
        let key = TestKey::from_seed(1);

        let result = verify(&tampered(&key.sign(&valid_claims())), &es384_key(&key));

        assert!(
            matches!(result, Err(JwtError::InvalidSignature)),
            "{result:?}"
        );
    }

    #[test]
    fn rs256_token_with_tampered_signature_is_rejected() {
        let token = tampered(&TestRsaKey::shared().sign(&valid_claims()));

        let result = verify(&token, &rs256_key());

        assert!(
            matches!(result, Err(JwtError::InvalidSignature)),
            "{result:?}"
        );
    }

    #[test]
    fn es384_token_signed_by_another_key_is_rejected() {
        let token = TestKey::from_seed(2).sign(&valid_claims());

        let result = verify(&token, &es384_key(&TestKey::from_seed(1)));

        assert!(
            matches!(result, Err(JwtError::InvalidSignature)),
            "{result:?}"
        );
    }

    #[test]
    fn token_for_another_algorithm_is_rejected_before_verifying() {
        let token = TestRsaKey::shared().sign(&valid_claims());

        let result = verify(&token, &es384_key(&TestKey::from_seed(1)));

        assert!(
            matches!(
                &result,
                Err(JwtError::InvalidAlgorithm { expected: Algorithm::ES384, found }) if found == "RS256"
            ),
            "{result:?}"
        );
    }

    #[test]
    fn expiry_allows_the_leeway_and_no_more() {
        let key = TestKey::from_seed(1);

        let within = verify(
            &key.sign(&claims_valid_until(now_secs() - 30)),
            &es384_key(&key),
        );
        assert!(within.is_ok(), "{within:?}");
        let beyond = verify(
            &key.sign(&claims_valid_until(now_secs() - 120)),
            &es384_key(&key),
        );
        assert!(matches!(beyond, Err(JwtError::Expired)), "{beyond:?}");
    }

    #[test]
    fn missing_expiry_is_rejected_when_required() {
        let key = TestKey::from_seed(1);
        let mut claims = valid_claims();
        claims.as_object_mut().unwrap().remove("exp");

        let result = verify(&key.sign(&claims), &es384_key(&key));

        assert!(matches!(result, Err(JwtError::MissingExpiry)), "{result:?}");
    }

    #[test]
    fn missing_expiry_is_accepted_when_ignored() {
        let key = TestKey::from_seed(1);
        let rules = Rules {
            expiry: Expiry::Ignored,
            check_not_before: false,
            issuer: None,
            audience: None,
        };
        let token = key.sign(&json!({ "name": "Steve" }));

        let result: Result<Value, _> = Token::parse(&token)
            .unwrap()
            .verify(&es384_key(&key), &rules);

        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn token_not_yet_valid_is_rejected() {
        let key = TestKey::from_seed(1);
        let mut claims = valid_claims();
        claims["nbf"] = json!(now_secs() + 600);

        let result = verify(&key.sign(&claims), &es384_key(&key));

        assert!(matches!(result, Err(JwtError::NotYetValid)), "{result:?}");
    }

    #[test]
    fn wrong_issuer_is_rejected() {
        let key = TestKey::from_seed(1);
        let mut claims = valid_claims();
        claims["iss"] = json!("Someone");

        let result = verify(&key.sign(&claims), &es384_key(&key));

        assert!(matches!(result, Err(JwtError::InvalidIssuer)), "{result:?}");
    }

    #[test]
    fn audience_may_be_a_list() {
        let key = TestKey::from_seed(1);
        let mut claims = valid_claims();

        claims["aud"] = json!(["other", "minecraft"]);
        let listed = verify(&key.sign(&claims), &es384_key(&key));
        assert!(listed.is_ok(), "{listed:?}");

        claims["aud"] = json!(["other"]);
        let missing = verify(&key.sign(&claims), &es384_key(&key));
        assert!(
            matches!(missing, Err(JwtError::InvalidAudience)),
            "{missing:?}"
        );
    }

    #[test]
    fn token_without_three_segments_is_malformed() {
        assert!(matches!(Token::parse("a.b"), Err(JwtError::Malformed)));
        assert!(matches!(Token::parse("a.b.c.d"), Err(JwtError::Malformed)));
    }

    #[test]
    fn jwk_that_is_not_rsa_has_no_rs256_key() {
        let jwks = TestKey::from_seed(1).jwks("current");

        assert!(matches!(
            jwks.keys[0].rs256_key(),
            Err(JwtError::InvalidJwk)
        ));
    }
}

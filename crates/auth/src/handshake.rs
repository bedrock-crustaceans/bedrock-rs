use crate::chain::{encode_es384_with_x5u, parse_public_key, verifying_key};
use crate::error::AuthError;
use crate::jwt::{Algorithm, Expiry, Rules, SALT_BASE64, Token};
use base64::Engine;
use p384::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HandshakeClaims {
    salt: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerHandshake {
    pub server_public_key: PublicKey,
    pub salt: Vec<u8>,
}

pub fn sign_server_handshake(server_key: &SecretKey, salt: &[u8]) -> Result<String, AuthError> {
    let claims = HandshakeClaims {
        salt: SALT_BASE64.encode(salt),
    };
    encode_es384_with_x5u(server_key, &claims)
}

pub fn verify_server_handshake(token: &str) -> Result<ServerHandshake, AuthError> {
    let parsed = Token::parse(token)?;
    parsed.require_algorithm(Algorithm::ES384)?;
    let x5u = parsed
        .header
        .x5u
        .as_deref()
        .ok_or(AuthError::Missing("x5u"))?;
    let rules = Rules {
        expiry: Expiry::Ignored,
        check_not_before: false,
        issuer: None,
        audience: None,
    };
    let claims: HandshakeClaims = parsed.verify(&verifying_key(x5u)?, &rules)?;
    Ok(ServerHandshake {
        server_public_key: parse_public_key(x5u)?,
        salt: SALT_BASE64
            .decode(claims.salt)
            .map_err(AuthError::InvalidSalt)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::encode_public_key;
    use crate::jwt::{Header, JwtError, encode_es384};
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::json;

    fn server_key() -> SecretKey {
        SecretKey::from_slice(&[5; 48]).unwrap()
    }

    fn token_with(header: &Header, claims: &serde_json::Value) -> String {
        encode_es384(header, claims, &server_key()).unwrap()
    }

    fn header_with_x5u(x5u: Option<String>) -> Header {
        Header {
            alg: "ES384".into(),
            kid: None,
            x5u,
        }
    }

    #[test]
    fn server_handshake_round_trips_key_and_salt() {
        let key = server_key();
        let salt = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let handshake =
            verify_server_handshake(&sign_server_handshake(&key, &salt).unwrap()).unwrap();

        assert_eq!(handshake.server_public_key, key.public_key());
        assert_eq!(handshake.salt, salt);
    }

    #[test]
    fn server_handshake_without_x5u_is_rejected() {
        let token = token_with(&header_with_x5u(None), &json!({ "salt": "AQID" }));

        let result = verify_server_handshake(&token);

        assert!(
            matches!(result, Err(AuthError::Missing("x5u"))),
            "{result:?}"
        );
    }

    #[test]
    fn server_handshake_with_tampered_signature_is_rejected() {
        let token = sign_server_handshake(&server_key(), &[1, 2, 3]).unwrap();
        let (signed, signature) = token.rsplit_once('.').unwrap();
        let mut signature = URL_SAFE_NO_PAD.decode(signature).unwrap();
        signature[0] ^= 1;
        let tampered = format!("{signed}.{}", URL_SAFE_NO_PAD.encode(signature));

        let result = verify_server_handshake(&tampered);

        assert!(
            matches!(result, Err(AuthError::Jwt(JwtError::InvalidSignature))),
            "{result:?}"
        );
    }

    #[test]
    fn padded_salt_is_accepted() {
        let x5u = encode_public_key(&server_key().public_key()).unwrap();
        let token = token_with(&header_with_x5u(Some(x5u)), &json!({ "salt": "AQI=" }));

        let handshake = verify_server_handshake(&token).unwrap();

        assert_eq!(handshake.salt, [1, 2]);
    }
}

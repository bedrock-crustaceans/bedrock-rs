use crate::chain::{parse_public_key, verifying_key};
use crate::error::AuthError;
use crate::jwt::{Expiry, Rules, Token};
use serde::de::DeserializeOwned;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub xuid: String,
    pub display_name: String,
    pub identity: Option<String>,
    pub title_id: Option<String>,
    pub playfab_id: Option<String>,
    pub public_key: String,
}

impl Identity {
    pub fn public_key(&self) -> Result<p384::PublicKey, AuthError> {
        parse_public_key(&self.public_key)
    }

    pub fn verify_client_data<T: DeserializeOwned>(&self, token: &str) -> Result<T, AuthError> {
        let rules = Rules {
            expiry: Expiry::Ignored,
            check_not_before: false,
            issuer: None,
            audience: None,
        };
        Ok(Token::parse(token)?.verify(&verifying_key(&self.public_key)?, &rules)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication {
    Authenticated(Identity),
    Unauthenticated(Identity),
}

impl Authentication {
    pub fn identity(&self) -> &Identity {
        match self {
            Self::Authenticated(identity) | Self::Unauthenticated(identity) => identity,
        }
    }

    pub fn into_identity(self) -> Identity {
        match self {
            Self::Authenticated(identity) | Self::Unauthenticated(identity) => identity,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::JwtError;
    use crate::test_support::TestKey;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use serde_json::{Value, json};

    fn identity_of(key: &TestKey) -> Identity {
        Identity {
            xuid: String::new(),
            display_name: "Steve".into(),
            identity: None,
            title_id: None,
            playfab_id: None,
            public_key: key.public_base64(),
        }
    }

    fn client_data() -> Value {
        json!({ "DeviceOS": 7, "ServerAddress": "127.0.0.1:19132", "SkinId": "steve" })
    }

    #[test]
    fn client_data_signed_by_identity_key_decodes() {
        let key = TestKey::from_seed(1);

        let decoded: Value = identity_of(&key)
            .verify_client_data(&key.sign(&client_data()))
            .unwrap();

        assert_eq!(decoded, client_data());
    }

    #[test]
    fn client_data_signed_by_another_key_is_rejected() {
        let key = TestKey::from_seed(1);
        let other = TestKey::from_seed(2);

        let result = identity_of(&key).verify_client_data::<Value>(&other.sign(&client_data()));

        assert!(
            matches!(result, Err(AuthError::Jwt(JwtError::InvalidSignature))),
            "{result:?}"
        );
    }

    #[test]
    fn public_key_parses_the_identity_key() {
        let key = TestKey::from_seed(1);

        let parsed = identity_of(&key).public_key().unwrap();

        let der = p384::pkcs8::EncodePublicKey::to_public_key_der(&parsed).unwrap();
        assert_eq!(STANDARD.encode(der.as_bytes()), key.public_base64());
    }

    #[test]
    fn public_key_rejects_text_that_is_not_a_key() {
        let mut identity = identity_of(&TestKey::from_seed(1));
        identity.public_key = "not a key".into();

        assert!(matches!(
            identity.public_key(),
            Err(AuthError::InvalidPublicKey)
        ));

        identity.public_key = STANDARD.encode(b"valid base64, not a SPKI key");
        assert!(matches!(
            identity.public_key(),
            Err(AuthError::InvalidPublicKey)
        ));
    }
}

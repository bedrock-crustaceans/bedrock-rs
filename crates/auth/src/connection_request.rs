use crate::auth_data::{AuthData, AuthPayload, AuthType};
use crate::authentication::Authentication;
use crate::chain::ChainRoot;
use crate::chain::{encode_es384_with_x5u, encode_public_key};
use crate::client_data::ClientData;
use crate::error::AuthError;
use crate::http::{AsyncHttpClient, HttpClient};
use crate::oidc::AuthOIDC;
use p384::SecretKey;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const SELF_SIGNED_NOT_BEFORE_SKEW_SECS: u64 = 60;
const SELF_SIGNED_LIFETIME_SECS: u64 = 24 * 60 * 60;

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionRequest {
    pub auth: AuthData,
    pub client_data: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Login {
    pub authentication: Authentication,
    pub client_data: ClientData,
}

impl ConnectionRequest {
    pub fn verify(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
    ) -> Result<Login, AuthError> {
        self.verify_client_data(self.auth.validate(oidc, chain_root)?)
    }

    pub fn verify_refreshing(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
        client: &impl HttpClient,
    ) -> Result<Login, AuthError> {
        self.verify_client_data(self.auth.validate_refreshing(oidc, chain_root, client)?)
    }

    pub async fn verify_refreshing_async(
        &self,
        oidc: Option<&AuthOIDC>,
        chain_root: &ChainRoot,
        client: &impl AsyncHttpClient,
    ) -> Result<Login, AuthError> {
        let authentication = self
            .auth
            .validate_refreshing_async(oidc, chain_root, client)
            .await?;
        self.verify_client_data(authentication)
    }

    pub fn verify_client_data(&self, authentication: Authentication) -> Result<Login, AuthError> {
        let client_data: ClientData = authentication
            .identity()
            .verify_client_data(&self.client_data)?;
        client_data.validate()?;
        Ok(Login {
            authentication,
            client_data,
        })
    }

    pub fn self_signed(
        identity_key: &SecretKey,
        display_name: &str,
        client_data: &ClientData,
    ) -> Result<Self, AuthError> {
        let public_key = encode_public_key(&identity_key.public_key())?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let claims = json!({
            "identityPublicKey": public_key,
            "nbf": now.saturating_sub(SELF_SIGNED_NOT_BEFORE_SKEW_SECS),
            "exp": now + SELF_SIGNED_LIFETIME_SECS,
            "extraData": {
                "XUID": "",
                "displayName": display_name,
                "identity": Uuid::new_v4().to_string(),
            },
        });
        let auth = AuthData {
            auth_type: AuthType::Offline,
            auth_payload: AuthPayload::Chain(vec![encode_es384_with_x5u(identity_key, &claims)?]),
        };
        Self::with_auth(auth, identity_key, client_data)
    }

    pub fn with_auth(
        auth: AuthData,
        identity_key: &SecretKey,
        client_data: &ClientData,
    ) -> Result<Self, AuthError> {
        Ok(Self {
            auth,
            client_data: encode_es384_with_x5u(identity_key, client_data)?,
        })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, AuthError> {
        let mut rest = bytes;
        let auth = take_length_prefixed(&mut rest)?;
        let client_data = take_length_prefixed(&mut rest)?;
        if !rest.is_empty() {
            return Err(AuthError::TrailingBytes(rest.len()));
        }

        Ok(Self {
            auth: serde_json::from_slice(auth)?,
            client_data: std::str::from_utf8(client_data)?.to_owned(),
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, AuthError> {
        let auth = serde_json::to_vec(&self.auth)?;
        let mut bytes = Vec::with_capacity(8 + auth.len() + self.client_data.len());
        put_length_prefixed(&mut bytes, &auth)?;
        put_length_prefixed(&mut bytes, self.client_data.as_bytes())?;
        Ok(bytes)
    }
}

fn take_length_prefixed<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], AuthError> {
    let (prefix, rest) = input.split_first_chunk::<4>().ok_or(AuthError::Truncated)?;
    let declared = u32::from_le_bytes(*prefix);
    let field = usize::try_from(declared)
        .ok()
        .and_then(|length| rest.get(..length))
        .ok_or(AuthError::LengthOutOfBounds {
            declared,
            remaining: rest.len(),
        })?;
    *input = &rest[field.len()..];
    Ok(field)
}

fn put_length_prefixed(bytes: &mut Vec<u8>, field: &[u8]) -> Result<(), AuthError> {
    let length = u32::try_from(field.len()).map_err(|_| AuthError::FieldTooLong(field.len()))?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(field);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ClientDataError;
    use crate::jwt::JwtError;
    use crate::test_support::{TestKey, realistic_client_data, valid_window, with_window};
    use serde_json::json;

    fn self_signed_request(identity_key: &TestKey, client_data_key: &TestKey) -> Vec<u8> {
        self_signed_request_with(identity_key, client_data_key, &realistic_client_data())
    }

    fn self_signed_request_with(
        identity_key: &TestKey,
        client_data_key: &TestKey,
        client_data: &serde_json::Value,
    ) -> Vec<u8> {
        let claims = json!({
            "identityPublicKey": identity_key.public_base64(),
            "extraData": { "XUID": "", "displayName": "Steve" },
        });
        ConnectionRequest {
            auth: AuthData {
                auth_type: AuthType::Offline,
                auth_payload: AuthPayload::Chain(vec![
                    identity_key.sign(&with_window(claims, valid_window())),
                ]),
            },
            client_data: client_data_key.sign(client_data),
        }
        .to_bytes()
        .unwrap()
    }

    #[test]
    fn self_signed_request_verifies_as_unauthenticated() {
        let key = p384::SecretKey::from_slice(&[9; 48]).unwrap();
        let client_data: ClientData = serde_json::from_value(realistic_client_data()).unwrap();

        let request = ConnectionRequest::self_signed(&key, "Alex", &client_data).unwrap();
        let login =
            verify_without_oidc(&ConnectionRequest::parse(&request.to_bytes().unwrap()).unwrap())
                .unwrap();

        assert!(!login.authentication.is_authenticated());
        assert_eq!(login.authentication.identity().display_name, "Alex");
        assert_eq!(
            login.authentication.identity().public_key().unwrap(),
            key.public_key()
        );
        assert_eq!(login.client_data, client_data);
    }

    fn verify_without_oidc(request: &ConnectionRequest) -> Result<Login, AuthError> {
        request.verify(None, &ChainRoot::default())
    }

    #[test]
    fn self_signed_login_verifies_identity_and_client_data() {
        let key = TestKey::from_seed(1);
        let request = ConnectionRequest::parse(&self_signed_request(&key, &key)).unwrap();

        let login = verify_without_oidc(&request).unwrap();

        assert!(!login.authentication.is_authenticated());
        assert_eq!(login.authentication.identity().display_name, "Steve");
        assert_eq!(
            login.client_data.skin_id,
            "c18e65aa-7b21-4637-9b63-8ad63622ef01_Alex"
        );
    }

    #[test]
    fn login_with_invalid_client_data_is_rejected() {
        let key = TestKey::from_seed(1);
        let mut client_data = realistic_client_data();
        client_data["DeviceOS"] = json!(0);
        let bytes = self_signed_request_with(&key, &key, &client_data);
        let request = ConnectionRequest::parse(&bytes).unwrap();

        let result = verify_without_oidc(&request);

        assert!(
            matches!(
                result,
                Err(AuthError::ClientData(ClientDataError::DeviceOs(0)))
            ),
            "{result:?}"
        );
    }

    #[test]
    fn login_with_client_data_from_another_key_is_rejected() {
        let key = TestKey::from_seed(1);
        let other = TestKey::from_seed(2);
        let request = ConnectionRequest::parse(&self_signed_request(&key, &other)).unwrap();

        let result = verify_without_oidc(&request);

        assert!(
            matches!(result, Err(AuthError::Jwt(JwtError::InvalidSignature))),
            "{result:?}"
        );
    }

    fn length_prefixed(fields: &[&[u8]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for field in fields {
            bytes.extend_from_slice(&(field.len() as u32).to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    }

    fn chain_request() -> ConnectionRequest {
        ConnectionRequest {
            auth: AuthData {
                auth_type: AuthType::Online,
                auth_payload: AuthPayload::Chain(vec!["a.b.c".into(), "d.e.f".into()]),
            },
            client_data: "g.h.i".into(),
        }
    }

    #[test]
    fn connection_request_parses_certificate_shape() {
        let certificate = br#"{"AuthenticationType":0,"Certificate":"{\"chain\":[\"a.b.c\",\"d.e.f\"]}","Token":""}"#;
        let bytes = length_prefixed(&[certificate, b"g.h.i"]);

        let request = ConnectionRequest::parse(&bytes).unwrap();

        assert_eq!(request, chain_request());
    }

    #[test]
    fn connection_request_parses_token_shape() {
        let certificate =
            br#"{"AuthenticationType":2,"Certificate":"{\"chain\":[\"\"]}","Token":"t.o.k"}"#;
        let bytes = length_prefixed(&[certificate, b"g.h.i"]);

        let request = ConnectionRequest::parse(&bytes).unwrap();

        assert_eq!(request.auth.auth_type, AuthType::Offline);
        assert_eq!(
            request.auth.auth_payload,
            AuthPayload::Token("t.o.k".into())
        );
    }

    #[test]
    fn connection_request_parses_legacy_chain_shape() {
        let certificate = br#"{"chain":["a.b.c","d.e.f"]}"#;
        let bytes = length_prefixed(&[certificate, b"g.h.i"]);

        let request = ConnectionRequest::parse(&bytes).unwrap();

        assert_eq!(request, chain_request());
    }

    #[test]
    fn connection_request_round_trips() {
        let request = chain_request();

        let bytes = request.to_bytes().unwrap();
        let parsed = ConnectionRequest::parse(&bytes).unwrap();

        assert_eq!(parsed, request);
        assert_eq!(parsed.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn connection_request_rejects_truncated_length() {
        let result = ConnectionRequest::parse(&[1, 0]);

        assert!(matches!(result, Err(AuthError::Truncated)), "{result:?}");
    }

    #[test]
    fn connection_request_rejects_length_past_input() {
        let mut bytes = u32::MAX.to_le_bytes().to_vec();
        bytes.extend_from_slice(b"{}");

        let result = ConnectionRequest::parse(&bytes);

        assert!(
            matches!(
                result,
                Err(AuthError::LengthOutOfBounds {
                    declared: u32::MAX,
                    remaining: 2
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn connection_request_rejects_missing_client_data() {
        let bytes = length_prefixed(&[br#"{"chain":[]}"#]);

        let result = ConnectionRequest::parse(&bytes);

        assert!(matches!(result, Err(AuthError::Truncated)), "{result:?}");
    }

    #[test]
    fn connection_request_rejects_trailing_bytes() {
        let mut bytes = length_prefixed(&[br#"{"chain":[]}"#, b"g.h.i"]);
        bytes.push(0);

        let result = ConnectionRequest::parse(&bytes);

        assert!(
            matches!(result, Err(AuthError::TrailingBytes(1))),
            "{result:?}"
        );
    }
}

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bedrock_auth::{
    AsyncHttpClient, AuthData, AuthOIDC, AuthPayload, AuthType, ClientData, ConnectionRequest,
    HttpClient, HttpError, encode_public_key,
};
use bedrock_protocol::V2225;
use bedrock_protocol::v662::enums::PlayStatus;
use bedrock_protocol::v662::packets::PlayStatusPacket;
use p384::SecretKey;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rsa::RsaPrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use rsa::traits::PublicKeyParts;
use serde_json::json;
use std::future::Future;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) fn identity_key() -> SecretKey {
    SecretKey::from_slice(&[0x33u8; 48]).unwrap()
}

pub(crate) fn client_key() -> SecretKey {
    SecretKey::from_slice(&[0x11u8; 48]).unwrap()
}

pub(crate) fn self_signed_request_bytes() -> Vec<u8> {
    let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Alex");
    ConnectionRequest::self_signed(&identity_key(), "Alex", &client_data)
        .unwrap()
        .to_bytes()
        .unwrap()
}

const ISSUER: &str = "https://authorization.franchise.minecraft-services.net";

fn oidc_key() -> &'static RsaPrivateKey {
    static KEY: OnceLock<RsaPrivateKey> = OnceLock::new();
    KEY.get_or_init(|| {
        RsaPrivateKey::new(&mut StdRng::seed_from_u64(7), 1024).expect("RSA key generates")
    })
}

fn jwks_json(kid: &str) -> serde_json::Value {
    let key = oidc_key();
    json!({ "keys": [{
        "kty": "RSA",
        "kid": kid,
        "use": "sig",
        "alg": "RS256",
        "n": URL_SAFE_NO_PAD.encode(key.n().to_be_bytes()),
        "e": URL_SAFE_NO_PAD.encode(key.e().to_be_bytes()),
    }] })
}

const JWKS_URI: &str = "https://jwks.invalid";

fn oidc_trusting(kid: &str) -> AuthOIDC {
    let jwks = serde_json::from_value(jwks_json(kid)).expect("JWKS parses");
    AuthOIDC::new(ISSUER, AuthOIDC::AUDIENCE, JWKS_URI, jwks)
}

pub(crate) fn trusted_oidc() -> AuthOIDC {
    oidc_trusting("current")
}

pub(crate) fn oidc_before_rotation() -> AuthOIDC {
    let mut oidc = oidc_trusting("previous");
    oidc.refresh_interval = Duration::ZERO;
    oidc
}

pub(crate) struct RotatedKeysHttp {
    published_kid: &'static str,
    requests: Mutex<Vec<String>>,
}

impl RotatedKeysHttp {
    pub(crate) fn publishing(published_kid: &'static str) -> Self {
        Self {
            published_kid,
            requests: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

impl RotatedKeysHttp {
    fn respond(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        self.requests.lock().unwrap().push(url.to_owned());
        if url == JWKS_URI {
            Ok(jwks_json(self.published_kid).to_string().into_bytes())
        } else {
            Err(format!("no response for {url}").into())
        }
    }
}

impl AsyncHttpClient for RotatedKeysHttp {
    fn get(&self, url: &str) -> impl Future<Output = Result<Vec<u8>, HttpError>> {
        std::future::ready(self.respond(url))
    }
}

impl HttpClient for RotatedKeysHttp {
    fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        self.respond(url)
    }
}

fn oidc_token(display_name: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let header = json!({ "alg": "RS256", "kid": "current" });
    let claims = json!({
        "mid": "playfab", "xid": "2535400000000001", "xname": display_name,
        "cpk": encode_public_key(&client_key().public_key()).unwrap(),
        "iss": ISSUER, "aud": AuthOIDC::AUDIENCE,
        "nbf": now - 60, "exp": now + 3600,
    });
    let message = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(header.to_string()),
        URL_SAFE_NO_PAD.encode(claims.to_string())
    );
    let signer = rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new(oidc_key().clone());
    let signature = URL_SAFE_NO_PAD.encode(signer.sign(message.as_bytes()).to_vec());
    format!("{message}.{signature}")
}

fn oidc_login_with_token(auth_type: AuthType, token: String) -> Vec<u8> {
    let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Guest");
    let auth = AuthData {
        auth_type,
        auth_payload: AuthPayload::Token(token),
    };
    let request = ConnectionRequest::with_auth(auth, &client_key(), &client_data).unwrap();
    request.to_bytes().unwrap()
}

pub(crate) fn verified_guest_login() -> Vec<u8> {
    oidc_login_with_token(AuthType::Guest, oidc_token("Guest"))
}

pub(crate) fn verified_online_login() -> Vec<u8> {
    oidc_login_with_token(AuthType::Online, oidc_token("Guest"))
}

pub(crate) fn forged_guest_login() -> Vec<u8> {
    let genuine = oidc_token("Guest");
    let altered = oidc_token("Host");
    let parts = genuine.split('.').collect::<Vec<_>>();
    let altered_payload = altered.split('.').nth(1).unwrap();
    oidc_login_with_token(
        AuthType::Guest,
        format!("{}.{altered_payload}.{}", parts[0], parts[2]),
    )
}

pub(crate) fn self_signed_login(display_name: &str) -> Vec<u8> {
    let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", display_name);
    let request =
        ConnectionRequest::self_signed(&client_key(), display_name, &client_data).unwrap();
    request.to_bytes().unwrap()
}

pub(crate) fn login_success() -> V2225 {
    V2225::PlayStatusPacket(Box::new(PlayStatusPacket {
        status: PlayStatus::LoginSuccess,
    }))
}

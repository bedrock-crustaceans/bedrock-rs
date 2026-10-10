use crate::jwt::{Header, JwkSet};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use p384::SecretKey;
use p384::ecdsa::signature::{SignatureEncoding, Signer};
use p384::elliptic_curve::sec1::ToSec1Point;
use p384::pkcs8::EncodePublicKey;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rsa::RsaPrivateKey;
use rsa::traits::PublicKeyParts;
use serde_json::{Value, json};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TestKey(SecretKey);

impl TestKey {
    pub fn from_seed(seed: u8) -> Self {
        Self(SecretKey::from_slice(&[seed; 48]).expect("seed is a valid P-384 scalar"))
    }

    pub fn public_base64(&self) -> String {
        let der = self
            .0
            .public_key()
            .to_public_key_der()
            .expect("public key encodes as SPKI DER");
        STANDARD.encode(der.as_bytes())
    }

    pub fn jwks(&self, kid: &str) -> JwkSet {
        let point = self.0.public_key().to_sec1_point(false);
        let (x, y) = point.as_bytes()[1..].split_at(48);
        serde_json::from_value(json!({ "keys": [{
            "kty": "EC",
            "kid": kid,
            "crv": "P-384",
            "x": URL_SAFE_NO_PAD.encode(x),
            "y": URL_SAFE_NO_PAD.encode(y),
        }] }))
        .expect("JWKS parses")
    }

    pub fn sign_with_kid(&self, claims: &Value, kid: &str) -> String {
        self.sign_with_header(
            Header {
                alg: "ES384".into(),
                kid: Some(kid.into()),
                x5u: None,
            },
            claims,
        )
    }

    pub fn sign(&self, claims: &Value) -> String {
        self.sign_with_header(
            Header {
                alg: "ES384".into(),
                kid: None,
                x5u: Some(self.public_base64()),
            },
            claims,
        )
    }

    fn sign_with_header(&self, header: Header, claims: &Value) -> String {
        crate::jwt::encode_es384(&header, claims, &self.0).expect("token encodes")
    }
}

pub struct TestRsaKey(RsaPrivateKey);

impl TestRsaKey {
    pub const SIGNING_KID: &str = "current";

    pub fn shared() -> &'static Self {
        static KEY: OnceLock<TestRsaKey> = OnceLock::new();
        KEY.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(7);
            Self(RsaPrivateKey::new(&mut rng, 1024).expect("RSA key generates"))
        })
    }

    pub fn jwks(&self, kid: &str) -> JwkSet {
        serde_json::from_value(json!({ "keys": [{
            "kty": "RSA",
            "kid": kid,
            "use": "sig",
            "alg": "RS256",
            "n": URL_SAFE_NO_PAD.encode(self.0.n().to_be_bytes()),
            "e": URL_SAFE_NO_PAD.encode(self.0.e().to_be_bytes()),
        }] }))
        .expect("JWKS parses")
    }

    pub fn sign(&self, claims: &Value) -> String {
        let header = Header {
            alg: "RS256".into(),
            kid: Some(Self::SIGNING_KID.into()),
            x5u: None,
        };
        let key = rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new(self.0.clone());
        crate::jwt::encode(&header, claims, |message| key.sign(message).to_vec())
            .expect("token encodes")
    }
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs() as i64
}

pub fn valid_window() -> Value {
    json!({ "nbf": now() - 3600, "exp": now() + 3600 })
}

pub fn with_window(mut claims: Value, window: Value) -> Value {
    if let (Some(claims), Some(window)) = (claims.as_object_mut(), window.as_object()) {
        claims.extend(window.clone());
    }
    claims
}

const CLIENT_DATA: &str = r##"{
    "AnimatedImageData": [{
        "AnimationExpression": 1,
        "Frames": 2.0,
        "Image": "",
        "ImageHeight": 64,
        "ImageWidth": 32,
        "Type": 1
    }],
    "ArmSize": "wide",
    "CapeData": "",
    "CapeId": "",
    "CapeImageHeight": 0,
    "CapeImageWidth": 0,
    "CapeOnClassicSkin": false,
    "ClientEditorConnectionIntent": 0,
    "ClientIsEditorCapable": false,
    "ClientRandomId": -6342519382542385612,
    "CompatibleWithClientSideChunkGen": true,
    "CurrentInputMode": 1,
    "DefaultInputMode": 1,
    "DeviceId": "a4f365bb1e04459bbe4cb3cbf9d546e0",
    "DeviceModel": "System Product Name (ASUS)",
    "DeviceOS": 7,
    "FilterProfanity": false,
    "GameVersion": "1.21.120",
    "GraphicsMode": 1,
    "GuiScale": 0,
    "IsEditorMode": false,
    "IsPartyLeader": false,
    "LanguageCode": "en_US",
    "MaxViewDistance": 96,
    "MemoryTier": 4,
    "OverrideSkin": false,
    "PartyId": "",
    "PersonaPieces": [{
        "IsDefault": true,
        "PackId": "68bfe60d-f30a-422f-b32c-72374ebdd057",
        "PieceId": "8f96d1f8-e9bb-40d2-acc8-eb79746c5d7c",
        "PieceType": "persona_skeleton",
        "ProductId": ""
    }],
    "PersonaSkin": false,
    "PieceTintColors": [{
        "Colors": ["#ffa12722", "#ff2f1f0f", "#ff3aafd9", "#0"],
        "PieceType": "persona_eyes"
    }],
    "PlatformOfflineId": "",
    "PlatformOnlineId": "",
    "PlatformType": 0,
    "PlayFabId": "8a7f3c2e1d5b9046",
    "PremiumSkin": false,
    "ProfileHash": "",
    "SelfSignedId": "05601fd2-9c71-30b1-b174-5fa11b9de09f",
    "ServerAddress": "play.example.com:19132",
    "SkinAnimationData": "",
    "SkinColor": "#0",
    "SkinData": "",
    "SkinGeometryData": "eyJmb3JtYXRfdmVyc2lvbiI6IjEuMTIuMCIsIm1pbmVjcmFmdDpnZW9tZXRyeSI6W119",
    "SkinGeometryDataEngineVersion": "MS4yMS4xMjA=",
    "SkinId": "c18e65aa-7b21-4637-9b63-8ad63622ef01_Alex",
    "SkinImageHeight": 64,
    "SkinImageWidth": 64,
    "SkinResourcePatch": "eyJnZW9tZXRyeSI6eyJkZWZhdWx0IjoiZ2VvbWV0cnkuaHVtYW5vaWQuY3VzdG9tU2xpbSJ9fQ==",
    "ThirdPartyName": "Steve",
    "ThirdPartyNameOnly": false,
    "TrustedSkin": true,
    "UIProfile": 0
}"##;

pub fn realistic_client_data() -> Value {
    let mut data: Value = serde_json::from_str(CLIENT_DATA).expect("client data parses");
    data["SkinData"] = json!(STANDARD.encode(vec![0u8; 64 * 64 * 4]));
    data["AnimatedImageData"][0]["Image"] = json!(STANDARD.encode(vec![0u8; 32 * 64 * 4]));
    data
}

pub struct FakeHttp {
    responses: std::collections::HashMap<String, Vec<u8>>,
    requests: std::cell::RefCell<Vec<String>>,
}

impl FakeHttp {
    pub fn serving(responses: &[(&str, Value)]) -> Self {
        Self {
            responses: responses
                .iter()
                .map(|(url, body)| (url.to_string(), body.to_string().into_bytes()))
                .collect(),
            requests: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> Vec<String> {
        self.requests.borrow().clone()
    }

    fn respond(&self, url: &str) -> Result<Vec<u8>, crate::http::HttpError> {
        self.requests.borrow_mut().push(url.to_owned());
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| format!("no response for {url}").into())
    }
}

impl crate::http::HttpClient for FakeHttp {
    fn get(&self, url: &str) -> Result<Vec<u8>, crate::http::HttpError> {
        self.respond(url)
    }
}

impl crate::http::AsyncHttpClient for FakeHttp {
    async fn get(&self, url: &str) -> Result<Vec<u8>, crate::http::HttpError> {
        self.respond(url)
    }
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    match future.as_mut().poll(&mut context) {
        std::task::Poll::Ready(output) => output,
        std::task::Poll::Pending => panic!("test future was not immediately ready"),
    }
}

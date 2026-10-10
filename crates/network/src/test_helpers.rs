use bedrock_auth::{ClientData, ConnectionRequest};
use bedrock_protocol::V2225;
use p384::SecretKey;

use crate::login::LoginPackets;

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

pub(crate) fn self_signed_login(display_name: &str) -> V2225 {
    let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", display_name);
    let request =
        ConnectionRequest::self_signed(&client_key(), display_name, &client_data).unwrap();
    V2225::login(V2225::PROTOCOL_VERSION, request.to_bytes().unwrap())
}

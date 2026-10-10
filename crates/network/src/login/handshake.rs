use crate::encryption::Encryption;
use crate::error::LoginError;
use p384::{PublicKey, SecretKey};

pub fn server_begin_encryption(
    client_identity: &PublicKey,
    server_key: &SecretKey,
    salt: &[u8],
) -> Result<(String, Encryption), LoginError> {
    let token = bedrock_auth::sign_server_handshake(server_key, salt)?;
    let encryption = Encryption::new(server_key, client_identity, salt);
    Ok((token, encryption))
}

pub fn client_finish_encryption(
    token: &str,
    identity_key: &SecretKey,
) -> Result<Encryption, LoginError> {
    let handshake = bedrock_auth::verify_server_handshake(token)?;
    Ok(Encryption::new(
        identity_key,
        &handshake.server_public_key,
        &handshake.salt,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_and_client_derive_the_same_keystream() {
        let client_key = SecretKey::from_slice(&[0x11u8; 48]).unwrap();
        let server_key = SecretKey::from_slice(&[0x22u8; 48]).unwrap();
        let salt = rand::random::<[u8; 16]>();

        let (token, mut server) =
            server_begin_encryption(&client_key.public_key(), &server_key, &salt).unwrap();
        let mut client = client_finish_encryption(&token, &client_key).unwrap();

        let down = server.encrypt(b"server to client".to_vec()).unwrap();
        assert_eq!(client.decrypt(down).unwrap(), b"server to client");

        let up = client.encrypt(b"client to server".to_vec()).unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"client to server");
    }
}

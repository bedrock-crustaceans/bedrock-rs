use crate::encryption::Encryption;
use crate::error::LoginError;
use bedrock_auth::Login;
use p384::elliptic_curve::common::Generate;
use p384::{PublicKey, SecretKey};

/// Builds the handshake token and the cipher for a verified login, using a fresh server key and
/// salt. Send the token, then enable the cipher on the connection.
///
/// # Errors
///
/// Returns [`LoginError::Auth`] when the login's identity key is invalid or the token cannot be
/// signed.
pub fn begin_encryption(login: &Login) -> Result<(String, Encryption), LoginError> {
    let identity = login.authentication.identity().public_key()?;
    let server_key = SecretKey::generate_from_rng(&mut rand::rng());
    let salt = rand::random::<[u8; 16]>();
    server_begin_encryption(&identity, &server_key, &salt)
}

/// Builds the handshake token and the cipher from the client's identity key, the server key and
/// the salt.
///
/// # Errors
///
/// Returns [`LoginError::Auth`] when the token cannot be signed.
pub fn server_begin_encryption(
    client_identity: &PublicKey,
    server_key: &SecretKey,
    salt: &[u8],
) -> Result<(String, Encryption), LoginError> {
    let token = bedrock_auth::sign_server_handshake(server_key, salt)?;
    let encryption = Encryption::new(server_key, client_identity, salt);
    Ok((token, encryption))
}

/// Verifies the server's handshake `token` and builds the client's cipher.
///
/// # Errors
///
/// Returns [`LoginError::Auth`] when `token` is not a valid server handshake.
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

        let down = server.encrypt(b"server to client").unwrap();
        assert_eq!(client.decrypt(down).unwrap(), b"server to client");

        let up = client.encrypt(b"client to server").unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"client to server");
    }
}

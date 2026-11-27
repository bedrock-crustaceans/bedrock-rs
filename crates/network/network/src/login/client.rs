use super::handshake::client_finish_encryption;
use crate::encryption::Encryption;
use crate::error::LoginError;
use bedrock_auth::{AuthData, ClientData, ConnectionRequest};
use p384::SecretKey;
use p384::elliptic_curve::common::Generate;

/// The key a client logs in with and the connection request it signed with that key.
pub struct ClientIdentity {
    key: SecretKey,
    request: Vec<u8>,
}

impl ClientIdentity {
    pub fn new(key: SecretKey, request: Vec<u8>) -> Self {
        Self { key, request }
    }

    pub fn generate_key() -> SecretKey {
        SecretKey::generate_from_rng(&mut rand::rng())
    }

    /// A self-signed identity carrying `display_name`, for servers that do not authenticate.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError::Auth`] when the connection request cannot be signed or encoded.
    pub fn offline(
        key: SecretKey,
        display_name: &str,
        client_data: &ClientData,
    ) -> Result<Self, LoginError> {
        let request = ConnectionRequest::self_signed(&key, display_name, client_data)?;
        Ok(Self::new(key, request.to_bytes()?))
    }

    /// An identity whose connection request carries an existing `auth` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError::Auth`] when the connection request cannot be signed or encoded.
    pub fn online(
        key: SecretKey,
        auth: AuthData,
        client_data: &ClientData,
    ) -> Result<Self, LoginError> {
        let request = ConnectionRequest::with_auth(auth, &key, client_data)?;
        Ok(Self::new(key, request.to_bytes()?))
    }

    pub fn key(&self) -> &SecretKey {
        &self.key
    }

    /// The bytes of the login packet's connection request.
    pub fn request(&self) -> &[u8] {
        &self.request
    }

    /// Verifies the server handshake token and builds the cipher for the connection.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError::Auth`] when `token` is not a valid server handshake.
    pub fn finish_encryption(&self, token: &str) -> Result<Encryption, LoginError> {
        client_finish_encryption(token, &self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_auth::ChainRoot;

    fn key() -> SecretKey {
        SecretKey::from_slice(&[0x55u8; 48]).unwrap()
    }

    fn verified(identity: &ClientIdentity) -> bedrock_auth::Login {
        ConnectionRequest::parse(identity.request())
            .unwrap()
            .verify(None, &ChainRoot::default())
            .unwrap()
    }

    #[test]
    fn offline_identity_carries_its_name_and_key() {
        let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Alex");
        let identity = ClientIdentity::offline(key(), "Alex", &client_data).unwrap();

        let login = verified(&identity);
        let claimed = login.authentication.identity();
        assert_eq!(claimed.display_name, "Alex");
        assert_eq!(claimed.public_key().unwrap(), key().public_key());
        assert_eq!(login.client_data.server_address, "127.0.0.1:19132");
    }

    #[test]
    fn generated_keys_differ() {
        assert_ne!(
            ClientIdentity::generate_key().public_key(),
            ClientIdentity::generate_key().public_key()
        );
    }

    #[test]
    fn finish_encryption_matches_the_server_side() {
        let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Alex");
        let identity = ClientIdentity::offline(key(), "Alex", &client_data).unwrap();
        let login = verified(&identity);

        let (token, mut server) = super::super::begin_encryption(&login).unwrap();
        let mut client = identity.finish_encryption(&token).unwrap();

        let up = client.encrypt(b"hello").unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"hello");
    }
}

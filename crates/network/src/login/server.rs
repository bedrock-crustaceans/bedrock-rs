use crate::error::LoginError;
use bedrock_auth::{
    AsyncHttpClient, AuthData, AuthOIDC, Authentication, ChainRoot, ConnectionRequest, HttpClient,
    Login,
};
use std::sync::Arc;

/// How a server verifies the connection request of a login packet.
#[derive(Clone)]
pub struct LoginOptions {
    chain_root: ChainRoot,
    oidc: Option<Arc<AuthOIDC>>,
    require_authentication: bool,
    allow_guests: bool,
}

impl Default for LoginOptions {
    fn default() -> Self {
        Self {
            chain_root: ChainRoot::default(),
            oidc: None,
            require_authentication: true,
            allow_guests: false,
        }
    }
}

impl LoginOptions {
    #[must_use]
    pub fn chain_root(mut self, chain_root: ChainRoot) -> Self {
        self.chain_root = chain_root;
        self
    }

    #[must_use]
    pub fn oidc(mut self, oidc: impl Into<Arc<AuthOIDC>>) -> Self {
        self.oidc = Some(oidc.into());
        self
    }

    /// Refuses logins that are not signed in to an online account. On by default.
    #[must_use]
    pub fn require_authentication(mut self, required: bool) -> Self {
        self.require_authentication = required;
        self
    }

    /// Accepts guest (split-screen) logins whose token verifies against the OIDC keys. Off by
    /// default.
    #[must_use]
    pub fn allow_guests(mut self, allowed: bool) -> Self {
        self.allow_guests = allowed;
        self
    }

    /// Verifies the connection request bytes of a login packet against the cached OIDC keys.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError::Auth`] when the request fails to parse or its chain, token or client
    /// data does not verify, and [`LoginError::NotAuthenticated`] when authentication is required
    /// and the login is not signed in to an online account.
    pub fn verify(&self, request: &[u8]) -> Result<Login, LoginError> {
        let request = ConnectionRequest::parse(request)?;
        let oidc = self.oidc.as_deref();
        let authentication = if self.allow_guests {
            request
                .auth
                .validate_allowing_guests(oidc, &self.chain_root)?
        } else {
            request.auth.validate(oidc, &self.chain_root)?
        };
        self.finish(&request, authentication)
    }

    /// Like [`Self::verify`], but refetches the OIDC keys once when the login names a key id
    /// the cache lacks. The refetch is rate limited by the keys themselves.
    ///
    /// # Errors
    ///
    /// The errors of [`Self::verify`], and [`LoginError::Auth`] when the refetch fails.
    pub fn verify_refreshing(
        &self,
        request: &[u8],
        client: &impl HttpClient,
    ) -> Result<Login, LoginError> {
        let request = ConnectionRequest::parse(request)?;
        let oidc = self.oidc.as_deref();
        let authentication = if self.allow_guests {
            request
                .auth
                .validate_allowing_guests_refreshing(oidc, &self.chain_root, client)?
        } else {
            request
                .auth
                .validate_refreshing(oidc, &self.chain_root, client)?
        };
        self.finish(&request, authentication)
    }

    /// Async twin of [`Self::verify_refreshing`].
    ///
    /// # Errors
    ///
    /// The errors of [`Self::verify_refreshing`].
    pub async fn verify_refreshing_async(
        &self,
        request: &[u8],
        client: &impl AsyncHttpClient,
    ) -> Result<Login, LoginError> {
        let request = ConnectionRequest::parse(request)?;
        let oidc = self.oidc.as_deref();
        let authentication = self
            .validate_refreshing_async(&request.auth, oidc, client)
            .await?;
        self.finish(&request, authentication)
    }

    async fn validate_refreshing_async(
        &self,
        auth: &AuthData,
        oidc: Option<&AuthOIDC>,
        client: &impl AsyncHttpClient,
    ) -> Result<Authentication, bedrock_auth::AuthError> {
        if self.allow_guests {
            auth.validate_allowing_guests_refreshing_async(oidc, &self.chain_root, client)
                .await
        } else {
            auth.validate_refreshing_async(oidc, &self.chain_root, client)
                .await
        }
    }

    fn finish(
        &self,
        request: &ConnectionRequest,
        authentication: Authentication,
    ) -> Result<Login, LoginError> {
        let login = request.verify_client_data(authentication)?;
        if self.require_authentication && !login.authentication.is_authenticated() {
            return Err(LoginError::NotAuthenticated);
        }
        Ok(login)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::handshake::{begin_encryption, client_finish_encryption};
    use crate::test_helpers::{
        RotatedKeysHttp, client_key, forged_guest_login, identity_key, oidc_before_rotation,
        self_signed_login, trusted_oidc, verified_guest_login, verified_online_login,
    };
    use bedrock_auth::{AuthError, ClientData};

    fn offline() -> LoginOptions {
        LoginOptions::default().require_authentication(false)
    }

    fn guests() -> LoginOptions {
        offline().oidc(trusted_oidc()).allow_guests(true)
    }

    fn rotated() -> LoginOptions {
        LoginOptions::default().oidc(oidc_before_rotation())
    }

    fn is_unknown_key_id(result: &Result<Login, LoginError>) -> bool {
        matches!(result, Err(LoginError::Auth(AuthError::UnknownKeyId(_))))
    }

    #[test]
    fn a_self_signed_login_is_accepted_when_authentication_is_not_required() {
        let login = offline().verify(&self_signed_login("Steve")).unwrap();

        assert_eq!(login.authentication.identity().display_name, "Steve");
    }

    #[test]
    fn a_self_signed_login_is_refused_by_default() {
        let result = LoginOptions::default().verify(&self_signed_login("Steve"));

        assert!(matches!(result, Err(LoginError::NotAuthenticated)));
    }

    #[test]
    fn a_tampered_connection_request_is_refused() {
        let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Steve");
        let genuine = ConnectionRequest::self_signed(&client_key(), "Steve", &client_data).unwrap();
        let forged =
            ConnectionRequest::with_auth(genuine.auth, &identity_key(), &client_data).unwrap();

        let result = offline().verify(&forged.to_bytes().unwrap());

        assert!(matches!(result, Err(LoginError::Auth(_))));
    }

    #[test]
    fn a_truncated_connection_request_is_refused() {
        let mut bytes = self_signed_login("Steve");
        bytes.truncate(bytes.len() / 2);

        assert!(matches!(offline().verify(&bytes), Err(LoginError::Auth(_))));
    }

    #[test]
    fn a_verified_guest_login_is_accepted_when_guests_are_allowed() {
        let login = guests().verify(&verified_guest_login()).unwrap();

        assert!(
            login.authentication.is_guest(),
            "{:?}",
            login.authentication
        );
        assert_eq!(login.authentication.identity().display_name, "Guest");
    }

    #[test]
    fn a_verified_guest_login_is_refused_unless_guests_are_allowed() {
        let options = offline().oidc(trusted_oidc());

        let result = options.verify(&verified_guest_login());

        assert!(
            matches!(result, Err(LoginError::Auth(AuthError::GuestLogin))),
            "{result:?}"
        );
    }

    #[test]
    fn a_guest_login_with_a_forged_token_is_refused_when_guests_are_allowed() {
        let result = guests().verify(&forged_guest_login());

        assert!(
            matches!(result, Err(LoginError::Auth(AuthError::Jwt(_)))),
            "{result:?}"
        );
    }

    #[test]
    fn a_login_signed_by_a_rotated_key_is_refused_without_a_refresh() {
        assert!(is_unknown_key_id(
            &rotated().verify(&verified_online_login())
        ));
    }

    #[test]
    fn a_login_signed_by_a_rotated_key_is_accepted_after_one_refetch() {
        let http = RotatedKeysHttp::publishing("current");

        let login = rotated()
            .verify_refreshing(&verified_online_login(), &http)
            .unwrap();

        assert!(login.authentication.is_authenticated());
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[::tokio::test]
    async fn a_login_signed_by_a_rotated_key_is_accepted_after_one_async_refetch() {
        let http = RotatedKeysHttp::publishing("current");

        let login = rotated()
            .verify_refreshing_async(&verified_online_login(), &http)
            .await
            .unwrap();

        assert!(login.authentication.is_authenticated());
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[test]
    fn a_login_signed_by_an_unpublished_key_is_refused_after_one_refetch() {
        let http = RotatedKeysHttp::publishing("previous");

        let result = rotated().verify_refreshing(&verified_online_login(), &http);

        assert!(is_unknown_key_id(&result));
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[::tokio::test]
    async fn a_login_signed_by_an_unpublished_key_is_refused_after_one_async_refetch() {
        let http = RotatedKeysHttp::publishing("previous");

        let result = rotated()
            .verify_refreshing_async(&verified_online_login(), &http)
            .await;

        assert!(is_unknown_key_id(&result));
        assert_eq!(http.requests(), ["https://jwks.invalid"]);
    }

    #[test]
    fn the_handshake_token_gives_both_sides_the_same_keystream() {
        let login = offline().verify(&self_signed_login("Steve")).unwrap();

        let (token, mut server) = begin_encryption(&login).unwrap();
        let mut client = client_finish_encryption(&token, &client_key()).unwrap();

        let down = server.encrypt(b"server to client").unwrap();
        assert_eq!(client.decrypt(down).unwrap(), b"server to client");
        let up = client.encrypt(b"client to server").unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"client to server");
    }

    #[test]
    fn a_handshake_token_from_another_server_key_does_not_decrypt() {
        let login = offline().verify(&self_signed_login("Steve")).unwrap();
        let (token, _) = begin_encryption(&login).unwrap();
        let (_, mut server) = begin_encryption(&login).unwrap();
        let mut client = client_finish_encryption(&token, &client_key()).unwrap();

        let up = client.encrypt(b"client to server").unwrap();

        assert_ne!(
            server.decrypt(up).ok().as_deref(),
            Some(&b"client to server"[..])
        );
    }
}

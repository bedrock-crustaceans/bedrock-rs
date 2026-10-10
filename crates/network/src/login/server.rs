use super::LoginAction;
use super::handshake::server_begin_encryption;
use super::packets::{DisconnectInfo, LoginEvent, LoginPackets, LoginStatus, NetworkSettings};
use crate::compression::Compression;
use crate::error::LoginError;
use bedrock_auth::{AuthOIDC, ChainRoot, Login};
use p384::SecretKey;
use p384::elliptic_curve::common::Generate;
use std::sync::Arc;

pub type ServerLoginAction<V> = LoginAction<V, Box<Login>>;

#[derive(Clone)]
pub struct ServerLoginOptions {
    compression: Compression,
    encryption: bool,
    chain_root: ChainRoot,
    oidc: Option<Arc<AuthOIDC>>,
    require_authentication: bool,
}

impl ServerLoginOptions {
    pub fn compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    pub fn encryption(mut self, enabled: bool) -> Self {
        self.encryption = enabled;
        self
    }

    pub fn chain_root(mut self, chain_root: ChainRoot) -> Self {
        self.chain_root = chain_root;
        self
    }

    pub fn oidc(mut self, oidc: impl Into<Arc<AuthOIDC>>) -> Self {
        self.oidc = Some(oidc.into());
        self
    }

    pub fn require_authentication(mut self, required: bool) -> Self {
        self.require_authentication = required;
        self
    }
}

impl Default for ServerLoginOptions {
    fn default() -> Self {
        Self {
            compression: Compression::Zlib {
                threshold: 256,
                compression_level: 6,
            },
            encryption: true,
            chain_root: ChainRoot::default(),
            oidc: None,
            require_authentication: true,
        }
    }
}

pub struct LoginFailure<V> {
    pub farewell: Vec<V>,
    pub error: LoginError,
}

impl<V> LoginFailure<V> {
    pub fn with_farewell(error: impl Into<LoginError>, packet: V) -> Self {
        Self {
            farewell: vec![packet],
            error: error.into(),
        }
    }
}

impl<V, E: Into<LoginError>> From<E> for LoginFailure<V> {
    fn from(error: E) -> Self {
        Self {
            farewell: Vec::new(),
            error: error.into(),
        }
    }
}

impl<V> std::fmt::Debug for LoginFailure<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginFailure")
            .field("farewell", &self.farewell.len())
            .field("error", &self.error)
            .finish()
    }
}

enum ServerState {
    AwaitSettings,
    AwaitLogin { protocol: i32 },
    AwaitHandshake { login: Box<Login> },
    Done,
}

impl ServerState {
    fn name(&self) -> &'static str {
        match self {
            Self::AwaitSettings => "awaiting network settings request",
            Self::AwaitLogin { .. } => "awaiting login",
            Self::AwaitHandshake { .. } => "awaiting client handshake",
            Self::Done => "login finished",
        }
    }
}

fn finish<V: LoginPackets>(login: Box<Login>) -> Vec<ServerLoginAction<V>> {
    vec![
        LoginAction::Send(vec![V::play_status(LoginStatus::LoginSuccess)]),
        LoginAction::Complete(login),
    ]
}

pub struct ServerLogin {
    options: ServerLoginOptions,
    state: ServerState,
}

impl ServerLogin {
    pub fn new(options: ServerLoginOptions) -> Self {
        Self {
            options,
            state: ServerState::AwaitSettings,
        }
    }

    pub fn handle<V: LoginPackets>(
        &mut self,
        packet: V,
    ) -> Result<Vec<ServerLoginAction<V>>, LoginFailure<V>> {
        match (
            std::mem::replace(&mut self.state, ServerState::Done),
            packet.into_event(),
        ) {
            (ServerState::AwaitSettings, LoginEvent::RequestNetworkSettings { protocol }) => {
                self.accept_settings_request(protocol)
            }
            (
                ServerState::AwaitLogin {
                    protocol: requested,
                },
                LoginEvent::Login { protocol, request },
            ) => self.accept_request(requested, protocol, &request),
            (ServerState::AwaitHandshake { login }, LoginEvent::ClientHandshake) => {
                Ok(finish(login))
            }
            (state, _) => {
                let name = state.name();
                self.state = state;
                Err(LoginError::UnexpectedPacket { state: name }.into())
            }
        }
    }

    fn accept_settings_request<V: LoginPackets>(
        &mut self,
        protocol: i32,
    ) -> Result<Vec<ServerLoginAction<V>>, LoginFailure<V>> {
        if protocol != V::PROTOCOL_VERSION {
            let status = if protocol < V::PROTOCOL_VERSION {
                LoginStatus::LoginFailedClientOld
            } else {
                LoginStatus::LoginFailedServerOld
            };
            return Err(LoginFailure::with_farewell(
                LoginError::ProtocolMismatch {
                    client: protocol,
                    server: V::PROTOCOL_VERSION,
                },
                V::play_status(status),
            ));
        }
        self.state = ServerState::AwaitLogin { protocol };
        let settings = NetworkSettings {
            compression: self.options.compression.clone(),
        };
        Ok(vec![
            LoginAction::Send(vec![V::network_settings(&settings)]),
            LoginAction::EnableCompression(settings.compression),
        ])
    }

    fn accept_request<V: LoginPackets>(
        &mut self,
        requested: i32,
        protocol: i32,
        request: &[u8],
    ) -> Result<Vec<ServerLoginAction<V>>, LoginFailure<V>> {
        if protocol != requested {
            return Err(LoginError::ProtocolChanged {
                requested,
                login: protocol,
            }
            .into());
        }
        let login = bedrock_auth::ConnectionRequest::parse(request)?
            .verify(self.options.oidc.as_deref(), &self.options.chain_root)?;
        if self.options.require_authentication && !login.authentication.is_authenticated() {
            return Err(LoginFailure::with_farewell(
                LoginError::NotAuthenticated,
                V::disconnect(&DisconnectInfo {
                    message: Some("disconnectionScreen.notAuthenticated".to_string()),
                }),
            ));
        }
        if self.options.encryption {
            self.begin_encryption(login)
        } else {
            Ok(finish(Box::new(login)))
        }
    }

    fn begin_encryption<V: LoginPackets>(
        &mut self,
        login: Login,
    ) -> Result<Vec<ServerLoginAction<V>>, LoginFailure<V>> {
        let identity = login.authentication.identity().public_key()?;
        let server_key = SecretKey::generate_from_rng(&mut rand::rng());
        let salt = rand::random::<[u8; 16]>();
        let (token, encryption) = server_begin_encryption(&identity, &server_key, &salt)?;
        self.state = ServerState::AwaitHandshake {
            login: Box::new(login),
        };
        Ok(vec![
            LoginAction::Send(vec![V::server_handshake(token)]),
            LoginAction::EnableEncryption(Box::new(encryption)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::handshake::client_finish_encryption;
    use crate::test_helpers::{client_key, identity_key, self_signed_login};
    use bedrock_auth::{ClientData, ConnectionRequest};
    use bedrock_protocol::V2225;

    fn expect_failure(login: &mut ServerLogin, packet: V2225) -> LoginFailure<V2225> {
        match login.handle(packet) {
            Err(failure) => failure,
            Ok(_) => panic!("the packet should have been rejected"),
        }
    }

    fn only_status(failure: LoginFailure<V2225>) -> LoginStatus {
        assert_eq!(failure.farewell.len(), 1);
        match failure.farewell.into_iter().next().unwrap().into_event() {
            LoginEvent::PlayStatus(status) => status,
            _ => panic!("farewell should be a play status"),
        }
    }

    fn offline_config() -> ServerLoginOptions {
        ServerLoginOptions::default()
            .compression(Compression::Snappy { threshold: 256 })
            .encryption(false)
            .require_authentication(false)
    }

    fn handle(login: &mut ServerLogin, packet: V2225) -> Vec<ServerLoginAction<V2225>> {
        login.handle(packet).unwrap()
    }

    #[test]
    fn server_login_without_encryption_succeeds() {
        let mut login = ServerLogin::new(offline_config());

        let mut actions = handle(
            &mut login,
            V2225::request_network_settings(V2225::PROTOCOL_VERSION),
        )
        .into_iter();
        let Some(LoginAction::Send(sent)) = actions.next() else {
            panic!("network settings are sent first");
        };
        assert_eq!(sent.len(), 1);
        assert!(matches!(
            sent.into_iter().next().unwrap().into_event(),
            LoginEvent::NetworkSettings(NetworkSettings {
                compression: Compression::Snappy { threshold: 256 }
            })
        ));
        assert!(matches!(
            actions.next(),
            Some(LoginAction::EnableCompression(Compression::Snappy {
                threshold: 256
            }))
        ));
        assert!(actions.next().is_none());

        let mut actions = handle(&mut login, self_signed_login("Steve")).into_iter();
        let Some(LoginAction::Send(sent)) = actions.next() else {
            panic!("login success is sent first");
        };
        assert!(matches!(
            sent.into_iter().next().unwrap().into_event(),
            LoginEvent::PlayStatus(LoginStatus::LoginSuccess)
        ));
        let Some(LoginAction::Complete(done)) = actions.next() else {
            panic!("login completes");
        };
        assert_eq!(done.authentication.identity().display_name, "Steve");
        assert!(actions.next().is_none());
    }

    #[test]
    fn login_before_network_settings_is_rejected() {
        let mut login = ServerLogin::new(offline_config());
        let failure = expect_failure(&mut login, self_signed_login("Steve"));
        assert!(matches!(failure.error, LoginError::UnexpectedPacket { .. }));
    }

    #[test]
    fn older_client_is_told_to_update() {
        let mut login = ServerLogin::new(offline_config());
        let failure = expect_failure(
            &mut login,
            V2225::request_network_settings(V2225::PROTOCOL_VERSION - 1),
        );
        assert!(matches!(
            failure.error,
            LoginError::ProtocolMismatch { client, server }
                if client == V2225::PROTOCOL_VERSION - 1 && server == V2225::PROTOCOL_VERSION
        ));
        assert_eq!(only_status(failure), LoginStatus::LoginFailedClientOld);
    }

    #[test]
    fn newer_client_is_told_the_server_is_old() {
        let mut login = ServerLogin::new(offline_config());
        let failure = expect_failure(
            &mut login,
            V2225::request_network_settings(V2225::PROTOCOL_VERSION + 1),
        );
        assert!(matches!(failure.error, LoginError::ProtocolMismatch { .. }));
        assert_eq!(only_status(failure), LoginStatus::LoginFailedServerOld);
    }

    #[test]
    fn login_protocol_differing_from_request_is_rejected() {
        let mut login = ServerLogin::new(offline_config());
        handle(
            &mut login,
            V2225::request_network_settings(V2225::PROTOCOL_VERSION),
        );
        let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Steve");
        let request = ConnectionRequest::self_signed(&client_key(), "Steve", &client_data).unwrap();
        let failure = expect_failure(
            &mut login,
            V2225::login(V2225::PROTOCOL_VERSION - 1, request.to_bytes().unwrap()),
        );
        assert!(matches!(
            failure.error,
            LoginError::ProtocolChanged { requested, login }
                if requested == V2225::PROTOCOL_VERSION && login == V2225::PROTOCOL_VERSION - 1
        ));
    }

    fn online_config() -> ServerLoginOptions {
        ServerLoginOptions::default().encryption(false)
    }

    fn ready_for_login(config: ServerLoginOptions) -> ServerLogin {
        let mut login = ServerLogin::new(config);
        handle(
            &mut login,
            V2225::request_network_settings(V2225::PROTOCOL_VERSION),
        );
        login
    }

    #[test]
    fn tampered_connection_request_is_rejected() {
        let client_data = ClientData::offline("1.21.0", "127.0.0.1:19132", "Steve");
        let genuine = ConnectionRequest::self_signed(&client_key(), "Steve", &client_data).unwrap();
        let other_key = identity_key();
        let forged = ConnectionRequest::with_auth(genuine.auth, &other_key, &client_data).unwrap();
        let mut login = ready_for_login(offline_config());
        let failure = expect_failure(
            &mut login,
            V2225::login(V2225::PROTOCOL_VERSION, forged.to_bytes().unwrap()),
        );
        assert!(matches!(failure.error, LoginError::Auth(_)));
    }

    #[test]
    fn unauthenticated_login_in_online_mode_is_disconnected() {
        let mut login = ready_for_login(online_config());
        let failure = expect_failure(&mut login, self_signed_login("Steve"));
        assert!(matches!(failure.error, LoginError::NotAuthenticated));
        assert_eq!(failure.farewell.len(), 1);
        assert!(matches!(
            failure.farewell.into_iter().next().unwrap().into_event(),
            LoginEvent::Disconnect(_)
        ));
    }

    fn encrypting_config() -> ServerLoginOptions {
        ServerLoginOptions::default()
            .encryption(true)
            .require_authentication(false)
    }

    #[test]
    fn server_login_with_encryption_handshakes_before_login_success() {
        let mut login = ready_for_login(encrypting_config());

        let mut actions = handle(&mut login, self_signed_login("Steve")).into_iter();
        let Some(LoginAction::Send(sent)) = actions.next() else {
            panic!("the server handshake is sent first");
        };
        let LoginEvent::ServerHandshake(token) = sent.into_iter().next().unwrap().into_event()
        else {
            panic!("the first packet is the server handshake");
        };
        let Some(LoginAction::EnableEncryption(mut server)) = actions.next() else {
            panic!("encryption is enabled after the handshake is sent");
        };
        assert!(actions.next().is_none());

        let mut client = client_finish_encryption(&token, &client_key()).unwrap();
        let down = server.encrypt(b"server to client".to_vec()).unwrap();
        assert_eq!(client.decrypt(down).unwrap(), b"server to client");
        let up = client.encrypt(b"client to server".to_vec()).unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"client to server");

        let mut actions = handle(&mut login, V2225::client_handshake()).into_iter();
        let Some(LoginAction::Send(sent)) = actions.next() else {
            panic!("login success follows the client handshake");
        };
        assert!(matches!(
            sent.into_iter().next().unwrap().into_event(),
            LoginEvent::PlayStatus(LoginStatus::LoginSuccess)
        ));
        let Some(LoginAction::Complete(done)) = actions.next() else {
            panic!("login completes");
        };
        assert_eq!(done.authentication.identity().display_name, "Steve");
    }

    #[test]
    fn packet_other_than_client_handshake_is_rejected_while_encrypting() {
        let mut login = ready_for_login(encrypting_config());
        handle(&mut login, self_signed_login("Steve"));
        let failure = expect_failure(&mut login, self_signed_login("Steve"));
        assert!(matches!(failure.error, LoginError::UnexpectedPacket { .. }));
    }
}

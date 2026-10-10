use super::LoginAction;
use super::handshake::client_finish_encryption;
use super::packets::{LoginEvent, LoginPackets, LoginStatus};
use crate::error::LoginError;
use p384::SecretKey;

enum ClientState {
    AwaitSettings,
    AwaitHandshakeOrSuccess,
    AwaitSuccess,
    Done,
}

impl ClientState {
    fn name(&self) -> &'static str {
        match self {
            Self::AwaitSettings => "awaiting network settings",
            Self::AwaitHandshakeOrSuccess => "awaiting server handshake or login success",
            Self::AwaitSuccess => "awaiting login success",
            Self::Done => "login finished",
        }
    }
}

pub struct ClientIdentity {
    key: SecretKey,
    request: Vec<u8>,
}

impl ClientIdentity {
    pub fn new(key: SecretKey, request: Vec<u8>) -> Self {
        Self { key, request }
    }
}

pub struct ClientLogin {
    state: ClientState,
    identity_key: SecretKey,
    request: Option<Vec<u8>>,
}

impl ClientLogin {
    pub fn new(identity: ClientIdentity) -> Self {
        Self {
            state: ClientState::AwaitSettings,
            identity_key: identity.key,
            request: Some(identity.request),
        }
    }

    pub fn start<V: LoginPackets>(&mut self) -> Vec<LoginAction<V>> {
        vec![LoginAction::Send(vec![V::request_network_settings(
            V::PROTOCOL_VERSION,
        )])]
    }

    pub fn handle<V: LoginPackets>(
        &mut self,
        packet: V,
    ) -> Result<Vec<LoginAction<V>>, LoginError> {
        match (&self.state, packet.into_event()) {
            (ClientState::AwaitSettings, LoginEvent::NetworkSettings(settings)) => {
                let request = self.request.take().ok_or(LoginError::UnexpectedPacket {
                    state: "login request already sent",
                })?;
                self.state = ClientState::AwaitHandshakeOrSuccess;
                Ok(vec![
                    LoginAction::EnableCompression(settings.compression),
                    LoginAction::Send(vec![V::login(V::PROTOCOL_VERSION, request)]),
                ])
            }
            (ClientState::AwaitHandshakeOrSuccess, LoginEvent::ServerHandshake(token)) => {
                let encryption = client_finish_encryption(&token, &self.identity_key)?;
                self.state = ClientState::AwaitSuccess;
                Ok(vec![
                    LoginAction::EnableEncryption(Box::new(encryption)),
                    LoginAction::Send(vec![V::client_handshake()]),
                ])
            }
            (
                ClientState::AwaitHandshakeOrSuccess | ClientState::AwaitSuccess,
                LoginEvent::PlayStatus(LoginStatus::LoginSuccess),
            ) => {
                self.state = ClientState::Done;
                Ok(vec![LoginAction::Complete(())])
            }
            (_, LoginEvent::Disconnect(info)) => Err(LoginError::Disconnected(info)),
            (_, LoginEvent::PlayStatus(status)) if status.is_failure() => {
                Err(LoginError::LoginRejected(status))
            }
            _ => Err(LoginError::UnexpectedPacket {
                state: self.state.name(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::Compression;
    use crate::login::packets::{LoginEvent, NetworkSettings};
    use crate::test_helpers::{identity_key, self_signed_request_bytes};
    use bedrock_protocol::V2225;

    fn client_with(request: Vec<u8>) -> ClientLogin {
        ClientLogin::new(ClientIdentity::new(identity_key(), request))
    }

    fn sent(action: Option<LoginAction<V2225>>) -> Vec<LoginEvent<V2225>> {
        match action {
            Some(LoginAction::Send(packets)) => {
                packets.into_iter().map(LoginPackets::into_event).collect()
            }
            _ => panic!("expected a send action"),
        }
    }

    #[test]
    fn client_applies_negotiated_compression_before_login() {
        let expected = self_signed_request_bytes();
        let mut login = client_with(expected.clone());

        let mut start = login.start::<V2225>().into_iter();
        let events = sent(start.next());
        assert!(matches!(
            events.as_slice(),
            [LoginEvent::RequestNetworkSettings { protocol }] if *protocol == V2225::PROTOCOL_VERSION
        ));
        assert!(start.next().is_none());

        let compression = Compression::Snappy { threshold: 256 };
        let settings = V2225::network_settings(&NetworkSettings {
            compression: compression.clone(),
        });
        let mut actions = login.handle(settings).unwrap().into_iter();
        assert!(matches!(
            actions.next(),
            Some(LoginAction::EnableCompression(Compression::Snappy {
                threshold: 256
            }))
        ));
        let events = sent(actions.next());
        assert!(matches!(
            events.as_slice(),
            [LoginEvent::Login { protocol, request }]
                if *protocol == V2225::PROTOCOL_VERSION && *request == expected
        ));
        assert!(actions.next().is_none());
    }

    #[test]
    fn client_completes_encrypted_login() {
        use crate::login::handshake::server_begin_encryption;
        use crate::login::packets::LoginStatus;

        let mut login = client_with(self_signed_request_bytes());
        login.start::<V2225>();
        login
            .handle(V2225::network_settings(&NetworkSettings {
                compression: Compression::None,
            }))
            .unwrap();

        let server_key = SecretKey::from_slice(&[0x44u8; 48]).unwrap();
        let (token, mut server) =
            server_begin_encryption(&identity_key().public_key(), &server_key, &[7u8; 16]).unwrap();
        let mut actions = login
            .handle(V2225::server_handshake(token))
            .unwrap()
            .into_iter();
        let Some(LoginAction::EnableEncryption(mut client)) = actions.next() else {
            panic!("encryption is enabled first");
        };
        let events = sent(actions.next());
        assert!(matches!(events.as_slice(), [LoginEvent::ClientHandshake]));
        assert!(actions.next().is_none());

        let up = client.encrypt(b"hello".to_vec()).unwrap();
        assert_eq!(server.decrypt(up).unwrap(), b"hello");

        let mut actions = login
            .handle(V2225::play_status(LoginStatus::LoginSuccess))
            .unwrap()
            .into_iter();
        assert!(matches!(actions.next(), Some(LoginAction::Complete(()))));
        assert!(actions.next().is_none());
    }

    #[test]
    fn client_completes_unencrypted_login() {
        use crate::login::packets::LoginStatus;

        let mut login = client_with(self_signed_request_bytes());
        login.start::<V2225>();
        login
            .handle(V2225::network_settings(&NetworkSettings {
                compression: Compression::None,
            }))
            .unwrap();
        let mut actions = login
            .handle(V2225::play_status(LoginStatus::LoginSuccess))
            .unwrap()
            .into_iter();
        assert!(matches!(actions.next(), Some(LoginAction::Complete(()))));
    }

    fn after_settings() -> ClientLogin {
        let mut login = client_with(self_signed_request_bytes());
        login.start::<V2225>();
        login
            .handle(V2225::network_settings(&NetworkSettings {
                compression: Compression::None,
            }))
            .unwrap();
        login
    }

    #[test]
    fn failed_play_status_rejects_the_login() {
        use crate::login::packets::LoginStatus;

        let mut login = after_settings();
        let result = login.handle(V2225::play_status(LoginStatus::LoginFailedServerOld));
        assert!(matches!(
            result,
            Err(LoginError::LoginRejected(LoginStatus::LoginFailedServerOld))
        ));
    }

    #[test]
    fn disconnect_during_login_carries_the_message() {
        use crate::login::packets::DisconnectInfo;

        let mut login = after_settings();
        let result = login.handle(V2225::disconnect(&DisconnectInfo {
            message: Some("full".to_string()),
        }));
        assert!(matches!(
            result,
            Err(LoginError::Disconnected(DisconnectInfo { message: Some(ref m) })) if m == "full"
        ));
    }

    #[test]
    fn handshake_before_network_settings_is_unexpected() {
        let mut login = client_with(self_signed_request_bytes());
        login.start::<V2225>();
        let result = login.handle(V2225::server_handshake("a.b.c".to_string()));
        assert!(matches!(result, Err(LoginError::UnexpectedPacket { .. })));
    }
}

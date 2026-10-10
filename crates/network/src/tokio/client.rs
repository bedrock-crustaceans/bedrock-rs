use super::connection::Connection;
use super::error::{ClientError, RakNetError};
use super::session::Session;
use super::transport::{RakNetConnection, TransportLayerConnection};
use crate::login::LoginPackets;
use crate::login::client::ClientIdentity;
use bedrock_auth::{AuthData, ClientData, ConnectionRequest};
use bedrock_protocol::ProtoVersion;
use p384::SecretKey;
use p384::elliptic_curve::common::Generate;
use raknet_tokio::prelude::RakClient;
use std::net::SocketAddr;
use std::time::Duration;

pub struct Client {
    identity_key: Option<SecretKey>,
    client_data: Option<ClientData>,
    display_name: String,
    auth: Option<AuthData>,
    timeout: Option<Duration>,
}

impl Client {
    pub fn offline(display_name: impl Into<String>) -> Self {
        Self {
            identity_key: None,
            client_data: None,
            display_name: display_name.into(),
            auth: None,
            timeout: None,
        }
    }

    pub fn online(auth: AuthData, client_data: ClientData) -> Self {
        Self {
            identity_key: None,
            client_data: Some(client_data),
            display_name: String::new(),
            auth: Some(auth),
            timeout: None,
        }
    }

    pub fn identity_key(mut self, key: SecretKey) -> Self {
        self.identity_key = Some(key);
        self
    }

    pub fn client_data(mut self, client_data: ClientData) -> Self {
        self.client_data = Some(client_data);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub(crate) fn connection_request(
        &self,
        identity_key: &SecretKey,
        game_version: &str,
        addr: SocketAddr,
    ) -> Result<ConnectionRequest, ClientError> {
        let client_data = self.client_data.clone().unwrap_or_else(|| {
            ClientData::offline(game_version, &addr.to_string(), &self.display_name)
        });
        let request = match &self.auth {
            Some(auth) => ConnectionRequest::with_auth(auth.clone(), identity_key, &client_data)?,
            None => ConnectionRequest::self_signed(identity_key, &self.display_name, &client_data)?,
        };
        Ok(request)
    }

    pub async fn connect<V: LoginPackets + ProtoVersion>(
        &self,
        addr: SocketAddr,
    ) -> Result<Session<V>, ClientError> {
        match self.timeout {
            Some(limit) => ::tokio::time::timeout(limit, self.connect_untimed::<V>(addr))
                .await
                .map_err(|_| ClientError::Timeout)?,
            None => self.connect_untimed::<V>(addr).await,
        }
    }

    async fn connect_untimed<V: LoginPackets + ProtoVersion>(
        &self,
        addr: SocketAddr,
    ) -> Result<Session<V>, ClientError> {
        let identity_key = self
            .identity_key
            .clone()
            .unwrap_or_else(|| SecretKey::generate_from_rng(&mut rand::rng()));
        let request = self
            .connection_request(&identity_key, V::GAME_VERSION, addr)?
            .to_bytes()?;

        let mut client =
            RakClient::new(|config| config.protocol = <V as LoginPackets>::RAKNET_VERSION);
        client.start().await.map_err(RakNetError::from)?;
        let session = match client.connect(addr).await {
            Ok(session) => session,
            Err(error) => {
                client.stop().await;
                return Err(RakNetError::from(error).into());
            }
        };

        let connection = Connection::<V>::from_transport_conn(TransportLayerConnection::RakNet(
            RakNetConnection::dialled(session, client),
        ));
        Ok(connection
            .connect(ClientIdentity::new(identity_key, request))
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_auth::ChainRoot;
    use bedrock_protocol::V2225;

    fn identity_key() -> SecretKey {
        SecretKey::from_slice(&[0x55u8; 48]).unwrap()
    }

    fn server_addr() -> SocketAddr {
        "127.0.0.1:19132".parse().unwrap()
    }

    fn verified(request: &ConnectionRequest) -> bedrock_auth::Login {
        ConnectionRequest::parse(&request.to_bytes().unwrap())
            .unwrap()
            .verify(None, &ChainRoot::default())
            .unwrap()
    }

    #[test]
    fn dialer_builds_request_from_its_identity() {
        let request = Client::offline("Alex")
            .connection_request(&identity_key(), V2225::GAME_VERSION, server_addr())
            .unwrap();

        let login = verified(&request);
        let identity = login.authentication.identity();
        assert_eq!(identity.display_name, "Alex");
        assert_eq!(identity.public_key().unwrap(), identity_key().public_key());
    }

    #[test]
    fn dialer_derives_client_data_from_the_target_when_none_is_given() {
        let request = Client::offline("Alex")
            .connection_request(&identity_key(), V2225::GAME_VERSION, server_addr())
            .unwrap();

        let client_data = verified(&request).client_data;
        assert_eq!(client_data.game_version, V2225::GAME_VERSION);
        assert_eq!(client_data.server_address, "127.0.0.1:19132");
    }

    #[test]
    fn dialer_prefers_explicit_client_data() {
        let custom = ClientData::offline("9.9.9", "example.org:1", "Alex");
        let request = Client::offline("Alex")
            .client_data(custom)
            .connection_request(&identity_key(), V2225::GAME_VERSION, server_addr())
            .unwrap();

        assert_eq!(verified(&request).client_data.game_version, "9.9.9");
    }
}

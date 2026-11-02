use super::connection::Connection;
use super::error::{ClientError, RakNetError};
use super::transport::{RakNetConnection, TransportLayerConnection};
use bedrock_protocol::ProtoVersion;
use bedrock_protocol_core::Packets;
use raknet_tokio::prelude::RakClient;
use std::net::SocketAddr;
use std::time::Duration;

/// Dials servers over `RakNet`. The result is a transport [`Connection`]; the caller performs the
/// login with the helpers in [`crate::login`].
#[derive(Clone, Debug, Default)]
pub struct Client {
    timeout: Option<Duration>,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Connects to the server at `addr`, giving up after the configured timeout.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Timeout`] when the timeout elapses and [`ClientError::RakNet`] when
    /// the `RakNet` client fails to start or connect.
    pub async fn connect<V: Packets + ProtoVersion>(
        &self,
        addr: SocketAddr,
    ) -> Result<Connection<V>, ClientError> {
        match self.timeout {
            Some(limit) => ::tokio::time::timeout(limit, Self::dial::<V>(addr))
                .await
                .map_err(|_| ClientError::Timeout)?,
            None => Self::dial::<V>(addr).await,
        }
    }

    async fn dial<V: Packets + ProtoVersion>(
        addr: SocketAddr,
    ) -> Result<Connection<V>, ClientError> {
        let mut client = RakClient::new(|config| config.protocol = V::RAKNET_VERSION);
        client.start().await.map_err(RakNetError::from)?;
        let session = match client.connect(addr).await {
            Ok(session) => session,
            Err(error) => {
                client.stop().await;
                return Err(RakNetError::from(error).into());
            }
        };

        Ok(Connection::from_transport_conn(
            TransportLayerConnection::RakNet(RakNetConnection::dialled(session, client)),
        ))
    }
}

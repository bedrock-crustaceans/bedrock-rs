use crate::motd::BedrockMOTD;
use crate::tokio::error::{RakNetError, TransportLayerError};
#[cfg(feature = "nethernet-tokio")]
use crate::tokio::transport::NetherNetConnection;
use crate::tokio::transport::TransportLayerConnection;
use raknet_tokio::prelude::*;

#[cfg(feature = "nethernet-tokio")]
pub struct NetherNetListener {
    server: nethernet_tokio::NetherServer,
    signaling: nethernet_tokio::ServerSignaling,
}

#[cfg(feature = "nethernet-tokio")]
impl NetherNetListener {
    /// Binds the signaling transport and starts a `NetherNet` server on it.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportLayerError`] when the server fails to bind.
    pub async fn bind(
        signaling: impl Into<nethernet_tokio::ServerSignaling>,
        config: nethernet_tokio::ConnectionConfig,
    ) -> Result<Self, TransportLayerError> {
        let signaling = signaling.into();
        let server = nethernet_tokio::NetherServer::bind_with(signaling.clone(), config).await?;
        Ok(Self { server, signaling })
    }

    pub(crate) fn advertise(&self, motd: &BedrockMOTD) {
        let data = nethernet_tokio::ServerData::from(motd);
        match &self.signaling {
            nethernet_tokio::ServerSignaling::Lan(lan) => lan.set_server_data(data),
            nethernet_tokio::ServerSignaling::Http(http) => http.set_server_data(data),
        }
    }
}

pub enum TransportLayerListener {
    RakNet(RakServer),
    #[cfg(feature = "nethernet-tokio")]
    NetherNet(NetherNetListener),
    // TODO: Quic(s2n_quic::server::Server),
    // TODO: Tcp(...),
}

impl TransportLayerListener {
    pub fn advertise(&mut self, motd: &BedrockMOTD) {
        match self {
            Self::RakNet(listener) => listener.set_message::<Box<[u8]>>(motd.into()),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => listener.advertise(motd),
        }
    }

    pub fn set_max_connections(&mut self, n: usize) {
        match self {
            Self::RakNet(listener) => listener.set_max_connections(n),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(_) => {}
        }
    }

    /// Starts listening.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportLayerError`] when the `RakNet` server fails to start.
    pub async fn start(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.start().await.map_err(RakNetError::from)?,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(_) => {}
        }

        Ok(())
    }

    /// Stops listening.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportLayerError`] when the `NetherNet` server fails to close.
    pub async fn stop(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.stop().await,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => listener.server.close().await?,
        }

        Ok(())
    }

    /// Waits for the next connection.
    ///
    /// # Errors
    ///
    /// Returns a [`TransportLayerError`] when the listener fails to accept a session.
    pub async fn accept(&mut self) -> Result<TransportLayerConnection, TransportLayerError> {
        let conn = match self {
            Self::RakNet(listener) => TransportLayerConnection::RakNet(
                listener.accept().await.map_err(RakNetError::from)?.into(),
            ),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => TransportLayerConnection::NetherNet(
                NetherNetConnection::accepted(listener.server.accept().await?).await,
            ),
        };

        Ok(conn)
    }
}
